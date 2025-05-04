use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RunRequest {
    pub bundle: String,   // 例: "com.apple.Notes"
    pub secret: String,   // Keychain ラベル
    pub text:   String,   // "Hello {secret}!"
}

#[derive(Serialize)]
pub struct RunResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl RunResponse {
    pub fn success() -> Self { Self { ok: true,  error: None } }
    pub fn fail<E: ToString>(e: E) -> Self { Self { ok: false, error: Some(e.to_string()) } }
}