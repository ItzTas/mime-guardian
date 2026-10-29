use std::path::PathBuf;

use crate::mimeapps::Effective;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Plan {
    pub files: Vec<FileEdit>,
    pub drifts: Vec<Drift>,
    pub skipped: Vec<Skipped>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEdit {
    pub path: PathBuf,
    pub before: Option<String>,
    pub after: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drift {
    pub mime: String,
    pub wanted: String,
    pub found: Option<Effective>,
    pub files: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skipped {
    pub mime: String,
    pub app: String,
}

impl Plan {
    pub fn is_clean(&self) -> bool {
        self.files.is_empty()
    }

    pub fn overridden(&self) -> impl Iterator<Item = &Drift> {
        self.drifts.iter().filter(|drift| drift.overridden())
    }

    pub fn unpinned(&self) -> impl Iterator<Item = &Drift> {
        self.drifts.iter().filter(|drift| !drift.overridden())
    }
}

impl Drift {
    pub fn overridden(&self) -> bool {
        self.found
            .as_ref()
            .is_none_or(|found| found.app != self.wanted)
    }

    pub fn previous(&self) -> Option<&str> {
        self.found.as_ref().map(|found| found.app.as_str())
    }
}
