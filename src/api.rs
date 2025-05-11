use crate::policy::validate_actions;
use crate::tree::WindowSelector;
use crate::tree::{list_windows_info, WindowInfo};
use crate::{action::ActionList, error::ApiError, job::JobManager, models::*, tree::UiNode};
use crate::{policy::load as load_policy, tree::snapshot_tree};
use axum::http::StatusCode;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use once_cell::sync::Lazy;
use serde::Deserialize;
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

#[derive(Deserialize)]
#[serde(untagged)]
enum WindowParam {
    Front(String),
    Obj { window: WindowArg },
}
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum WindowArg {
    Index(usize),
    Title(String),
    Doc(String),
}

pub async fn snapshot_handler(Json(body): Json<WindowParam>) -> Result<Json<UiNode>, ApiError> {
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

    let sel = match body {
        WindowParam::Front(s) => {
            println!("[DEBUG] Front window parameter: {}", s);
            WindowSelector::Front
        }
        WindowParam::Obj {
            window: WindowArg::Index(i),
        } => WindowSelector::Index(i),
        WindowParam::Obj {
            window: WindowArg::Title(t),
        } => WindowSelector::Title(t),
        WindowParam::Obj {
            window: WindowArg::Doc(d),
        } => WindowSelector::Doc(d),
    };
    // validate_snapshot_policy()?;
    let tree = snapshot_tree(sel).map_err(ApiError::BadRequest)?;
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

/// GET /windows
pub async fn windows_handler() -> Json<Vec<WindowInfo>> {
    eprintln!("[DEBUG] /windows called");
    let list = list_windows_info();
    Json(list)
}

pub fn build_router() -> Router {
    let state = Arc::new(AppState {
        job_manager: Arc::new(JobManager::new()),
    });
    Router::new()
        .route("/run", post(run_handler))
        .route("/job/:id", get(job_status))
        .route("/run-json", post(run_json))
        .route("/snapshot", post(snapshot_handler))
        .route("/windows", get(windows_handler))
        .with_state(state)
}
