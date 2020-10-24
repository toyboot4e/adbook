//! Command line interface
//!
//! Built on top of clap ver.3 beta.
//!
//! # Example
//!
//! `main.rs`:
//!
//! ```no_run
//! use {adbook::cli::Cli, anyhow::*, clap::Clap};
//!
//! fn main() -> Result<()> {
//!     env_logger::init();
//!     Cli::parse().run()
//! }
//! ```

use {
    anyhow::*,
    clap::Clap,
    colored::*,
    futures::executor::block_on,
    std::{fs, path::PathBuf},
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

/// `adbook <sub command>`
#[derive(Clap, Debug)]
pub enum SubCommand {
    #[clap(name = "init", alias = "i")]
    /// Initializes a directory as an `adbook` project
    Init(Init),
    #[clap(name = "build", alias = "b")]
    /// Builds an `adbook` project
    Build(Build),
    /// Converts an AsciiDoc file
    #[clap(name = "convert", alias = "c")]
    Convert(Convert),
    /// Prints one of the preset files: `article.adoc`, `book.ron` or `toc.ron`
    #[clap(name = "preset", alias = "p")]
    Preset(Preset),
    // TODO: clean
}

impl SubCommand {
    pub fn run(&mut self) -> Result<()> {
        match self {
            SubCommand::Build(build) => build.run(),
            SubCommand::Init(new) => new.run(),
            SubCommand::Preset(preset) => preset.run(),
            SubCommand::Convert(convert) => convert.run(),
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

        info!("===> Loading book structure");
        let book = BookStructure::from_dir(&dir)?;

        info!("===> Building the book");
        block_on(crate::build::build_book(&book))?;

        Ok(())
    }
}

/// `adbook init`
#[derive(Clap, Debug)]
pub struct Init {
    pub dir: Option<String>,
}

impl Init {
    pub fn run(&mut self) -> Result<()> {
        let dir = self.dir.as_ref().unwrap_or(&".".into()).clone();
        let dir = PathBuf::from(&dir);

        if !dir.exists() {
            fs::create_dir(&dir)
                .with_context(|| format!("Unable to create directory at: {}", dir.display()))?;
        } else {
            let book_ron = dir.join("book.ron");
            if book_ron.exists() {
                return Err(anyhow!("book.ron exists in the target directory"));
            }
        }

        // book.ron (ensured that it doesn't exist)
        {
            let book = dir.join("book.ron");
            fs::write(&book, crate::book::preset::BOOK_RON)?;
        }

        // `.gitigore`, `src/toc.ron`, `src/1.adoc`, `src/img`

        {
            let ignore = dir.join(".gitignore");
            if !ignore.exists() {
                fs::write(ignore, ".DS_Store")?;
            }
        }

        let src = dir.join("src");
        if !src.exists() {
            fs::create_dir(&src)?;
        }

        {
            let toc = src.join("toc.ron");
            if !toc.exists() {
                fs::write(toc, crate::book::preset::TOC_RON)?;
            }
        }

        {
            let adoc = src.join("1.adoc");
            if !adoc.exists() {
                fs::write(adoc, crate::book::preset::ARTICLE_ADOC)?;
            }
        }

        {
            let img = src.join("img");
            if !img.exists() {
                fs::create_dir(&img)?;
            }
        }

        println!(
            "Initialized new adbook project at {}",
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
        let file = self.file.as_ref().map(|s| s.as_str()).unwrap_or("");

        match file {
            "b" | "book" | "book.ron" => {
                let s = std::str::from_utf8(crate::book::preset::BOOK_RON)?;
                println!("{}", s);
            }
            "t" | "toc" | "toc.ron" => {
                let s = std::str::from_utf8(crate::book::preset::TOC_RON)?;
                println!("{}", s);
            }
            "a" | "article" | "article.adoc" => {
                let s = std::str::from_utf8(crate::book::preset::ARTICLE_ADOC)?;
                println!("{}", s);
            }
            _ => {
                eprintln!("specify one of `book`, `toc` or `article");
            }
        }

        Ok(())
    }
}

/// `adbook convert`
#[derive(Clap, Debug)]
pub struct Convert {
    pub src_file: PathBuf,
    /// Handlebars template file path **relative to the source file**
    #[clap(long, short)]
    pub hbs: Option<PathBuf>,
}

impl Convert {
    pub fn run(&mut self) -> Result<()> {
        ensure!(self.src_file.is_file(), "Not given path to file");

        let src_dir = self.src_file.parent().unwrap();
        let site_dir = self.src_file.parent().unwrap();
        let dst_name = "<stdout>";

        let mut opts = vec![];
        if let Some(hbs) = self.hbs.take() {
            opts.push(("-a".to_string(), vec![format!("hbs={}", hbs.display())]));
        }

        let text = crate::build::convert::convert_adoc(
            &self.src_file,
            src_dir,
            site_dir,
            dst_name,
            &opts,
        )?;

        println!("{}", text);

        Ok(())
    }
}
