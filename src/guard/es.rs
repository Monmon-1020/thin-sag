// src/guard/es.rs

use crate::error::ApiError;
use crate::guard::GuardEvent;
use anyhow::Result;
use endpoint_sec::{sys, version, Client, Message};
use std::thread;
use tokio::sync::mpsc::Sender;

/// Asynchronous entry point called by Tokio.
/// Runs the ES subscription loop in a separate thread.
pub fn start_es_listener(tx: Sender<GuardEvent>) -> Result<(), ApiError> {
    es_loop_blocking(tx)
}

fn es_loop_blocking(tx: Sender<GuardEvent>) -> Result<(), ApiError> {
    // Set macOS runtime version (e.g., 10.15.0)
    eprintln!("[guard][ES] entering es_loop_blocking");
    version::set_runtime_version(10, 15, 0);

    // Create a client
    let mut client = Client::new(move |client, msg: Message| {
        eprintln!("[guard] ES callback: event_type={:?}", msg.event_type());
        use sys::es_event_type_t as T;
        match msg.event_type() {
            T::ES_EVENT_TYPE_AUTH_UNLINK
            | T::ES_EVENT_TYPE_AUTH_RENAME
            | T::ES_EVENT_TYPE_AUTH_EXEC
            | T::ES_EVENT_TYPE_AUTH_OPEN => {
                eprintln!("[guard][ES] matched AUTH event");
                let ev = GuardEvent::from(&msg);
                eprintln!("[guard][ES] GuardEvent constructed: {:?}", ev);
                let _ = tx.send(ev);
            }
            _ => {}
        }
    })?;
    eprintln!("[guard][ES] subscribing to AUTH events");
    // Subscribe to events
    client.subscribe(&[
        sys::es_event_type_t::ES_EVENT_TYPE_AUTH_UNLINK,
        sys::es_event_type_t::ES_EVENT_TYPE_AUTH_RENAME,
        sys::es_event_type_t::ES_EVENT_TYPE_AUTH_EXEC,
        sys::es_event_type_t::ES_EVENT_TYPE_AUTH_OPEN,
    ])?;

    // Keep the thread alive
    thread::park();
    Ok(())
}
