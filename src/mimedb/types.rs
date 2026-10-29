use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MimeDb {
    pub types: BTreeSet<String>,
    pub aliases: HashMap<String, String>,
}
