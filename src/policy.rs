use anyhow::{anyhow, Result};
use globset::{Glob, GlobSetBuilder};
use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::{fs, path::PathBuf, sync::RwLock};

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

use crate::action::{Action, ActionList};

pub fn validate_actions(actions: &ActionList) -> Result<()> {
    let pol = load()?;
    let mut gb = GlobSetBuilder::new();
    for pat in &pol.denied_targets {
        gb.add(Glob::new(pat)?);
    }
    let denied_set = gb.build()?;

    for act in &actions.0 {
        let name = act_name(act);
        if !pol.allow_acts.iter().any(|s| s == name) {
            return Err(anyhow!("policy_violation: act `{}` not allowed", name));
        }
        if let Some(t) = act_target(act) {
            if denied_set.is_match(t) {
                return Err(anyhow!("policy_violation: target `{}` denied", t));
            }
        }
        if let Action::Wait { ms } = act {
            if let Some(max) = pol.max_wait_ms {
                if *ms > max {
                    return Err(anyhow!("policy_violation: wait {}ms exceeds {}", ms, max));
                }
            }
        }
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
