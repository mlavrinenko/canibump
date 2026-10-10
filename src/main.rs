use anyhow::Result;
use clap::Parser;

/// Check whether upstream fixes you wait on have landed, so you know when to bump
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Name to greet
    #[arg(default_value = "canibump")]
    name: String,
}

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();
    canibump::greet(&cli.name)?;

    Ok(())
}
