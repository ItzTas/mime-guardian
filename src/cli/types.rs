use clap::{ArgAction, Parser, Subcommand};

use super::commands::{
    AdoptArgs, ApplyArgs, CompletionsArgs, InitArgs, LogArgs, RestoreArgs, ServiceArgs, SetArgs,
    StatusArgs, UnsetArgs, WatchArgs,
};

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
    #[command(about = "Pin every protected type to the application you set, once")]
    Apply(ApplyArgs),
    #[command(about = "Keep running and put every protected type back as soon as it changes")]
    Watch(WatchArgs),
    #[command(about = "List the times an application took a protected type and mmg put it back")]
    Log(LogArgs),
    #[command(about = "Put back the mimeapps.list files an earlier write replaced")]
    Restore(RestoreArgs),
    #[command(about = "Protect a type with an application, then pin it")]
    Set(SetArgs),
    #[command(about = "Stop protecting a type")]
    Unset(UnsetArgs),
    #[command(about = "Run mmg watch as a systemd user service")]
    Service(ServiceArgs),
    #[command(
        about = "Run a program and keep the defaults it changes",
        long_about = "Run a program that sets default applications, such as handlr or xdg-mime. \
Once it exits, every type whose default it changed in your mimeapps.list files is copied into \
the configuration, so mmg keeps your choice instead of putting the old one back. The watcher \
waits while the program runs."
    )]
    Adopt(AdoptArgs),
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
