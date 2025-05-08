use anyhow::{anyhow, Result};
use globset::{Glob, GlobSetBuilder};
use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::{fs, path::PathBuf, sync::RwLock};

/// ~/.thin-sag/policy.yaml を読み込む
const DEFAULT_PATH: &str = ".thin-sag/policy.yaml";

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Policy {
    pub allow_acts: Vec<String>,
    pub denied_targets: Vec<String>,
    pub max_wait_ms: Option<u64>,
    pub click_bounds: Option<Bounds>,
    pub allow_snapshot: bool,
    pub max_snapshot_per_min: Option<u32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Bounds {
    pub x_min: i32,
    pub x_max: i32,
    pub y_min: i32,
    pub y_max: i32,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            allow_acts: vec![
                "launch".into(),
                "type".into(),
                "wait".into(),
                "click".into(),
                "scroll".into(),
                "keypress".into(),
            ],
            denied_targets: vec![],
            max_wait_ms: Some(30_000),
            click_bounds: None,
            allow_snapshot: true,
            max_snapshot_per_min: Some(10),
        }
    }
}

/// シングルトンキャッシュ
static POLICY: OnceCell<RwLock<Policy>> = OnceCell::new();

pub(crate) fn load() -> Result<std::sync::Arc<Policy>> {
    POLICY
        .get_or_try_init(|| {
            let path = dirs::home_dir()
                .unwrap_or(PathBuf::from("/"))
                .join(DEFAULT_PATH);
            let pol: Policy = if path.exists() {
                let txt = fs::read_to_string(&path)?;
                serde_yaml::from_str(&txt)?
            } else {
                Policy::default()
            };
            Ok(RwLock::new(pol))
        })
        .map(|lock| std::sync::Arc::new(lock.read().unwrap().clone()))
}

/// ------------- 検証関数 (public) -----------------
use crate::action::{Action, ActionList};

/// Action 配列をポリシーで検証
pub fn validate_actions(actions: &ActionList) -> Result<()> {
    let pol = load()?;
    // denied_targets 用 Glob セットを作成
    let mut gb = GlobSetBuilder::new();
    for pat in &pol.denied_targets {
        gb.add(Glob::new(pat)?);
    }
    let denied_set = gb.build()?;

    for act in &actions.0 {
        // 1) allow_acts
        let name = act_name(act);
        if !pol.allow_acts.iter().any(|s| s == name) {
            return Err(anyhow!("policy_violation: act `{}` not allowed", name));
        }
        // 2) denied_targets
        if let Some(t) = act_target(act) {
            if denied_set.is_match(t) {
                return Err(anyhow!("policy_violation: target `{}` denied", t));
            }
        }
        // 3) wait 超過
        if let Action::Wait { ms } = act {
            if let Some(max) = pol.max_wait_ms {
                if *ms > max {
                    return Err(anyhow!("policy_violation: wait {}ms exceeds {}", ms, max));
                }
            }
        }
        // 4) クリック座標
        if let (
            Some(b),
            Action::Click {
                x: Some(px),
                y: Some(py),
                ..
            },
        ) = (pol.click_bounds.as_ref(), act)
        {
            if *px < b.x_min || *px > b.x_max || *py < b.y_min || *py > b.y_max {
                return Err(anyhow!(
                    "policy_violation: click ({},{}) out of bounds",
                    px,
                    py
                ));
            }
        }
    }
    Ok(())
}

/// helper: act 名
fn act_name(a: &Action) -> &'static str {
    match a {
        Action::Launch { .. } => "launch",
        Action::Type { .. } => "type",
        Action::Wait { .. } => "wait",
        Action::Click { .. } => "click",
        Action::Scroll { .. } => "scroll",
        Action::Keypress { .. } => "keypress",
        Action::Unsupported => "unsupported",
    }
}
/// helper: 対象文字列 (launch target / click selector 等)
fn act_target(a: &Action) -> Option<&str> {
    match a {
        Action::Launch { target } => Some(target),
        Action::Click {
            selector: Some(sel),
            ..
        } => Some(sel),
        _ => None,
    }
}
