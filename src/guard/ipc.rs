// src/guard/ipc.rs
use crate::guard::GuardEvent;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::UnixStream,
    time::timeout,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserDecision {
    Allow,
    Deny,
}

#[derive(Serialize)]
struct IpcRequest {
    rule_id: String,
    path: String,
    pid: i32,
}

#[derive(Deserialize)]
struct IpcResponse {
    decision: String,
    cache_min: Option<u64>,
}

// Get socket path (can be overridden by environment variable)
fn socket_path() -> PathBuf {
    if let Ok(p) = std::env::var("SAG_DANGER_SOCKET") {
        PathBuf::from(p)
    } else {
        dirs::home_dir()
            .expect("home_dir not found")
            .join(".thin-sag/danger.sock")
    }
}

// Query the Danger-Pop UI process and get the user's decision.
// Timeout is 30 seconds. Returns Deny on error or timeout.
pub async fn ask_user(event: &GuardEvent, rule_id: &str) -> UserDecision {
    let result = timeout(Duration::from_secs(30), async move {
        let path = socket_path();
        let mut stream = UnixStream::connect(&path)
            .await
            .map_err(|e| Box::new(e) as Box<_>)?;
        let req = IpcRequest {
            rule_id: rule_id.to_string(),
            path: event.path.clone(),
            pid: event.pid,
        };
        let req_json = serde_json::to_string(&req)?;
        stream
            .write_all(req_json.as_bytes())
            .await
            .map_err(|e| Box::new(e) as Box<_>)?;
        stream
            .write_all(b"\n")
            .await
            .map_err(|e| Box::new(e) as Box<_>)?;

        let reader = BufReader::new(stream);
        let mut lines = reader.lines();
        if let Some(line) = lines.next_line().await.map_err(|e| Box::new(e) as Box<_>)? {
            let resp: IpcResponse = serde_json::from_str(&line)?;
            if resp.decision.eq_ignore_ascii_case("allow") {
                Ok::<UserDecision, Box<dyn std::error::Error + Send + Sync>>(UserDecision::Allow)
            } else {
                Ok::<UserDecision, Box<dyn std::error::Error + Send + Sync>>(UserDecision::Deny)
            }
        } else {
            Ok::<UserDecision, Box<dyn std::error::Error + Send + Sync>>(UserDecision::Deny)
        }
    })
    .await;
    match result {
        Ok(Ok(decision)) => decision,
        Ok(Err(e)) => {
            eprintln!("[ipc] error communicating with UI: {}", e);
            UserDecision::Deny
        }
        Err(_) => {
            eprintln!("[ipc] ask_user timed out");
            UserDecision::Deny
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;
    use tokio::net::UnixListener;
    use tokio::task;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_ask_user_allow_and_deny() {
        let tmp = TempDir::new().unwrap();
        let sock_path = tmp.path().join("test.sock");
        env::set_var("SAG_DANGER_SOCKET", &sock_path);

        let listener = UnixListener::bind(&sock_path).unwrap();
        let server = task::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                reader.read_line(&mut line).await.unwrap();
                let mut writer = reader.into_inner();
                let resp = r#"{"decision":"allow","cache_min":60}"#;
                writer.write_all(resp.as_bytes()).await.unwrap();
                writer.write_all(b"\n").await.unwrap();
            }
        });

        sleep(Duration::from_millis(10)).await;
        let event = GuardEvent {
            pid: 99,
            path: "/tmp/x".into(),
        };
        let decision = ask_user(&event, "rule-test").await;
        assert_eq!(decision, UserDecision::Allow);

        server.abort();
    }

    #[tokio::test]
    async fn test_ask_user_timeout() {
        env::set_var("SAG_DANGER_SOCKET", "/nonexistent/socket");
        let event = GuardEvent {
            pid: 1,
            path: "/".into(),
        };
        let decision = ask_user(&event, "r0").await;
        assert_eq!(decision, UserDecision::Deny);
    }
}
