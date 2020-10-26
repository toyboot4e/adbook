//! Table of contents, list of items in a directory
//!
//! It's recursive and makes up a book file structure.

use {
    std::{
        fmt, fs, io,
        path::{Path, PathBuf},
    },
    thiserror::Error,
};

use crate::book::config::{TocRon, TocRonItem};

/// Error when loading `toc.ron`
#[derive(Debug, Error)]
pub enum TocLoadError {
    /// (relative_path_to_the_file, book_ron_directory_path)
    #[error("Unable to locate `{0}` in `{1}`")]
    FailedToLocateItem(PathBuf, PathBuf),
    /// (relative_path_to_the_file, book_ron_directory_path)
    #[error("Unable to locate preface `{0}` in `{1}`")]
    FailedToLocatePreface(PathBuf, PathBuf),
    #[error("Unexpected item with path: {0}")]
    FoundOddItem(PathBuf),
    #[error("Found directory without `toc.ron`: {0}")]
    FoundDirectoryWithoutTocRon(PathBuf),
    #[error("Failed to read toc.ron at: {0}. IO error: {1}")]
    FailedToReadTocRon(PathBuf, io::Error),
    #[error("Failed to parse toc.ron at: {0}")]
    FailedToParseTocRon(PathBuf, ron::Error),
    #[error("Errors in sub `toc.ron`: {0}")]
    FoundErrorsInSubToc(Box<SubTocLoadErrors>),
}

/// Errors when loading a sub `toc.ron`, a type just for printing
#[derive(Debug)]
pub struct SubTocLoadErrors {
    errors: Vec<TocLoadError>,
}

impl fmt::Display for SubTocLoadErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for err in &self.errors {
            writeln!(f, "{}", err)?;
        }
        Ok(())
    }
}

/// The recursive book structure, corresponds to `mod.rs` in Rust
#[derive(Debug, Clone)]
pub struct Toc {
    /// Absolute path to the directory
    pub dir: PathBuf,
    pub name: String,
    /// File that describes this directory
    pub preface: PathBuf,
    /// Items
    pub items: Vec<TocItem>,
}

#[derive(Debug, Clone)]
pub enum TocItem {
    /// (name, absolute_path)
    File(String, PathBuf),
    Dir(Box<Toc>),
}

impl Toc {
    /// Loads `toc.ron` recursively. Invalid items are excluded
    pub fn from_toc_ron_recursive(
        toc_ron: &TocRon,
        toc_ron_dir: &Path,
    ) -> Result<(Self, Vec<TocLoadError>), TocLoadError> {
        let mut errors = vec![];
        let mut items = vec![];

        trace!("parsing toc.ron at directory `{}`", toc_ron_dir.display());

        let preface = {
            let file = toc_ron_dir.join(&toc_ron.preface.1);
            // preface file is required
            if !file.is_file() {
                return Err(TocLoadError::FailedToLocatePreface(
                    toc_ron.preface.1.to_owned(),
                    toc_ron_dir.to_owned(),
                ));
            }
            file.canonicalize().unwrap()
        };

        for item in &toc_ron.items {
            match item {
                TocRonItem::File(name, rel_path) => {
                    let path = {
                        let path = toc_ron_dir.join(rel_path);
                        if !path.exists() {
                            errors.push(TocLoadError::FailedToLocateItem(
                                rel_path.into(),
                                toc_ron_dir.to_path_buf(),
                            ));
                            continue;
                        }
                        path.canonicalize().unwrap()
                    };

                    items.push(TocItem::File(name.to_string(), path));
                }
                TocRonItem::Dir(rel_path) => {
                    let path = {
                        let path = toc_ron_dir.join(rel_path);
                        if !path.exists() {
                            errors.push(TocLoadError::FailedToLocateItem(
                                rel_path.into(),
                                toc_ron_dir.to_path_buf(),
                            ));
                            continue;
                        }
                        path.canonicalize().unwrap()
                    };

                    let (toc, toc_errors) = {
                        let nested_toc_ron = {
                            let file = path.join("toc.ron");
                            if !file.is_file() {
                                errors.push(TocLoadError::FoundDirectoryWithoutTocRon(path));
                                continue;
                            }
                            file
                        };

                        let toc_ron: TocRon = {
                            let toc_ron_str = match fs::read_to_string(&nested_toc_ron) {
                                Ok(s) => s,
                                Err(err) => {
                                    errors.push(TocLoadError::FailedToReadTocRon(path, err));
                                    continue;
                                }
                            };

                            match ron::from_str(&toc_ron_str) {
                                Ok(ron) => ron,
                                Err(err) => {
                                    errors
                                        .push(TocLoadError::FailedToParseTocRon(path.clone(), err));
                                    continue;
                                }
                            }
                        };

                        match Toc::from_toc_ron_recursive(&toc_ron, &path) {
                            Ok((a, b)) => (a, b),
                            Err(err) => {
                                errors.push(err);
                                continue;
                            }
                        }
                    };

                    if !toc_errors.is_empty() {
                        errors.push(TocLoadError::FoundErrorsInSubToc(Box::new(
                            SubTocLoadErrors { errors: toc_errors },
                        )));
                    }

                    items.push(TocItem::Dir(Box::new(toc)));
                }
            }
        }

        Ok((
            Self {
                dir: toc_ron_dir.to_path_buf(),
                name: toc_ron.preface.0.to_owned(),
                preface,
                items,
            },
            errors,
        ))
    }
}
