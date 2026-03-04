mod analysis;
mod args;
mod commands;
mod formatters;
mod language;
mod runtime;
mod trace;
mod trace_contract;
mod types;

use std::process::ExitCode;

use args::{parse_command, write_usage};
use commands::execute;
use types::{CliError, CliErrorKind};

pub(crate) fn run() -> ExitCode {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let command = match parse_command(args) {
        Ok(Some(command)) => command,
        Ok(None) => {
            let _ = write_usage(std::io::stdout());
            return ExitCode::SUCCESS;
        }
        Err(error) if error.kind() == CliErrorKind::Usage => {
            print_error(&error);
            let _ = write_usage(std::io::stderr());
            return ExitCode::from(error.exit_code());
        }
        Err(error) => return print_error_exit(error),
    };

    match execute(command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => print_error_exit(error),
    }
}

fn print_error_exit(error: CliError) -> ExitCode {
    print_error(&error);
    ExitCode::from(error.exit_code())
}

fn print_error(error: &CliError) {
    eprintln!("error [{}]: {}", error.code(), error.message());
}
