//! Book data structure

use {
    anyhow::{Context, Result},
    serde::de::Deserialize,
    std::{
        fs,
        path::{Path, PathBuf},
    },
    thiserror::Error,
};

use crate::config::{BookRon, Toc, TocRon};

#[derive(Error, Debug)]
pub enum BookLoadError {
    #[error("Not given directory path")]
    NotGivenDirectoryPath,
    #[error("No `book.ron` found")]
    NotFoundBookRon,
}

/// Directory structure of a book project
#[derive(Debug)]
pub struct BookStructure {
    pub book_ron: BookRon,
    pub toc: Toc,
}

impl BookStructure {
    pub fn load_dir(path: impl AsRef<Path>) -> Result<Self> {
        let book_ron_path = self::find_book_ron(path)?;
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
                    "Failed to locate `book.ron`. Expected path: {}",
                    book_ron_path.display()
                )
            })?;

            ron::from_str(&cfg_str).with_context(|| {
                format!("Failed to load book.ron at: {}", book_ron_path.display())
            })?
        };
        trace!("book.ron loaded: {:?}", book_ron);

        let src = root.join(&book_ron.src);

        let (toc, toc_errors) = {
            let toc_path = src.join("toc.ron");
            let toc_str = fs::read_to_string(&toc_path).with_context(|| {
                format!("Failed to locate root toc file at: {}", toc_path.display())
            })?;

            let toc_ron: TocRon = ron::from_str(&toc_str)
                .with_context(|| format!("Failed to parse `toc.ron` at: {}", toc_path.display()))?;
            trace!("root toc.ron loaded: {:?}", toc_ron);

            Toc::from_toc_ron_recursive(&toc_ron, &src)
        };
        trace!("toc.ron loaded: {:#?}", toc);

        if !toc_errors.is_empty() {
            eprintln!("{} errors while parsing `toc.ron`:", toc_errors.len());
            for err in &toc_errors {
                eprintln!("- {}", err);
            }
        }

        Ok(Self { book_ron, toc })
    }
}

/// Tries to return a canonicalized path to `book.ron`
fn find_book_ron(path: impl AsRef<Path>) -> Result<PathBuf> {
    let path = path.as_ref().canonicalize()?;
    ensure!(path.is_dir(), BookLoadError::NotGivenDirectoryPath);

    // go up the ancestors and find `book.ron`
    for dir in path.ancestors() {
        let book_ron = dir.join("book.ron");
        if !book_ron.is_file() {
            continue;
        }
        return Ok(book_ron);
    }

    Err(BookLoadError::NotFoundBookRon.into())
}
