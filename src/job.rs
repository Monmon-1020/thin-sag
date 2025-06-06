use crate::mask::register_secret;
use crate::{
    action::{Action, ActionList},
    adapter::UiAdapter,
    error::ApiError,
    mac_ax::MacAdapter,
    models::{RunRequest, RunResponse},
    vault,
};
use lazy_static::lazy_static;
use regex::Regex;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use uuid::Uuid;

lazy_static! {
    static ref SECRET_REGEX: Regex = Regex::new(r"\{secret\.([a-zA-Z0-9_-]+)\}").unwrap();
}

#[derive(Clone, Copy, serde::Serialize, Debug)]
pub enum JobStatus {
    Pending,
    Running,
    Success,
    Failed,
}

#[derive(Clone)]
pub struct JobResult {
    pub status: JobStatus,
    pub output: Option<String>,
}

pub struct JobManager {
    map: Arc<RwLock<HashMap<String, JobResult>>>,
    sender: mpsc::Sender<(String, JobRequest)>,
    sender_json: mpsc::Sender<(String, ActionList)>,
}

#[derive(Debug)]
pub enum JobRequest {
    Old(RunRequest),
    New(ActionList),
}

impl JobManager {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::channel::<(String, JobRequest)>(100);
        let (tx_json, mut rx_json) = mpsc::channel::<(String, ActionList)>(100);
        let map: Arc<RwLock<HashMap<String, JobResult>>> = Arc::new(RwLock::new(HashMap::new()));
        let map_clone_1 = map.clone();
        let map_clone_2 = map.clone();

        tokio::spawn(async move {
            while let Some((id, req)) = rx.recv().await {
                {
                    let mut guard = map_clone_1.write().await;
                    guard.get_mut(&id).unwrap().status = JobStatus::Running;
                }
                let res = (|| -> Result<RunResponse, ApiError> {
                    if let JobRequest::Old(req) = req {
                        let secret = vault::get_secret(&req.secret).map_err(ApiError::Internal)?;
                        let text = req.text.replace("{secret}", &secret);
                        MacAdapter::new()
                            .launch(&req.bundle)
                            .map_err(ApiError::Internal)?;
                        MacAdapter::new()
                            .type_text(&text)
                            .map_err(ApiError::Internal)?;
                        Ok(RunResponse::success())
                    } else {
                        Err(ApiError::BadRequest(anyhow::anyhow!(
                            "Old API called with new request"
                        )))
                    }
                })();

                let mut guard = map_clone_1.write().await;
                let entry = guard.get_mut(&id).unwrap();
                match res {
                    Ok(res) => {
                        entry.status = JobStatus::Success;
                        entry.output = Some(format!("{:?}", res));
                    }
                    Err(e) => {
                        entry.status = JobStatus::Failed;
                        entry.output = Some(format!("{:?}", RunResponse::fail(e.to_string())));
                    }
                }
            }
        });

        tokio::spawn(async move {
            while let Some((id, actions)) = rx_json.recv().await {
                {
                    let mut guard = map_clone_2.write().await;
                    guard.get_mut(&id).unwrap().status = JobStatus::Running;
                }

                let mut expanded_actions = Vec::with_capacity(actions.0.len());
                for act in actions.0 {
                    let act = match act {
                        Action::Type { text } => {
                            let processed_text = SECRET_REGEX
                                .replace_all(&text, |caps: &regex::Captures| {
                                    let label = &caps[1];
                                    vault::get_secret(label).unwrap_or_else(|_| "".into())
                                })
                                .to_string();
                            Action::Type {
                                text: processed_text,
                            }
                        }
                        other => other,
                    };
                    expanded_actions.push(act);
                }

                let res = execute_actions(&expanded_actions);

                let mut guard = map_clone_2.write().await;
                let entry = guard.get_mut(&id).unwrap();
                match res {
                    Ok(()) => {
                        entry.status = JobStatus::Success;
                        entry.output = Some("Actions executed successfully".to_string());
                    }
                    Err(e) => {
                        entry.status = JobStatus::Failed;
                        entry.output = Some(format!("Error executing actions: {}", e));
                    }
                }
            }
        });

        Self {
            map,
            sender: tx,
            sender_json: tx_json,
        }
    }

    pub async fn enqueue(&self, req: RunRequest) -> String {
        let id = Uuid::new_v4().to_string();
        self.map.write().await.insert(
            id.clone(),
            JobResult {
                status: JobStatus::Pending,
                output: None,
            },
        );
        self.sender
            .send((id.clone(), JobRequest::Old(req)))
            .await
            .unwrap();
        id
    }

    pub async fn enqueue_json(&self, actions: ActionList) -> String {
        let id = Uuid::new_v4().to_string();
        self.map.write().await.insert(
            id.clone(),
            JobResult {
                status: JobStatus::Pending,
                output: None,
            },
        );
        self.sender_json.send((id.clone(), actions)).await.unwrap();
        id
    }

    pub async fn get(&self, id: &str) -> Option<JobResult> {
        self.map.read().await.get(id).cloned()
    }
}

fn execute_actions(actions: &[Action]) -> Result<(), ApiError> {
    let ui = MacAdapter::new();

    for act in actions {
        match act {
            Action::Launch { target } => ui.launch(target)?,
            Action::Type { text } => {
                register_secret(text);
                ui.type_text(text)?;
            }
            Action::Wait { ms } => ui.wait_ms(*ms),
            Action::Click { selector, x, y } => ui.click(selector.as_deref(), *x, *y)?,
            Action::Scroll { dy } => ui.scroll(*dy)?,
            Action::Keypress { key } => ui.keypress(key)?,
            Action::Unsupported => {
                return Err(ApiError::BadRequest(anyhow::anyhow!("unsupported act")))
            }
        }
    }
    Ok(())
}
