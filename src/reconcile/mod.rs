mod apply;
mod constants;
mod drift;
mod types;

pub use apply::{apply, target};
pub use drift::plan;
pub use types::{Drift, FileEdit, Outcome, Plan, Skipped};
