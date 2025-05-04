use crate::{models::RunRequest, models::RunResponse, ui_adapter, vault, error::ApiError};
use uuid::Uuid;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tokio::sync::mpsc;

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
    sender: mpsc::Sender<(String, RunRequest)>,
}

impl JobManager {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::channel::<(String, RunRequest)>(100);
        let map: Arc<RwLock<HashMap<String, JobResult>>> = Arc::new(RwLock::new(HashMap::new()));
        let map_clone = map.clone();

        // ワーカータスク (1 本だけ)
        tokio::spawn(async move {
            while let Some((id, req)) = rx.recv().await {
                {
                    let mut guard = map_clone.write().await;
                    guard.get_mut(&id).unwrap().status = JobStatus::Running;
                }
                // 実行
                let res = (|| -> Result<RunResponse, ApiError> {
                    let secret = vault::get_secret(&req.secret)
                        .map_err(ApiError::Internal)?;
                    let text = req.text.replace("{secret}", &secret);
                    ui_adapter::launch_app(&req.bundle).map_err(ApiError::Internal)?;
                    ui_adapter::type_text(&text).map_err(ApiError::Internal)?;
                    Ok(RunResponse::success())
                })();

                let mut guard = map_clone.write().await;
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

        Self { map, sender: tx }
    }

    pub async fn enqueue(&self, req: RunRequest) -> String {
        let id = Uuid::new_v4().to_string();
        self.map.write().await.insert(id.clone(), JobResult{status:JobStatus::Pending,output:None});
        self.sender.send((id.clone(), req)).await.unwrap();
        id
    }

    pub async fn get(&self, id: &str) -> Option<JobResult> {
        self.map.read().await.get(id).cloned()
    }
}
