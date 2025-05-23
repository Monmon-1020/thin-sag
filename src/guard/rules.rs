// src/guard/rules.rs

use crate::guard::GuardEvent;
use regex::Regex;
use serde::Deserialize;

/// 危険ルール（path, host, exec のいずれかにマッチすると警告を出す）
#[derive(Debug, Deserialize)]
pub struct DangerRule {
    pub id: String,
    /// ファイルパスやURL全体に対してマッチ
    #[serde(default, with = "serde_regex")]
    pub path_regex: Option<Regex>,
    /// HTTPホスト名や接続先ドメインに対してマッチ
    #[serde(default, with = "serde_regex")]
    pub host_regex: Option<Regex>,
    /// 実行コマンド文字列に対してマッチ
    #[serde(default, with = "serde_regex")]
    pub exec_regex: Option<Regex>,
}

impl DangerRule {
    /// GuardEvent の path に対していずれかの正規表現がマッチするか
    pub fn matches(&self, event: &GuardEvent) -> bool {
        if let Some(re) = &self.path_regex {
            if re.is_match(&event.path) {
                return true;
            }
        }
        if let Some(re) = &self.host_regex {
            if re.is_match(&event.path) {
                return true;
            }
        }
        if let Some(re) = &self.exec_regex {
            if re.is_match(&event.path) {
                return true;
            }
        }
        false
    }
}

/// `~/.thin-sag/policy.yaml` の `danger_rules` をロードする
pub fn load_rules() -> Vec<DangerRule> {
    let path = dirs::home_dir()
        .expect("home_dir not found")
        .join(".thin-sag/policy.yaml");
    let yml = std::fs::read_to_string(path).expect("failed to read ~/.thin-sag/policy.yaml");
    let doc: serde_yaml::Value = serde_yaml::from_str(&yml).expect("invalid YAML in policy.yaml");

    doc["danger_rules"]
        .as_sequence()
        .expect("`danger_rules` must be a YAML sequence")
        .iter()
        .map(|v| serde_yaml::from_value(v.clone()).expect("invalid DangerRule entry"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guard::GuardEvent;

    #[test]
    fn test_path_regex_match() {
        let rule = DangerRule {
            id: "r1".into(),
            path_regex: Some(Regex::new(r"^/tmp/.*\.txt$").unwrap()),
            host_regex: None,
            exec_regex: None,
        };
        let ev_ok = GuardEvent {
            pid: 1,
            path: "/tmp/secret.txt".into(),
        };
        let ev_ng = GuardEvent {
            pid: 1,
            path: "/tmp/secret.log".into(),
        };
        assert!(rule.matches(&ev_ok));
        assert!(!rule.matches(&ev_ng));
    }

    #[test]
    fn test_host_regex_match() {
        let rule = DangerRule {
            id: "r2".into(),
            path_regex: None,
            host_regex: Some(Regex::new(r"^.*\.example\.com$").unwrap()),
            exec_regex: None,
        };
        // HTTPリクエストなど host 部分が event.path に含まれている想定
        let ev = GuardEvent {
            pid: 2,
            path: "api.example.com".into(),
        };
        assert!(rule.matches(&ev));
    }

    #[test]
    fn test_exec_regex_match() {
        let rule = DangerRule {
            id: "r3".into(),
            path_regex: None,
            host_regex: None,
            exec_regex: Some(Regex::new(r"^rm -rf ").unwrap()),
        };
        let ev = GuardEvent {
            pid: 3,
            path: "rm -rf /Users/alice".into(),
        };
        assert!(rule.matches(&ev));
    }

    #[test]
    fn test_no_regex() {
        let rule = DangerRule {
            id: "r0".into(),
            path_regex: None,
            host_regex: None,
            exec_regex: None,
        };
        let ev = GuardEvent {
            pid: 0,
            path: "/any".into(),
        };
        assert!(!rule.matches(&ev));
    }
}
