use anyhow::{Context, Result};
use clap::{Args, CommandFactory};
use clap_complete::Shell;

use crate::cli::Cli;

#[derive(Debug, Args)]
pub struct CompletionsArgs {
    #[arg(value_name = "SHELL", value_enum, help = "The shell the script is for")]
    pub shell: Shell,
}

pub fn completions_cmd(args: CompletionsArgs) -> Result<()> {
    print!("{}", script(args.shell)?);
    Ok(())
}

fn script(shell: Shell) -> Result<String> {
    let mut command = Cli::command();
    let name = command.get_name().to_string();

    let mut script = Vec::new();
    clap_complete::generate(shell, &mut command, name, &mut script);

    String::from_utf8(script).context("completions: the generated script is not valid UTF-8")
}
