use {adbook::cli::Cli, anyhow::*, clap::Clap};

fn main() -> Result<()> {
    env_logger::init();
    Cli::parse().run()
}
