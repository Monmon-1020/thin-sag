// MCP Proxy - 外部MCP透過転送
// 将来的に外部のMCPサーバーへのプロキシ機能を実装

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde_json::Value;
use std::sync::Arc;
use std::collections::HashMap;

use crate::error::ApiError;

#[derive(Clone)]
pub struct ProxyState {
    pub external_servers: Arc<HashMap<String, String>>, // server_name -> endpoint_url
}

/// GET /mcp/proxy/servers - 利用可能な外部MCPサーバー一覧
pub async fn list_external_servers_handler(
    State(state): State<Arc<ProxyState>>,
) -> Json<Value> {
    let servers: Vec<&String> = state.external_servers.keys().collect();
    Json(serde_json::json!({
        "external_servers": servers,
        "count": servers.len()
    }))
}

/// POST /mcp/proxy/{server_name}/tools/{tool_name}/call - 外部MCPサーバーのツール呼び出し
pub async fn proxy_tool_call_handler(
    State(_state): State<Arc<ProxyState>>,
    Path((server_name, tool_name)): Path<(String, String)>,
    Json(_params): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    // TODO: 実際の外部MCP呼び出し実装
    // HTTP クライアントを使用して外部MCPサーバーにリクエストを転送
    
    // 現在はプレースホルダー実装
    Ok(Json(serde_json::json!({
        "status": "not_implemented",
        "message": format!("Proxy call to {}/{} not yet implemented", server_name, tool_name),
        "server": server_name,
        "tool": tool_name
    })))
}

/// GET /mcp/proxy/{server_name}/tools - 外部MCPサーバーのツール一覧
pub async fn proxy_list_tools_handler(
    State(_state): State<Arc<ProxyState>>,
    Path(server_name): Path<String>,
) -> Result<Json<Value>, ApiError> {
    // TODO: 実際の外部MCP呼び出し実装
    
    // 現在はプレースホルダー実装
    Ok(Json(serde_json::json!({
        "status": "not_implemented",
        "message": format!("Tool listing for {} not yet implemented", server_name),
        "server": server_name,
        "tools": []
    })))
}

/// MCPプロキシルータを構築
pub fn build_proxy_router() -> Router {
    let external_servers = Arc::new(HashMap::new());
    let state = Arc::new(ProxyState { external_servers });

    Router::new()
        .route("/servers", get(list_external_servers_handler))
        .route("/:server_name/tools", get(proxy_list_tools_handler))
        .route("/:server_name/tools/:tool_name/call", post(proxy_tool_call_handler))
        .with_state(state)
}

// 将来の実装のためのヘルパー関数

/// 外部MCPサーバーを登録
pub async fn register_external_server(
    state: &Arc<ProxyState>,
    name: String,
    endpoint: String,
) -> Result<(), anyhow::Error> {
    // TODO: 実装
    // - 外部サーバーの可用性チェック
    // - 認証情報の管理
    // - ヘルスチェック機能
    
    Ok(())
}

/// 外部MCPサーバーへのHTTPリクエスト送信
async fn send_to_external_mcp(
    endpoint: &str,
    path: &str,
    method: &str,
    body: Option<Value>,
) -> Result<Value, anyhow::Error> {
    // TODO: reqwest または他のHTTPクライアントを使用した実装
    // - タイムアウト設定
    // - エラーハンドリング
    // - レスポンス変換
    
    Err(anyhow::anyhow!("External MCP communication not implemented"))
}
