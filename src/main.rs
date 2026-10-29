use std::process::ExitCode;

use mime_guardian::{cli, output};

fn main() -> ExitCode {
    match cli::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            output::error(format!("{err:#}"));
            ExitCode::FAILURE
        }
    }
}
