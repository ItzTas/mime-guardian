mod atomic;
mod fs;

pub use atomic::{replace_contents, replace_file};
pub use fs::{create_parent, metadata, write_contents};
