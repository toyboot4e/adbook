/*! `adbook` configuration deserialized from `.ron` files

# Configuration files

* `book.ron`: Something like `Cargo.toml`, a root file for an adbook project
* `toc.ron`: Something like `mod.rs`, which lists `(name, path)` pairs

ToC stands for table of contents.
!*/

use {
    serde::{Deserialize, Serialize},
    std::{
        fmt, fs, io,
        path::{Path, PathBuf},
    },
    thiserror::Error,
};

/// Arguments to a command
pub type CmdOptions = Vec<(String, Vec<String>)>;

// --------------------------------------------------------------------------------
// Directly deserialized from ron files

/// Deserialized from `book.ron`
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct BookRon {
    /// Authors of the book
    pub authors: Vec<String>,
    /// Title of the book
    pub title: String,
    /// The source directory
    pub src: PathBuf,
    /// The destination directory where source files are converted
    pub site: PathBuf,
    /// Files or directories copied to `site` directory
    pub includes: Vec<PathBuf>,
    /// Additional options for `asciidoctor` command
    pub adoc_opts: CmdOptions,
}

/// Deserialized from `toc.ron`
#[derive(Deserialize, Serialize, Debug)]
pub struct TocRon {
    pub items: Vec<(String, String)>,
}

// --------------------------------------------------------------------------------
// Retrieved from ron struct

/// Error when loading `toc.ron`
#[derive(Debug, Error)]
pub enum TocLoadError {
    /// (relative_path_to_the_file, book_ron_directory_path)
    #[error("Unable to locate `{0}` in `{1}`")]
    FailedToLocateItem(PathBuf, PathBuf),
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

/// Errors when loading a sub `toc.ron`
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

/// Got from [`TocRon`], which is deserialiezd from `toc.ron`
#[derive(Debug, Clone)]
pub struct Toc {
    /// Absolute path to the [`toc.ron`]
    pub path: PathBuf,
    pub items: Vec<TocItem>,
}

/// Item in `toc.ron`: (File | SubToc) with name
#[derive(Debug, Clone)]
pub struct TocItem {
    pub name: String,
    pub content: TocItemContent,
}

/// File | SubToc
#[derive(Debug, Clone)]
pub enum TocItemContent {
    /// Absolute path to the file
    File(PathBuf),
    SubToc(Box<Toc>),
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
                errors.push(TocLoadError::FailedToLocateItem(
                    rel_path.into(),
                    toc_ron_dir.to_path_buf(),
                ));
                continue;
            }

            let path = path.canonicalize().unwrap();

            // 3 cases:
            if path.is_file() {
                // case 1. File
                items.push(TocItem {
                    name: name.to_string(),
                    content: TocItemContent::File(path.clone()),
                });
            } else if path.is_dir() {
                // case 2. Directory with `toc.ron`
                let nested_toc_ron = path.join("toc.ron");
                if !nested_toc_ron.is_file() {
                    errors.push(TocLoadError::FoundDirectoryWithoutTocRon(path));
                    continue;
                }

                let toc_ron_str = match fs::read_to_string(&nested_toc_ron) {
                    Ok(s) => s,
                    Err(err) => {
                        errors.push(TocLoadError::FailedToReadTocRon(path, err));
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

        (
            Self {
                path: toc_ron_dir.to_path_buf(),
                items,
            },
            errors,
        )
    }
}
