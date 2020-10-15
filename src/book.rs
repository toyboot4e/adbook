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
        let cfg_path = self::find_book_ron(path)?;
        info!("book.ron located at {}", cfg_path.display());

        let cfg_str = fs::read_to_string(&cfg_path)?;
        let cfg = ron::from_str(&cfg_str)
            .with_context(|| format!("failed to load book.ron at `{}`", cfg_path.display()))?;
        trace!("{:?}", cfg);

        Ok(Self { cfg })
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
