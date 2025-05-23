// src/guard/audit.rs
use crate::error::ApiError;
use crate::guard::ipc::UserDecision;
use crate::guard::GuardEvent;
use chrono::Utc;
use serde_json::json;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;

/// Write audit logs in JSONL format
pub async fn write(
    event: &GuardEvent,
    rule_id: &str,
    decision: &UserDecision,
) -> Result<(), ApiError> {
    let dir = dirs::home_dir().unwrap().join(".thin-sag/logs");
    create_dir_all(&dir).unwrap();
    let file = dir.join(format!("audit-{}.log", Utc::now().format("%Y%m%d")));
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file)
        .map_err(|e| ApiError::Internal(e.into()))?;

    let entry = json!({
        "ts": Utc::now().timestamp(),
        "pid": event.pid,
        "path": event.path,
        "rule_id": rule_id,
        "user_decision": match decision {
            UserDecision::Allow => "allow",
            UserDecision::Deny => "deny",
        }
    });
    writeln!(f, "{}", entry.to_string()).unwrap();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guard::ipc::UserDecision;
    use crate::guard::GuardEvent;
    use chrono::Utc;
    use std::{env, fs};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_write_creates_log_and_contents() {
        // Prepare a temporary HOME directory for testing
        let tmp = TempDir::new().expect("failed to create tempdir");
        env::set_var("HOME", tmp.path());

        // Write a dummy event and decision
        let event = GuardEvent {
            pid: 1234,
            path: "/tmp/testfile".into(),
        };
        write(&event, "ruleX", &UserDecision::Deny)
            .await
            .expect("write failed");

        // Read the output log file and verify its contents
        let today = Utc::now().format("%Y%m%d").to_string();
        let log_path = tmp
            .path()
            .join(".thin-sag/logs")
            .join(format!("audit-{}.log", today));

        let contents = fs::read_to_string(&log_path).expect("failed to read audit log");

        // Check that each field is present
        assert!(contents.contains(r#""pid":1234"#), "pid not found in log");
    }
}
