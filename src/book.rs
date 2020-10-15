//! Book data structure

use {
    anyhow::Result,
    std::path::{Path, PathBuf},
    thiserror::Error,
};

use crate::config::Config;

#[derive(Error, Debug)]
pub enum BookLoadError {
    #[error("Not given directory path")]
    NotGivenDirectoryPath,
    #[error("book.ron not found")]
    NotFoundBookRon,
}

pub struct Book {
    pub cfg: Config,
}

impl Book {
    pub fn load_dir(path: impl AsRef<Path>) -> Result<Self> {
        let book_ron = self::find_book_ron(path)?;
        info!("book.ron located at {}", book_ron.display());

        // default configuration (hard coded)
        let cfg = Config {
            authors: vec!["toyboot4e".into()],
            src: "src".into(),
            title: "test book".into(),
        };

        Ok(Self { cfg })
    }
}

/// Tries to return a canoncalized path to `book.ron`
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
