use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Backup {
    pub id: String,
    pub dir: PathBuf,
    pub saved: Vec<String>,
    pub created: Vec<String>,
}
