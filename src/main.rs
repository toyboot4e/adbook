use {adbook::cli::Cli, anyhow::*, clap::Clap};

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();
    cli.run()?;

    Ok(())
}
