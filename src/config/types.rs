use std::collections::BTreeMap;

use serde::Deserialize;

use super::constants::{DEFAULT_ICON, DEFAULT_LINE, DEFAULT_SUMMARY};

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub defaults: BTreeMap<String, String>,
    #[serde(default)]
    pub guard: Guard,
    #[serde(default)]
    pub notification: Notification,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Guard {
    pub notify: bool,
    pub added_associations: bool,
    pub desktops: Vec<String>,
}

impl Default for Guard {
    fn default() -> Self {
        Self {
            notify: true,
            added_associations: true,
            desktops: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Notification {
    pub summary: String,
    pub line: String,
    pub icon: String,
    pub transient: bool,
    pub command: Option<String>,
}

impl Default for Notification {
    fn default() -> Self {
        Self {
            summary: DEFAULT_SUMMARY.to_string(),
            line: DEFAULT_LINE.to_string(),
            icon: DEFAULT_ICON.to_string(),
            transient: false,
            command: None,
        }
    }
}
