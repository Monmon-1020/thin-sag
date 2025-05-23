use crate::guard::GuardEvent;
use regex::Regex;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DangerRule {
    pub id: String,
    #[serde(default, with = "serde_regex")]
    pub path_regex: Option<Regex>,
}

impl DangerRule {
    /// Returns true if the event matches this rule
    pub fn matches(&self, event: &GuardEvent) -> bool {
        if let Some(re) = &self.path_regex {
            return re.is_match(&event.path);
        }
        false
    }
}

/// Loads the danger_rules section from policy.yaml
pub fn load_rules() -> Vec<DangerRule> {
    let path = dirs::home_dir().unwrap().join(".thin-sag/policy.yaml");
    let yml = std::fs::read_to_string(path).expect("failed to read policy.yaml");
    let doc: serde_yaml::Value = serde_yaml::from_str(&yml).expect("invalid YAML");
    doc["danger_rules"]
        .as_sequence()
        .unwrap()
        .iter()
        .map(|v| serde_yaml::from_value(v.clone()).unwrap())
        .collect()
}
