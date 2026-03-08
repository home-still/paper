mod cli;
mod commands;
mod exit_codes;
mod output;

use clap::Parser;
use cli::{Cli, NounCmd};
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    let result = rt.block_on(async {
        match cli.command {
            NounCmd::Paper { action } => commands::paper::run(action, &cli.global).await,
            NounCmd::Config { action } => commands::config::run(action, &cli.global).await,
        }
    });

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            if !cli.global.quiet {
                eprintln!("Error: {e:#}");
            }
            exit_codes::from_error(&e)
        }
    }
}
