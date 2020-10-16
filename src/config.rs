//! Configuration files
//!
//! * `book.ron`: It is put in root directory and sets general settings
//! * `toc.ron`: They are in each directory and list files/sub directories giving names
//!
//! Types with name `Ron` are directly deserialized from configuration files.

use {
    serde::{Deserialize, Serialize},
    std::{
        fmt, fs, io,
        path::{Path, PathBuf},
    },
    thiserror::Error,
};

/// `book.ron`, configurations
#[derive(Deserialize, Serialize, Debug)]
pub struct BookRon {
    pub authors: Vec<String>,
    pub title: String,
    pub src: PathBuf,
    pub out: PathBuf,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TocRon {
    pub items: Vec<(String, String)>,
}

#[derive(Debug, Error)]
pub enum TocLoadError {
    #[error("Failed to locate toc item at: {0}")]
    FailedToLocateItem(PathBuf),
    #[error("Unexpected item with path: {0}")]
    FoundOddItem(PathBuf),
    #[error("Failed to load item at: {0}. IO error: {1}")]
    FailedToLoadFile(PathBuf, io::Error),
    #[error("Failed to parse toc.ron at: {0}. Ron error: {1}")]
    FailedToParseTocRon(PathBuf, ron::Error),
    #[error("Errors in sub `toc.ron`: {0}")]
    FoundErrorsInSubToc(Box<SubTocLoadErrors>),
}

#[derive(Debug)]
pub struct SubTocLoadErrors {
    errors: Vec<TocLoadError>,
}

impl fmt::Display for SubTocLoadErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for err in &self.errors {
            writeln!(f, "{}", err);
        }
        Ok(())
    }
}

/// `toc.ron`, table of contents in the module
#[derive(Debug)]
pub struct Toc {
    items: Vec<TocItem>,
}

impl Toc {
    /// Loads `toc.ron` recursively
    ///
    /// # Warning
    ///
    /// `adbook` can cause stack overflow if there is path definition (e.g. toc item with path
    /// "toc.ron").
    pub fn from_toc_ron_recursive(
        toc_ron: &TocRon,
        toc_ron_dir: &Path,
    ) -> (Self, Vec<TocLoadError>) {
        let mut errors = vec![];
        let mut items = vec![];

        trace!("parsing toc.ron at directory `{}`", toc_ron_dir.display());

        for (name, rel_path) in &toc_ron.items {
            let path = toc_ron_dir.join(rel_path);
            if !path.exists() {
                errors.push(TocLoadError::FailedToLocateItem(path));
                continue;
            }

            if path.is_file() {
                // case 1. File
                items.push(TocItem {
                    name: name.to_string(),
                    content: TocItemContent::File(path.clone()),
                });
            } else if path.is_dir() {
                // case 2. Directory
                let toc_ron_str = match fs::read_to_string(&path) {
                    Ok(s) => s,
                    Err(err) => {
                        errors.push(TocLoadError::FailedToLoadFile(path, err));
                        continue;
                    }
                };

                let toc_ron: TocRon = match ron::from_str(&toc_ron_str) {
                    Ok(ron) => ron,
                    Err(err) => {
                        errors.push(TocLoadError::FailedToParseTocRon(path.clone(), err));
                        continue;
                    }
                };

                let (sub_toc, sub_errors) = Toc::from_toc_ron_recursive(&toc_ron, &path);
                if !sub_errors.is_empty() {
                    errors.push(TocLoadError::FoundErrorsInSubToc(Box::new(
                        SubTocLoadErrors { errors: sub_errors },
                    )));
                }

                items.push(TocItem {
                    name: name.to_string(),
                    content: TocItemContent::SubToc(Box::new(sub_toc)),
                });
            } else {
                // case 3. Unexpected item
                errors.push(TocLoadError::FoundOddItem(path));
            }
        }

        (Self { items }, errors)
    }
}

/// Item in `toc.ron`
#[derive(Debug)]
pub struct TocItem {
    pub name: String,
    pub content: TocItemContent,
}

#[derive(Debug)]
pub enum TocItemContent {
    File(PathBuf),
    SubToc(Box<Toc>),
}
