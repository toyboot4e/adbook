/*! Book data structure

It's based on two kinds of files: `book.ron` and `toc.ron` (toc standing for table of contents).
Those files are written in the human-friendly [Ron] format.

[Ron]: https://github.com/ron-rs/ron

# `book.ron`

An adbook project has such a file structure:

```sh
.
├── book.ron  # configuration file in root
├── site      # `.html` files
└── src       # source files
```

`book.ron` maps to [`BookRon`]. It's indicates a root directory and provides some configuration such
as the book name and the author name.

[`BookRon`]: crate::book::config::BookRon

# `toc.ron`

When you run `adbook build`, it will look into `src/toc.ron` and searches files or sub directroes in
it, recursively:

```sh
└── src
    ├── a.adoc
    ├── sub_directory
    │   ├── b.adoc
    │   └── toc.ron  # lists `b.adoc`
    └── toc.ron      # lists `a.adoc` and `sub_directory`
```

`toc.ron` maps to [`TocRon`]. It's similar to `mod.rs` in Rust.

[`TocRon`]: crate::book::config::TocRon
!*/

pub mod config;
pub mod preset;
pub mod toc;
pub mod walk;

use {
    anyhow::{Context, Result},
    std::{
        fs,
        path::{Path, PathBuf},
    },
    thiserror::Error,
};

use self::{
    config::{BookRon, TocRon},
    toc::Toc,
};

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
    /// `book.ron`
    pub book_ron: BookRon,
    /// `src/toc.ron`
    pub toc: Toc,
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

        let src_dir = root.join(&book_ron.src_dir);

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

        crate::utils::print_errors(&toc_errors, "while parsing toc.ron");

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
