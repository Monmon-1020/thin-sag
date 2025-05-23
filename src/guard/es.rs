// src/guard/es.rs

use crate::error::ApiError;
use crate::guard::GuardEvent;
use tokio::sync::mpsc::Sender;
/// Subscribes to AUTH_* events from EndpointSecurity and sends GuardEvent.
pub async fn start_es_listener(tx: Sender<GuardEvent>) -> Result<(), ApiError> {
    // TODO: Subscribe to AUTH_UNLINK, AUTH_RENAME, AUTH_EXEC using ES new_client,
    // and send GuardEvent { pid, path } via tx.send().await when events occur.
    Ok(())
}
