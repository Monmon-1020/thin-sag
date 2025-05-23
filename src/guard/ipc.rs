// src/guard/ipc.rs

use crate::guard::GuardEvent;

/// User's decision
pub enum UserDecision {
    Allow,
    Deny,
}

/// Query the Danger-Pop UI process for a user decision
pub async fn ask_user(event: &GuardEvent, rule_id: &str) -> UserDecision {
    // TODO: Send a JSON request via Unix domain socket and convert the response to UserDecision
    UserDecision::Deny
}
