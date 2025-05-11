use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RunRequest {
    pub bundle: String, // ex: "com.apple.Notes"
    pub secret: String,
    pub text: String,
}

#[derive(Debug, serde::Serialize)]
pub struct RunResponse {
    pub success: bool,
    pub message: Option<String>,
}

impl RunResponse {
    pub fn success() -> Self {
        Self {
            success: true,
            message: None,
        }
    }

    pub fn fail(message: String) -> Self {
        Self {
            success: false,
            message: Some(message),
        }
    }
}
