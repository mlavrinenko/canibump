use std::process::ExitCode;

use clap::Parser;

fn main() -> ExitCode {
    env_logger::init();
    let cli = canibump::cli::Cli::parse();
    match canibump::cli::run(&cli) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("canibump: {error:#}");
            ExitCode::from(canibump::report::EXIT_ERROR)
        }
    }
}
