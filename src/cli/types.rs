use clap::{ArgAction, Parser, Subcommand};

use super::commands::{CompletionsArgs, InitArgs, StatusArgs};

#[derive(Debug, Parser)]
#[command(
    name = "mmg",
    version,
    about = "Keeps your default applications the way you set them",
    long_about = "mmg watches the mimeapps.list files that decide which application \
opens each file type and link. When a program quietly makes itself the default, mmg puts \
your choice back and tells you what changed.\n\
\n\
The defaults you want live in a TOML file. Wildcards such as image/* expand to every image type \
the shared MIME database knows.",
    arg_required_else_help = true
)]
pub struct Cli {
    #[arg(
        short,
        long,
        global = true,
        action = ArgAction::Count,
        help = "Log what mmg is doing (-vv logs more)"
    )]
    pub verbose: u8,
    #[command(subcommand)]
    pub command: Cmd,
}

#[derive(Debug, Subcommand)]
pub enum Cmd {
    #[command(about = "Create the configuration from the defaults this session uses now")]
    Init(InitArgs),
    #[command(about = "List the protected types whose default is not the one you set")]
    Status(StatusArgs),
    #[command(about = "Print a completion script for a shell")]
    Completions(CompletionsArgs),
    #[command(about = "Print the manual page")]
    Man,
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn the_declaration_is_consistent() {
        Cli::command().debug_assert();
    }
}
