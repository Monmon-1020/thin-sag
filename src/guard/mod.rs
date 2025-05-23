// src/guard/mod.rs
pub mod audit;
pub mod es;
pub mod ipc;
pub mod rules;

use crate::error::ApiError;
use rules::DangerRule;

/// Event generated from ES events for guard processing
#[derive(Debug)]
pub struct GuardEvent {
    pub pid: i32,
    pub path: String,
}

/// Starts the guard engine
pub async fn start_guard() -> Result<(), ApiError> {
    let rules = rules::load_rules();

    let (tx, mut rx) = tokio::sync::mpsc::channel::<GuardEvent>(100);

    tokio::spawn(async move {
        if let Err(err) = es::start_es_listener(tx).await {
            eprintln!("[guard] ES listener failed: {:?}", err);
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
