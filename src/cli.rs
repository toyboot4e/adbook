//! Command line interface
//!
//! Built on top of clap ver.3 beta.

use {anyhow::*, clap::Clap};

use crate::book::BookStructure;

// `adbook`
#[derive(Clap, Debug)]
#[clap(
    name = "adbook creates a book from AsciiDoc files",
    setting = clap::AppSettings::ColoredHelp
)]
pub struct Cli {
    #[clap(subcommand)]
    cmd: SubCommand,
}

impl Cli {
    pub fn run(&self) -> Result<()> {
        self.cmd.run()
    }
}

// `adbook <sub command>`
#[derive(Clap, Debug)]
pub enum SubCommand {
    #[clap(name = "new", alias = "n")]
    /// Initializes a directory as an `adbook` project
    Init(Init),
    #[clap(name = "build", alias = "b")]
    /// Builds `adbook` projects
    Build(Build),
}

impl SubCommand {
    pub fn run(&self) -> Result<()> {
        match self {
            SubCommand::Build(build) => build.run(),
            SubCommand::Init(new) => new.run(),
        }
    }
}

/// `adbook build`
#[derive(Clap, Debug)]
pub struct Build {
    #[clap(short, long, default_value = ".")]
    dir: String,
}

impl Build {
    pub fn run(&self) -> Result<()> {
        trace!("===> Loading book structure");
        let book = BookStructure::from_dir(&self.dir)?;

        trace!("===> Building the book");
        crate::builder::build(&book)?;

        Ok(())
    }
}

/// `adbook init`
#[derive(Clap, Debug)]
pub struct Init {
    #[clap(short, long, default_value = ".")]
    dir: String,
}

impl Init {
    pub fn run(&self) -> Result<()> {
        trace!("===> Loading book structure");
        let book = BookStructure::from_dir(&self.dir)?;

        trace!("===> Building the book");
        crate::builder::build(&book)?;

        Ok(())
    }
}
