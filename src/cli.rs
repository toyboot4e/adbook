/*!
Command line interface by clap 3.0

# Example

`main.rs`:

```no_run
use adbook::cli::Cli;
use anyhow::*;
use clap::Parser;

fn main() -> Result<()> {
    // initialize your `log` implemetation
    // env_logger::init();

    // then run!
    Cli::parse().run()
}
```
*/

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::*;
use clap::Parser;
use colored::*;

use crate::book::BookStructure;

// `adbook`
#[derive(Parser, Debug)]
#[clap(name = "adbook is a simple SSG powered by asciidoctor")]
pub struct Cli {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

impl Cli {
    pub fn run(&mut self) -> Result<()> {
        self.cmd.run()
    }
}

#[derive(Parser, Debug)]
pub enum SubCommand {
    /// Initializes a directory as an `adbook` project
    #[clap(name = "init", alias = "i")]
    Init(Init),
    /// Builds an `adbook` project
    #[clap(name = "build", alias = "b")]
    Build(Build),
    /// Prints one of the preset files: `article.adoc`, `book.ron` or `index.ron`
    #[clap(name = "preset", alias = "p")]
    Preset(Preset),
    /// Clears the site directory contents and the build cache
    Clear(Clear),
}

impl SubCommand {
    pub fn run(&mut self) -> Result<()> {
        match self {
            SubCommand::Build(build) => build.run(),
            SubCommand::Init(init) => init.run(),
            SubCommand::Preset(preset) => preset.run(),
            SubCommand::Clear(clear) => clear.run(),
        }
    }
}

/// `adbook build`
#[derive(Parser, Debug)]
pub struct Build {
    pub dir: Option<String>,
    /// Clears cache and builds the whole book
    #[clap(short, long = "force")]
    pub force_rebuild: bool,
    /// Prints verbose log
    #[clap(short, long)]
    pub verbose: bool,
}

impl Build {
    pub fn run(&mut self) -> Result<()> {
        let dir = self.dir.as_ref().unwrap_or(&".".into()).clone();

        log::trace!("---- Loading book structure");
        let book = BookStructure::from_dir(&dir)?;

        log::info!("===> Building the book");
        crate::build::build_book(&book, self.force_rebuild, self.verbose)?;
        log::info!("<==> Finished bulding");

        Ok(())
    }
}

/// `adbook init`
#[derive(Parser, Debug)]
pub struct Init {
    pub dir: String,
}

impl Init {
    pub fn run(&mut self) -> Result<()> {
        let dir = PathBuf::from(&self.dir);

        {
            let book_ron = dir.join("book.ron");
            ensure!(
                !book_ron.exists(),
                "`book.ron` already exists in the target directory"
            );
        }

        if !dir.exists() {
            fs::create_dir(&dir)
                .with_context(|| format!("Failed to create directory at `{}`", dir.display()))?;
        }

        crate::book::init::gen_init_files(&dir)?;

        println!(
            "Initialized a new adbook project at {}",
            format!("{}", dir.display()).green()
        );

        Ok(())
    }
}

/// `adbook preset`
#[derive(Parser, Debug)]
pub struct Preset {
    pub file: Option<String>,
}

impl Preset {
    pub fn run(&mut self) -> Result<()> {
        use crate::book::init::files;

        let file = self.file.as_ref().map(|s| s.as_str()).unwrap_or("");
        match file {
            "b" | "book" | "book.ron" => {
                let s = std::str::from_utf8(files::BOOK)?;
                println!("{}", s);
            }
            "i" | "index" | "index.ron" => {
                let s = std::str::from_utf8(files::src::INDEX_RON)?;
                println!("{}", s);
            }
            "a" | "article" | "article.adoc" => {
                let s = std::str::from_utf8(files::src::ARTICLE)?;
                println!("{}", s);
            }
            _ => {
                eprintln!("specify one of `book`, `index` or `article");
            }
        }

        Ok(())
    }
}

/// `adbook clear`
#[derive(Parser, Debug)]
pub struct Clear {
    pub dir: Option<String>,
}

impl Clear {
    pub fn run(&mut self) -> Result<()> {
        let dir = self.dir.as_ref().unwrap_or(&".".into()).clone();

        log::info!("===> Loading book structure");
        let book = BookStructure::from_dir(dir)?;

        fn is_path_to_keep(path: &Path) -> bool {
            let name = match path.file_name().and_then(|s| s.to_str()) {
                Some(name) => name,
                None => {
                    log::error!("Unexpected path while clearning: {}", path.display());
                    return true;
                }
            };
            name.starts_with(".")
        }

        log::info!("===> Clearing the site directory");
        crate::utils::clear_directory_items(&book.site_dir_path(), is_path_to_keep)?;

        log::info!("===> Clearing build cache");
        crate::build::cache::clear_cache(&book)?;

        Ok(())
    }
}
