use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RunRequest {
    pub bundle: String,   // 例: "com.apple.Notes"
    pub secret: String,   // Keychain ラベル
    pub text:   String,   // "Hello {secret}!"
}

#[derive(Debug, serde::Serialize)]
pub struct RunResponse {
    pub success: bool,
    pub message: Option<String>,
}

impl RunResponse {
    pub fn success() -> Self {
        Self { success: true, message: None }
    }

    pub fn fail(message: String) -> Self {
        Self { success: false, message: Some(message) }
    }
}