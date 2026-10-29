use std::io::IsTerminal;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use super::commands;
use super::constants::{DEFAULT_FILTER, TRACE_FILTER, VERBOSE_FILTER};
use super::types::{Cli, Cmd};

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    run_command(cli.command)
}

fn init_tracing(verbose: u8) {
    tracing_subscriber::fmt()
        .with_env_filter(filter(verbose))
        .with_writer(std::io::stderr)
        .with_ansi(std::io::stderr().is_terminal())
        .without_time()
        .init();
}

fn filter(verbose: u8) -> EnvFilter {
    match verbose {
        0 => EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER)),
        1 => EnvFilter::new(VERBOSE_FILTER),
        _ => EnvFilter::new(TRACE_FILTER),
    }
}

fn run_command(command: Cmd) -> Result<()> {
    match command {
        Cmd::Init(args) => commands::init_cmd(args),
        Cmd::Status(args) => commands::status_cmd(args),
        Cmd::Apply(args) => commands::apply_cmd(args),
        Cmd::Watch(args) => commands::watch_cmd(args),
        Cmd::Log(args) => commands::log_cmd(args),
        Cmd::Restore(args) => commands::restore_cmd(args),
        Cmd::Set(args) => commands::set_cmd(args),
        Cmd::Unset(args) => commands::unset_cmd(args),
        Cmd::Service(args) => commands::service_cmd(args),
        Cmd::Completions(args) => commands::completions_cmd(args),
        Cmd::Man => commands::man_cmd(),
    }
}
