use axum::{
    routing::post,
    extract::State,
    Json, Router,
};
use crate::{vault, ui_adapter};
use crate::models::{RunRequest, RunResponse};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    // 共有キャッシュ・設定を持たせたい場合に使う
}

pub async fn run_handler(
    State(_st): State<Arc<AppState>>,
    Json(req): Json<RunRequest>,
) -> Json<RunResponse> {
    // 1) Keychain から値を取る
    let secret = match vault::get_secret(&req.secret) {
        Ok(s) => s,
        Err(e) => return Json(RunResponse::fail(e)),
    };
    // 2) テキスト置換
    let text = req.text.replace("{secret}", &secret);

    // 3) アプリ起動
    if let Err(e) = ui_adapter::launch_app(&req.bundle) {
        return Json(RunResponse::fail(e));
    }

    // 4) 入力
    if let Err(e) = ui_adapter::type_text(&text) {
        return Json(RunResponse::fail(e));
    }

    Json(RunResponse::success())
}

/// axum ルータ生成
pub fn build_router() -> Router {
    let state = Arc::new(AppState {});
    Router::new()
        .route("/run", post(run_handler))
        .with_state(state)
}