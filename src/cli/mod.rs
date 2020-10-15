use {anyhow::*, clap::Clap};

use crate::book::Book;

/// `adbook` command line interface
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
pub struct Build {
    #[clap(short, long, default_value = ".")]
    dir: String,
}

impl Build {
    pub fn run(&self) -> Result<()> {
        let book = Book::load_dir(&self.dir)?;

        Ok(())
    }
}
