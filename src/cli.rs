/*!
Command line interface

Built on top of clap ver.3 beta.

# Example

`main.rs`:

```no_run
use {adbook::cli::Cli, anyhow::*, clap::Clap};

fn main() -> Result<()> {
    // initialize your `log` implemetation
    // env_logger::init();
    // then run!
    Cli::parse().run()
}
```
*/

use {
    anyhow::*,
    clap::Clap,
    colored::*,
    std::{
        fs,
        path::{Path, PathBuf},
    },
};

use crate::book::BookStructure;

// `adbook`
#[derive(Clap, Debug)]
#[clap(
    name = "adbook creates a book from AsciiDoc files",
    setting = clap::AppSettings::ColoredHelp
)]
pub struct Cli {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

impl Cli {
    pub fn run(&mut self) -> Result<()> {
        self.cmd.run()
    }
}

#[derive(Clap, Debug)]
pub enum SubCommand {
    #[clap(name = "init", alias = "i")]
    /// Initializes a directory as an `adbook` project
    Init(Init),
    #[clap(name = "build", alias = "b")]
    /// Builds an `adbook` project
    Build(Build),
    /// Prints one of the preset files: `article.adoc`, `book.ron` or `toc.ron`
    #[clap(name = "preset", alias = "p")]
    Preset(Preset),
    /// Clears site directory contents
    Clean(Clean),
}

impl SubCommand {
    pub fn run(&mut self) -> Result<()> {
        match self {
            SubCommand::Build(build) => build.run(),
            SubCommand::Init(init) => init.run(),
            SubCommand::Preset(preset) => preset.run(),
            SubCommand::Clean(clean) => clean.run(),
        }
    }
}

/// `adbook build`
#[derive(Clap, Debug)]
pub struct Build {
    pub dir: Option<String>,
}

impl Build {
    pub fn run(&mut self) -> Result<()> {
        let dir = self.dir.as_ref().unwrap_or(&".".into()).clone();

        trace!("---- Loading book structure");
        let book = BookStructure::from_dir(&dir)?;

        info!("===> Building the book");
        crate::build::build_book(&book)?;
        info!("<==> Finished bulding");

        Ok(())
    }
}

/// `adbook init`
#[derive(Clap, Debug)]
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
                "book.ron exists in the target directory"
            );
        }

        if !dir.exists() {
            fs::create_dir(&dir).with_context(|| {
                format!("Unable to create init directory at: {}", dir.display())
            })?;
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
#[derive(Clap, Debug)]
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
            "t" | "toc" | "toc.ron" => {
                let s = std::str::from_utf8(files::src::TOC)?;
                println!("{}", s);
            }
            "a" | "article" | "article.adoc" => {
                let s = std::str::from_utf8(files::src::ARTICLE)?;
                println!("{}", s);
            }
            _ => {
                eprintln!("specify one of `book`, `toc` or `article");
            }
        }

        Ok(())
    }
}

/// `adbook clean`
#[derive(Clap, Debug)]
pub struct Clean {
    pub dir: Option<String>,
}

impl Clean {
    pub fn run(&mut self) -> Result<()> {
        let dir = self.dir.as_ref().unwrap_or(&".".into()).clone();

        info!("===> Loading book structure");
        let book = BookStructure::from_dir(dir)?;

        fn is_path_to_keep(path: &Path) -> bool {
            let name = match path.file_name().and_then(|s| s.to_str()) {
                Some(name) => name,
                None => {
                    error!("Unexpected path while clearning: {}", path.display());
                    return true;
                }
            };
            name.starts_with(".")
        }

        info!("===> Clearing the site directory");
        crate::utils::clear_directory_items(&book.site_dir_path(), is_path_to_keep)?;

        Ok(())
    }
}
