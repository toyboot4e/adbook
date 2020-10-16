//! Command line interface
//!
//! Built on top of clap ver.3 beta.

use {anyhow::*, clap::Clap};

use crate::{book::BookStructure, builder::BuildConfig};

/// `adbook`
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

/// `adbook <sub command>`
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
        let build_cfg = BuildConfig::from_site_path(&book.site_dir_path());

        trace!("===> Building the book");
        crate::builder::build(&book, &build_cfg)?;

        Ok(())
    }
}
