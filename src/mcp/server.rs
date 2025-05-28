// MCP Server ルータ
// /mcp エンドポイントの実装

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::ApiError;
use crate::api::AppState;

/// GET /mcp/tools - 利用可能なツール一覧を返す
pub async fn list_tools_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, ApiError> {
    let tools = state.tool_catalog.list_tools().await;
    Ok(Json(json!({
        "tools": tools,
        "version": "1.0.0"
    })))
}

/// POST /mcp/tools/{tool_name}/call - 指定されたツールを実行
pub async fn call_tool_handler(
    State(state): State<Arc<AppState>>,
    Path(tool_name): Path<String>,
    Json(params): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let result = state.tool_catalog.call_tool(&tool_name, params).await
        .map_err(|e| ApiError::Internal(e))?;
    
    Ok(Json(result))
}

/// GET /mcp/status - MCP サーバーの状態を返す
pub async fn status_handler() -> Json<Value> {
    Json(json!({
        "status": "active",
        "protocol": "mcp",
        "version": "1.0.0",
        "capabilities": [
            "tools",
            "resources"
        ]
    }))
}

/// MCPルータを構築
pub fn build_mcp_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/tools", get(list_tools_handler))
        .route("/tools/:tool_name/call", post(call_tool_handler))
        .route("/status", get(status_handler))
}
