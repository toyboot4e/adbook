use anyhow::*;
use clap::Clap;

#[derive(Clap, Debug)]
#[clap(name = "adbook", about = "Creates a book from AsciiDoc files")]

pub struct Cli {
    #[clap(subcommand)]
    cmd: SubCommand,
}

impl Cli {
    pub fn run(&self) -> Result<()> {
        self.cmd.run()
    }
}

#[derive(Clap, Debug)]
pub enum SubCommand {
    #[clap(name = "build", alias = "b")]
    /// Builds adbook directory
    Build(Build),
}

impl SubCommand {
    pub fn run(&self) -> Result<()> {
        match self {
            SubCommand::Build(build) => build.run(),
        }
    }
}

#[derive(Clap, Debug)]
pub struct Build {}

impl Build {
    pub fn run(&self) -> Result<()> {
        Ok(())
    }
}
