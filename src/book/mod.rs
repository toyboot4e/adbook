/*! Book data structure

The book structure is actually built into a site (destination) directory with
[`crate::builder`].
!*/

pub mod config;

use {
    anyhow::{Context, Result},
    colored::*,
    std::{
        fs,
        path::{Path, PathBuf},
    },
    thiserror::Error,
};

use self::config::{BookRon, Toc, TocRon};

/// Error while loading `book.ron`
#[derive(Error, Debug)]
pub enum BookLoadError {
    #[error("Given non-directory path")]
    GivenNonDirectoryPath,
    #[error("Not found root directory (not found `book.ron`)")]
    NotFoundRoot,
}

/// File structure of an adbook project read from `book.ron` and `toc.ron`s
#[derive(Debug, Clone)]
pub struct BookStructure {
    /// Absolute path to a directory with `book.ron`
    pub root: PathBuf,
    pub book_ron: BookRon,
    pub toc: Toc,
}

impl BookStructure {
    pub fn src_dir_path(&self) -> PathBuf {
        self.root.join(&self.book_ron.src)
    }

    pub fn site_dir_path(&self) -> PathBuf {
        self.root.join(&self.book_ron.site)
    }
}

impl BookStructure {
    /// Tries to find `book.ron` going up the directories and parses it into a file structure
    pub fn from_dir(path: impl AsRef<Path>) -> Result<Self> {
        let book_ron_path = self::find_root_book_ron(path)?;
        info!("book.ron located at: {}", book_ron_path.display());

        let root = book_ron_path
            .parent()
            .unwrap()
            .canonicalize()
            .with_context(|| {
                format!(
                    "Failed to canonicalize parent directory of `book.ron`: {}",
                    book_ron_path.display()
                )
            })?;

        let book_ron: BookRon = {
            let cfg_str = fs::read_to_string(&book_ron_path).with_context(|| {
                format!(
                    "Failed to load root `book.ron` file. Expected path: {}",
                    book_ron_path.display()
                )
            })?;

            // Here we actually load `book.ron`
            ron::from_str(&cfg_str).with_context(|| {
                format!("Failed to load book.ron at: {}", book_ron_path.display())
            })?
        };
        trace!("root `book.ron` loaded: {:?}", book_ron);

        let src_dir = root.join(&book_ron.src);

        let (toc, toc_errors) = {
            let toc_path = src_dir.join("toc.ron");
            let toc_str = fs::read_to_string(&toc_path).with_context(|| {
                format!("Unable to read root `toc.ron` at: {}", toc_path.display())
            })?;

            let toc_ron: TocRon = ron::from_str(&toc_str)
                .with_context(|| format!("Failed to parse `toc.ron` at: {}", toc_path.display()))?;
            trace!("root toc.ron loaded: {:?}", toc_ron);

            // Here we actually load a root `toc.ron`
            Toc::from_toc_ron_recursive(&toc_ron, &src_dir)
        };
        trace!("toc.ron loaded: {:#?}", toc);

        if !toc_errors.is_empty() {
            eprintln!(
                "{} {}",
                format!("{}", toc_errors.len()).red(),
                "errors while parsing `toc.ron`:".red()
            );
            for err in &toc_errors {
                eprintln!("- {}", err);
            }
        }

        Ok(Self {
            root,
            book_ron,
            toc,
        })
    }
}

/// Tries to return a canonicalized path to `book.ron` locating a root directory
fn find_root_book_ron(path: impl AsRef<Path>) -> Result<PathBuf> {
    let path = path.as_ref().canonicalize().with_context(|| {
        format!(
            "Unable to find given directory path: {}",
            path.as_ref().display()
        )
    })?;

    ensure!(path.is_dir(), BookLoadError::GivenNonDirectoryPath);

    // go up the ancestors and find `book.ron`
    for dir in path.ancestors() {
        let book_ron = dir.join("book.ron");
        if !book_ron.is_file() {
            continue;
        }
        return Ok(book_ron);
    }

    Err(BookLoadError::NotFoundRoot.into())
}
