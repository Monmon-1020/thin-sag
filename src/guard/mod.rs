// src/guard/mod.rs

pub mod audit;
pub mod es;
pub mod ipc;
pub mod rules;

use crate::error::ApiError;
/// Structure wrapping events from ES
#[derive(Debug)]
pub struct GuardEvent {
    pub pid: i32,
    pub path: String,
}

/// Create a `GuardEvent` from `&Message<'_>`
impl From<&endpoint_sec::Message> for GuardEvent {
    fn from(msg: &endpoint_sec::Message) -> Self {
        use endpoint_sec::sys::es_event_type_t as T;
        use endpoint_sec::Event;
        use endpoint_sec::EventRenameDestinationFile;

        let pid = msg.process().ppid();

        let path = match msg.event_type() {
            T::ES_EVENT_TYPE_AUTH_UNLINK => {
                if let Some(Event::AuthUnlink(unlink)) = msg.event() {
                    unlink.target().path().to_string_lossy().into_owned()
                } else {
                    String::new()
                }
            }
            T::ES_EVENT_TYPE_AUTH_RENAME => {
                if let Some(Event::AuthRename(rename)) = msg.event() {
                    if let Some(dest) = rename.destination() {
                        match dest {
                            EventRenameDestinationFile::ExistingFile(file) => {
                                file.path().to_string_lossy().into_owned()
                            }
                            EventRenameDestinationFile::NewPath {
                                directory,
                                filename,
                            } => {
                                format!(
                                    "{}/{}",
                                    directory.path().to_string_lossy(),
                                    filename.to_string_lossy()
                                )
                            }
                        }
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            }
            T::ES_EVENT_TYPE_AUTH_EXEC => {
                if let Some(Event::AuthExec(exec)) = msg.event() {
                    exec.target()
                        .executable()
                        .path()
                        .to_string_lossy()
                        .into_owned()
                } else {
                    String::new()
                }
            }
            T::ES_EVENT_TYPE_AUTH_OPEN => {
                if let Some(Event::AuthOpen(open)) = msg.event() {
                    open.file().path().to_string_lossy().into_owned()
                } else {
                    String::new()
                }
            }
            _ => String::new(),
        };

        GuardEvent { pid, path }
    }
}

/// Start the Guard engine
///
/// - Spawns `start_es_listener` to run in the background
/// - Processes events received via a tokio mpsc channel
pub async fn start_guard() -> Result<(), ApiError> {
    let rules = rules::load_rules();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<GuardEvent>(100);

    tokio::spawn(async move {
        if let Err(e) = es::start_es_listener(tx) {
            eprintln!("[guard] ES listener failed: {:?}", e);
        }
    });

    while let Some(event) = rx.recv().await {
        for rule in &rules {
            if rule.matches(&event) {
                let decision = ipc::ask_user(&event, &rule.id).await;
                audit::write(&event, &rule.id, &decision).await?;
                break;
            }
        }
    }

    Ok(())
}
