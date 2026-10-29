use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub defaults: BTreeMap<String, String>,
    #[serde(default)]
    pub guard: Guard,
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
