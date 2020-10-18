//! Command line interface
//!
//! Built on top of clap ver.3 beta.

use {
    anyhow::*,
    clap::Clap,
    colored::*,
    std::{
        fs,
        io::prelude::*,
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
    #[clap(name = "init", alias = "i")]
    /// Initializes a directory as an `adbook` project
    Init(Init),
    #[clap(name = "build", alias = "b")]
    /// Builds an `adbook` project
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
    dir: Option<String>,
}

impl Build {
    pub fn run(&self) -> Result<()> {
        let dir = self.dir.as_ref().unwrap_or(&".".into()).clone();

        trace!("===> Loading book structure");
        let book = BookStructure::from_dir(&dir)?;

        trace!("===> Building the book");
        crate::builder::build(&book)?;

        Ok(())
    }
}

/// `adbook init`
#[derive(Clap, Debug)]
pub struct Init {
    dir: Option<String>,
}

impl Init {
    pub fn run(&self) -> Result<()> {
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

        // create `.gitigore`, `src/toc.ron`, `src/1.adoc`, `src/img`
        let ignore = dir.join(".gitignore");
        if !ignore.exists() {
            let mut f = fs::File::create(ignore)?;
            writeln!(f, ".DS_Store")?;
        }

        let book = dir.join("book.ron");
        let mut book = fs::File::create(&book)?;
        write!(
            book,
            r#"(
    authors: ["author"],
    title: "title",
    src: "src",
    site: "site",

    includes: [
        "img",
    ],

    // options for `asciidoctor`
    adoc_opts: [
        ("-a", [
            "linkcss",
            //
            "imagesdir@=/img",
            "imagesoutdir@=${{src_dir}}/img",
            //
            "hardbreaks",
            "sectnums",
            "sectnumlevels@=2",
            //
            "experimental",
            "stem@=latexmath",
            "icons@=font",
        ]),

        // add extensions
        ("-r", [
            "asciidoctor-diagram",
        ]),
    ]
)
"#
        )?;

        let src = dir.join("src");
        if !src.exists() {
            fs::create_dir(&src)?;
        }

        let toc = src.join("toc.ron");
        if !toc.exists() {
            let mut f = fs::File::create(&toc)?;
            write!(
                f,
                r#"(
    items:[
        ("1", "1.adoc"),
    ]
)"#
            )?;
        }

        let adoc = src.join("1.adoc");
        if !adoc.exists() {
            let mut f = fs::File::create(&adoc)?;
            writeln!(f, "= Helllllooo\n")?;
        }

        let img = src.join("img");
        if !img.exists() {
            fs::create_dir(&img)?;
        }

        println!(
            "Initialized new adbook project at {}",
            format!("{}", dir.display()).green()
        );

        Ok(())
    }
}
