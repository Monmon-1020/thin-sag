use crate::policy::validate_actions;
use crate::{action::ActionList, error::ApiError, job::JobManager, models::*, tree::UiNode};
use crate::{policy::load as load_policy, tree::snapshot_tree};
use axum::http::StatusCode;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::time::{Duration, Instant};

static SNAP_COUNT: Lazy<AtomicU32> = Lazy::new(|| AtomicU32::new(0));
static SNAP_RESET: Lazy<tokio::sync::Mutex<Instant>> =
    Lazy::new(|| tokio::sync::Mutex::new(Instant::now()));
#[derive(Clone)]
pub struct AppState {
    job_manager: Arc<JobManager>,
}

pub async fn snapshot_handler() -> Result<Json<UiNode>, ApiError> {
    // ポリシー
    println!("[DEBUG] /snapshot called");
    let pol = load_policy().map_err(ApiError::Internal)?;
    if !pol.allow_snapshot {
        return Err(ApiError::BadRequest(anyhow::anyhow!("snapshot disabled")));
    }
    // rate‑limit
    {
        let mut guard = SNAP_RESET.lock().await;
        if guard.elapsed() > Duration::from_secs(60) {
            SNAP_COUNT.store(0, Ordering::SeqCst);
            *guard = Instant::now();
        }
    }
    if SNAP_COUNT.fetch_add(1, Ordering::SeqCst) >= pol.max_snapshot_per_min.unwrap_or(10) {
        return Err(ApiError::BadRequest(anyhow::anyhow!("rate limit")));
    }
    let tree = snapshot_tree().map_err(ApiError::Internal)?;
    Ok(Json(tree))
}

pub async fn run_handler(
    State(st): State<Arc<AppState>>,
    Json(req): Json<RunRequest>,
) -> Result<(axum::http::StatusCode, Json<serde_json::Value>), ApiError> {
    let id = st.job_manager.enqueue(req).await;
    Ok((
        axum::http::StatusCode::ACCEPTED,
        Json(serde_json::json!({"job_id": id})),
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
        None => Err(ApiError::NotFound(anyhow::anyhow!("Job ID 不明"))),
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
    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({"job_id": id})),
    ))
}

pub fn build_router() -> Router {
    let state = Arc::new(AppState {
        job_manager: Arc::new(JobManager::new()),
    });
    Router::new()
        .route("/run", post(run_handler))
        .route("/job/:id", get(job_status))
        .route("/run-json", post(run_json))
        .route("/snapshot", get(snapshot_handler))
        .with_state(state)
}
