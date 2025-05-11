use serde::Deserialize;
#[derive(Deserialize, Debug)]
pub struct ActionList(pub Vec<Action>);

#[derive(Deserialize, Debug)]
#[serde(tag = "act")]
pub enum Action {
    #[serde(rename = "launch")]
    Launch { target: String },

    #[serde(rename = "type")]
    Type { text: String },

    #[serde(rename = "wait")]
    Wait { ms: u64 },

    #[serde(rename = "click")]
    Click {
        selector: Option<String>,
        x: Option<i32>,
        y: Option<i32>,
    },

    #[serde(rename = "scroll")]
    Scroll { dy: i32 },

    #[serde(rename = "keypress")]
    Keypress { key: String },

    #[serde(other)]
    Unsupported,
}
