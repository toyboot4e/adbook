/*!
List of items in a directory

It's recursive and makes up a book file structure.
*/

use {
    std::{
        fmt, fs, io,
        path::{Path, PathBuf},
    },
    thiserror::Error,
};

use crate::book::config::{IndexRon, IndexRonItem};

const INDEX_RON: &'static str = "index.ron";

/// Error when loading `index.ron`
#[derive(Debug, Error)]
pub enum IndexLoadError {
    /// (relative_path_to_the_file, book_ron_directory_path)
    #[error("Unable to locate `{0}` in `{1}`")]
    FailedToLocateItem(PathBuf, PathBuf),
    /// (relative_path_to_the_file, book_ron_directory_path)
    #[error("Unable to locate summary `{0}` in `{1}`")]
    FailedToLocateSummary(PathBuf, PathBuf),
    #[error("Unexpected item with path: {0}")]
    FoundOddItem(PathBuf),
    #[error("Found directory without `index.ron`: {0}")]
    FoundDirectoryWithoutIndexRon(PathBuf),
    #[error("Failed to read `index.ron` at: {0}. IO error: {1}")]
    FailedToReadIndexRon(PathBuf, io::Error),
    #[error("Failed to parse `index.ron` at: {0}")]
    FailedToParseIndexRon(PathBuf, ron::Error),
    #[error("Errors in sub `index.ron`: {0}")]
    FoundErrorsInSubIndex(Box<SubIndexLoadErrors>),
}

/// Errors when loading a sub `index.ron`, a type just for printing
#[derive(Debug)]
pub struct SubIndexLoadErrors {
    errors: Vec<IndexLoadError>,
}

impl fmt::Display for SubIndexLoadErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for err in &self.errors {
            writeln!(f, "{}", err)?;
        }
        Ok(())
    }
}

/// The recursive book structure, corresponds to `mod.rs` in Rust
#[derive(Debug, Clone)]
pub struct Index {
    /// Absolute path to the directory
    pub dir: PathBuf,
    pub name: String,
    /// File that describes this directory
    pub summary: PathBuf,
    /// Items
    pub items: Vec<IndexItem>,
}

#[derive(Debug, Clone)]
pub enum IndexItem {
    /// (name, absolute_path)
    File(String, PathBuf),
    Dir(Box<Index>),
}

impl Index {
    /// Loads `index.ron` recursively. Invalid items are excluded
    pub fn from_index_ron_recursive(
        ix_ron: &IndexRon,
        ix_ron_dir: &Path,
    ) -> Result<(Self, Vec<IndexLoadError>), IndexLoadError> {
        let mut errors = vec![];
        let mut items = vec![];

        // trace!("parsing `index.ron` at directory `{}`", index_ron_dir.display());

        let preface = {
            let file = ix_ron_dir.join(&ix_ron.summary.1);
            // preface file is required
            if !file.is_file() {
                return Err(IndexLoadError::FailedToLocateSummary(
                    ix_ron.summary.1.to_owned(),
                    ix_ron_dir.to_owned(),
                ));
            }
            file.canonicalize().unwrap()
        };

        for item in &ix_ron.items {
            match item {
                IndexRonItem::File(name, rel_path) => {
                    let path = {
                        let path = ix_ron_dir.join(rel_path);
                        if !path.exists() {
                            errors.push(IndexLoadError::FailedToLocateItem(
                                rel_path.into(),
                                ix_ron_dir.to_path_buf(),
                            ));
                            continue;
                        }
                        path.canonicalize().unwrap()
                    };

                    items.push(IndexItem::File(name.to_string(), path));
                }
                IndexRonItem::Dir(rel_path) => {
                    let path = {
                        let path = ix_ron_dir.join(rel_path);
                        if !path.exists() {
                            errors.push(IndexLoadError::FailedToLocateItem(
                                rel_path.into(),
                                ix_ron_dir.to_path_buf(),
                            ));
                            continue;
                        }
                        path.canonicalize().unwrap()
                    };

                    let (index, index_errors) = {
                        let nested_index_ron = {
                            let file = path.join(INDEX_RON);
                            if !file.is_file() {
                                errors.push(IndexLoadError::FoundDirectoryWithoutIndexRon(path));
                                continue;
                            }
                            file
                        };

                        let index_ron: IndexRon = {
                            let index_ron_str = match fs::read_to_string(&nested_index_ron) {
                                Ok(s) => s,
                                Err(err) => {
                                    errors.push(IndexLoadError::FailedToReadIndexRon(path, err));
                                    continue;
                                }
                            };

                            match crate::utils::load_ron(&index_ron_str) {
                                Ok(ron) => ron,
                                Err(err) => {
                                    errors.push(IndexLoadError::FailedToParseIndexRon(
                                        path.clone(),
                                        err,
                                    ));
                                    continue;
                                }
                            }
                        };

                        match Index::from_index_ron_recursive(&index_ron, &path) {
                            Ok((a, b)) => (a, b),
                            Err(err) => {
                                errors.push(err);
                                continue;
                            }
                        }
                    };

                    if !index_errors.is_empty() {
                        errors.push(IndexLoadError::FoundErrorsInSubIndex(Box::new(
                            SubIndexLoadErrors {
                                errors: index_errors,
                            },
                        )));
                    }

                    items.push(IndexItem::Dir(Box::new(index)));
                }
            }
        }

        Ok((
            Self {
                dir: ix_ron_dir.to_path_buf(),
                name: ix_ron.summary.0.to_owned(),
                summary: preface,
                items,
            },
            errors,
        ))
    }
}
