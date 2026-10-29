use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Trigger {
    Apply,
    Watch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub at: u64,
    pub trigger: Trigger,
    pub mime: String,
    pub from: Option<String>,
    pub to: String,
    pub overridden: bool,
    pub files: Vec<String>,
    pub backup: Option<String>,
}
