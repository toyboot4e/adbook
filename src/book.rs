/*! Book data structure

It's based on two kinds of files: `book.ron` and `index.ron`. Those files are written in the
human-friendly [Ron] format (NOTE: in `adbook`, it's allowed to omit outermost parentheses in RON
format).

[Ron]: https://github.com/ron-rs/ron

# `book.ron`

An adbook project has such a file structure:

```sh
.
├── book.ron  # configuration file in root
├── site      # `.html` files
└── src       # source files
```

`book.ron` is located at the root and mapped to [`BookRon`]. It indicates a root directory and
metadata such as the book name and the author name.

[`BookRon`]: crate::book::config::BookRon

# `index.ron`

When you run `adbook build`, it will look into `src/index.ron` and searches files or sub directroes in
it, recursively:

```sh
└── src
    ├── a.adoc
    ├── sub_directory
    │   ├── preface.adoc
    │   └── index.ron  # lists `preface.adoc`
    └── index.ron      # lists `a.adoc` and `sub_directory`
```

`index.ron` maps to [`IndexRon`]. It's similar to `mod.rs` in Rust; it's a list of source files in the
directory.
[`IndexRon`]: crate::book::config::IndexRon
!*/

pub mod config;
pub mod index;
pub mod init;
pub mod walk;

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::*;
use thiserror::Error;

use self::{
    config::{BookRon, IndexRon},
    index::Index,
};

const INDEX_RON: &'static str = "index.ron";

/// Error while loading `book.ron`
#[derive(Error, Debug)]
pub enum BookLoadError {
    #[error("Given non-directory path")]
    GivenNonDirectoryPath,
    #[error("Not found root directory (not found `book.ron`)")]
    NotFoundRoot,
}

/// File structure of an adbook project read from `book.ron` and `index.ron`s
#[derive(Debug, Clone)]
pub struct BookStructure {
    /// Absolute path to a directory with `book.ron`
    pub root: PathBuf,
    /// `book.ron`
    pub book_ron: BookRon,
    /// `src/index.ron`, the recursive book structure
    pub index: Index,
}

impl BookStructure {
    pub fn src_dir_path(&self) -> PathBuf {
        self.root.join(&self.book_ron.src_dir)
    }

    pub fn site_dir_path(&self) -> PathBuf {
        self.root.join(&self.book_ron.site_dir)
    }
}

impl BookStructure {
    /// Tries to find `book.ron` going up the directories and parses it into a file structure
    pub fn from_dir(path: impl AsRef<Path>) -> Result<Self> {
        let book_ron_path = self::find_root_book_ron(path)?;
        log::trace!("book.ron located at: {}", book_ron_path.display());

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
            crate::utils::load_ron(&cfg_str).with_context(|| {
                format!("Failed to load book.ron at: {}", book_ron_path.display())
            })?
        };

        log::trace!("root `book.ron` loaded");
        // log::trace!("{:?}", book_ron);

        let src_dir = root.join(&book_ron.src_dir);

        let (index, index_errors) = {
            let index_path = src_dir.join(INDEX_RON);
            let index_str = fs::read_to_string(&index_path).with_context(|| {
                format!(
                    "Unable to read root `index.ron` at: {}",
                    index_path.display()
                )
            })?;

            let index_ron: IndexRon = crate::utils::load_ron(&index_str).with_context(|| {
                format!("Failed to parse `index.ron` at: {}", index_path.display())
            })?;
            log::trace!("root `index.ron` loaded");

            log::trace!("loading `index.ron`");
            Index::from_index_ron_recursive(&index_ron, &src_dir)?
        };

        log::trace!("`index.ron` loaded");

        crate::utils::print_errors(&index_errors, "while parsing `index.ron`");

        Ok(Self {
            root,
            book_ron,
            index,
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
