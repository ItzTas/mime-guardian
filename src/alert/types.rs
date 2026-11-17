#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alert {
    pub summary: String,
    pub body: String,
    pub count: usize,
}
