#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub mime: String,
    pub before: Option<String>,
    pub after: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Adoption {
    pub text: String,
    pub adopted: Vec<Change>,
    pub refused: Vec<(Change, String)>,
}

impl Adoption {
    pub fn is_empty(&self) -> bool {
        self.adopted.is_empty() && self.refused.is_empty()
    }
}
