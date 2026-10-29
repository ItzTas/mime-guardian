use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct App {
    pub path: PathBuf,
    pub hidden: bool,
    pub mime_types: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Installed {
    pub apps: BTreeMap<String, App>,
}

impl Installed {
    pub fn contains(&self, id: &str) -> bool {
        self.apps.get(id).is_some_and(|app| !app.hidden)
    }

    pub fn mime_types(&self) -> impl Iterator<Item = &str> {
        self.apps
            .values()
            .filter(|app| !app.hidden)
            .flat_map(|app| app.mime_types.iter().map(String::as_str))
    }
}
