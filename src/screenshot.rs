// src/screenshot.rs

use crate::error::ApiError;
use anyhow::anyhow;
use axum::{
    extract::Query,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use base64;
use serde::{Deserialize, Serialize};
use std::{fs, process::Command};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ScreenshotParams {
    output: Option<String>,
}

#[derive(Serialize)]
struct Base64Response {
    data: String,
}

pub async fn screenshot_handler(
    Query(params): Query<ScreenshotParams>,
) -> Result<impl IntoResponse, ApiError> {
    let tmp_path = std::env::temp_dir().join(format!("thin_sag_screenshot_{}.png", Uuid::new_v4()));
    eprintln!("DEBUG: tmp_path = {:?}", tmp_path);

    let output = Command::new("screencapture")
        .arg("-x")
        .arg(&tmp_path)
        .output()
        .map_err(|e| ApiError::Internal(anyhow!("failed to spawn screencapture: {}", e)))?;
    eprintln!("DEBUG: status = {:?}", output.status);
    if !output.status.success() {
        let code = output.status.code();
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ApiError::Internal(anyhow!(
            "screencapture failed (code {:?}): {}",
            code,
            stderr
        )));
    }

    let image_data = fs::read(&tmp_path)
        .map_err(|e| ApiError::Internal(anyhow!("failed to read screenshot: {}", e)))?;

    if let Err(e) = fs::remove_file(&tmp_path) {
        eprintln!("WARNING: remove_file {:?}: {}", tmp_path, e);
    }

    if let Some(dest) = params.output {
        fs::write(&dest, &image_data)
            .map_err(|e| ApiError::Internal(anyhow!("failed to write to {}: {}", dest, e)))?;
        let body = format!("Saved to {}", dest);
        let resp = Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(body.into())
            .map_err(|e| ApiError::Internal(anyhow!("response build error: {}", e)))?;
        Ok(resp)
    } else {
        let b64 = base64::encode(&image_data);
        let resp = Json(Base64Response { data: b64 });
        Ok((StatusCode::OK, resp).into_response())
    }
}
