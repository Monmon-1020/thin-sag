// src/screenshot.rs

use crate::error::ApiError;
use anyhow::anyhow;
use axum::body::Body;
use axum::{
    body::Bytes,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use std::{fs, process::Command};
use uuid::Uuid;

/// GET /screenshot
/// 画面全体をキャプチャして image/png として返します。
pub async fn screenshot_handler() -> Result<impl IntoResponse, ApiError> {
    // 1) 一時ファイルパスを生成
    let tmp_path = std::env::temp_dir().join(format!("thin_sag_screenshot_{}.png", Uuid::new_v4()));
    eprintln!("DEBUG: screenshot tmp path = {:?}", tmp_path);

    // 2) screencapture 実行
    let output = Command::new("screencapture")
        .arg("-x") // シャッター音オフ
        .arg(&tmp_path) // 出力先ファイル
        .output()
        .map_err(|e| ApiError::Internal(anyhow!("failed to spawn screencapture: {}", e)))?;

    // 3) デバッグ出力
    eprintln!("DEBUG: screencapture status = {:?}", output.status);
    eprintln!(
        "DEBUG: screencapture stderr = {}",
        String::from_utf8_lossy(&output.stderr)
    );

    if !output.status.success() {
        return Err(ApiError::Internal(anyhow!(
            "screencapture returned error status: {:?}",
            output.status.code()
        )));
    }

    // 4) 一時ファイルを読み込み
    let image_data = fs::read(&tmp_path)
        .map_err(|e| ApiError::Internal(anyhow!("failed to read screenshot file: {}", e)))?;

    // 5) 一時ファイルを削除
    if let Err(e) = fs::remove_file(&tmp_path) {
        eprintln!("WARNING: failed to remove temp file {:?}: {}", tmp_path, e);
    }

    // 6) レスポンス組み立て
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/png")
        .body(Bytes::from(image_data))
        .map_err(|e| ApiError::Internal(anyhow!("failed to build response: {}", e)))?;

    Ok(response.map(axum::body::Body::from))
}
