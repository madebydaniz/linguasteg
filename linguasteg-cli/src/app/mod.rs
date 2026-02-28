mod analysis;
mod args;
mod commands;
mod formatters;
mod runtime;
mod trace;
mod types;

use std::process::ExitCode;

use args::{parse_command, write_usage};
use commands::execute;
use types::CliError;

pub(crate) fn run() -> ExitCode {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let command = match parse_command(args) {
        Ok(Some(command)) => command,
        Ok(None) => {
            let _ = write_usage(std::io::stdout());
            return ExitCode::SUCCESS;
        }
        Err(CliError::Usage(message)) => {
            eprintln!("{message}");
            let _ = write_usage(std::io::stderr());
            return ExitCode::from(2);
        }
    };

    match execute(command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(1)
        }
    }
}
