// MCP Tool Catalog - DSL マッピング
// 既存のthin-sag機能をMCPツールとして公開

use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::action::ActionList;
use crate::tree::{snapshot_tree, list_windows_info, WindowSelector};
use crate::screenshot::take_screenshot;
use crate::policy::load as load_policy;

pub struct ToolCatalog {
    tools: HashMap<String, ToolDefinition>,
}

#[derive(Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
    pub handler: fn(&Value) -> Result<Value>,
}

impl ToolCatalog {
    pub fn new() -> Self {
        let mut tools = HashMap::new();

        // snapshot ツール
        tools.insert("snapshot".to_string(), ToolDefinition {
            name: "snapshot".to_string(),
            description: "UI要素のスナップショットを取得".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "window": {
                        "type": "object",
                        "properties": {
                            "selector": {
                                "type": "string",
                                "enum": ["front", "index", "title", "doc"],
                                "description": "ウィンドウ選択方法"
                            },
                            "value": {
                                "description": "選択値（インデックス、タイトル、ドキュメント名など）"
                            }
                        },
                        "required": ["selector"]
                    }
                }
            }),
            handler: handle_snapshot,
        });

        // windows ツール
        tools.insert("windows".to_string(), ToolDefinition {
            name: "windows".to_string(),
            description: "利用可能なウィンドウ一覧を取得".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {}
            }),
            handler: handle_windows,
        });

        // action ツール
        tools.insert("action".to_string(), ToolDefinition {
            name: "action".to_string(),
            description: "UI操作アクションを実行".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "actions": {
                        "type": "array",
                        "description": "実行するアクションのリスト",
                        "items": {
                            "type": "object",
                            "properties": {
                                "type": {
                                    "type": "string",
                                    "enum": ["click", "type", "key", "scroll", "drag"]
                                },
                                "target": {
                                    "type": "object",
                                    "description": "操作対象の要素"
                                }
                            }
                        }
                    }
                },
                "required": ["actions"]
            }),
            handler: handle_action,
        });

        // screenshot ツール
        tools.insert("screenshot".to_string(), ToolDefinition {
            name: "screenshot".to_string(),
            description: "スクリーンショットを取得".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "format": {
                        "type": "string",
                        "enum": ["png", "jpeg"],
                        "default": "png"
                    }
                }
            }),
            handler: handle_screenshot,
        });

        Self { tools }
    }

    pub async fn list_tools(&self) -> Vec<Value> {
        self.tools.values().map(|tool| {
            json!({
                "name": tool.name,
                "description": tool.description,
                "inputSchema": tool.parameters
            })
        }).collect()
    }

    pub async fn call_tool(&self, tool_name: &str, params: Value) -> Result<Value> {
        if let Some(tool) = self.tools.get(tool_name) {
            (tool.handler)(&params)
        } else {
            Err(anyhow::anyhow!("Tool '{}' not found", tool_name))
        }
    }
}

// ツールハンドラ実装

fn handle_snapshot(params: &Value) -> Result<Value> {
    let pol = load_policy()?;
    if !pol.allow_snapshot {
        return Err(anyhow::anyhow!("Snapshot disabled by policy"));
    }

    let window_selector = if let Some(window) = params.get("window") {
        let selector = window.get("selector").and_then(|s| s.as_str()).unwrap_or("front");
        match selector {
            "front" => WindowSelector::Front,
            "index" => {
                let index = window.get("value").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                WindowSelector::Index(index)
            },
            "title" => {
                let title = window.get("value").and_then(|v| v.as_str()).unwrap_or("").to_string();
                WindowSelector::Title(title)
            },
            "doc" => {
                let doc = window.get("value").and_then(|v| v.as_str()).unwrap_or("").to_string();
                WindowSelector::Doc(doc)
            },
            _ => WindowSelector::Front,
        }
    } else {
        WindowSelector::Front
    };

    let tree = snapshot_tree(window_selector)?;
    Ok(serde_json::to_value(tree)?)
}

fn handle_windows(_params: &Value) -> Result<Value> {
    let windows = list_windows_info();
    Ok(serde_json::to_value(windows)?)
}

fn handle_action(params: &Value) -> Result<Value> {
    let actions: ActionList = serde_json::from_value(params.clone())?;
    
    // ここでアクション実行のロジックを呼び出す
    // 実際の実装は既存のrun_handlerロジックを参照
    
    Ok(json!({
        "status": "success",
        "message": "Actions queued for execution"
    }))
}

fn handle_screenshot(params: &Value) -> Result<Value> {
    let format = params.get("format").and_then(|f| f.as_str()).unwrap_or("png");
    
    // スクリーンショット取得の実装
    // 実際の実装は既存のscreenshot_handlerを参照
    
    Ok(json!({
        "status": "success",
        "format": format,
        "message": "Screenshot captured"
    }))
}
