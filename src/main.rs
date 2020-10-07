use adbook::cli::Cli;
use anyhow::*;
use clap::Clap;

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.run()?;

    Ok(())
}
