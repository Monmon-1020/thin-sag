use axum::{routing::{post,get}, Json, Router, extract::{State, Path}};
use crate::{models::*, job::JobManager, error::ApiError, action::ActionList};
use std::sync::Arc;
use axum::http::StatusCode; 
use crate::policy::validate_actions;
#[derive(Clone)]
pub struct AppState {
    job_manager: Arc<JobManager>,
}

pub async fn run_handler(
    State(st): State<Arc<AppState>>,
    Json(req): Json<RunRequest>,
) -> Result<(axum::http::StatusCode, Json<serde_json::Value>), ApiError> {
    let id = st.job_manager.enqueue(req).await;
    Ok((
        axum::http::StatusCode::ACCEPTED,
        Json(serde_json::json!({"job_id": id}))
    ))
}

/// GET /job/{id}
pub async fn job_status(
    State(st): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    match st.job_manager.get(&id).await {
        Some(r) => Ok(Json(serde_json::json!({
            "status": format!("{:?}", r.status),
            "result": r.output
        }))),
        None => Err(ApiError::NotFound(anyhow::anyhow!("Job ID 不明")))
    }
}

pub async fn run_json(
    State(st): State<Arc<AppState>>,
    Json(list): Json<ActionList>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    // 🔒 ポリシー検証
    if let Err(e) = validate_actions(&list) {
        return Err(ApiError::BadRequest(e));
    }
    // 実行リクエストをキューへ
    let id = st.job_manager.enqueue_json(list).await;
    Ok((StatusCode::ACCEPTED, Json(serde_json::json!({"job_id": id}))))
}

pub fn build_router() -> Router {
    let state = Arc::new(AppState{ job_manager: Arc::new(JobManager::new()) });
    Router::new()
        .route("/run", post(run_handler))
        .route("/job/:id", get(job_status))
        .route("/run-json", post(run_json))
        .with_state(state)
}
