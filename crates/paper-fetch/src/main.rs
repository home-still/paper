mod cli;
mod commands;
mod exit_codes;
mod output;

use std::sync::Arc;

use clap::Parser;
use cli::{Cli, NounCmd};
use hs_style::mode::{self, OutputMode};
use hs_style::reporter::{Reporter, SilentReporter};
use hs_style::styles::Styles;
use hs_style::tty_reporter::TtyReporter;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();

    let mode = mode::detect(&cli.global.color_str(), cli.global.is_json());

    match mode {
        OutputMode::Rich => owo_colors::set_override(true),
        _ => owo_colors::set_override(false),
    }

    let reporter: Arc<dyn Reporter> = if cli.global.quiet() {
        Arc::new(SilentReporter)
    } else {
        match mode {
            OutputMode::Rich => Arc::new(TtyReporter::new(true)),
            OutputMode::Plain => Arc::new(TtyReporter::new(false)),
            OutputMode::Pipe => Arc::new(hs_style::pipe_reporter::PipeReporter),
        }
    };

    let styles = match mode {
        OutputMode::Rich => Styles::colored(),
        _ => Styles::plain(),
    };

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    let result = rt.block_on(async {
        match cli.command {
            NounCmd::Paper { action } => {
                commands::paper::run(action, &cli.global, &reporter, &styles).await
            }
            NounCmd::Config { action } => commands::config::run(action, &cli.global).await,
        }
    });

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            reporter.error(&format!("Error: {e:#}"));
            exit_codes::from_error(&e)
        }
    }
}
