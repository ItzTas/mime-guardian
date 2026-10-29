use std::path::PathBuf;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MimeApps {
    pub(super) lines: Vec<Line>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Line {
    pub raw: String,
    pub kind: Kind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Kind {
    Header(String),
    Entry { key: String, value: String },
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub path: PathBuf,
    pub writable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layer {
    pub source: Source,
    pub file: MimeApps,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    Default,
    Added,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Effective {
    pub app: String,
    pub path: PathBuf,
    pub origin: Origin,
}
