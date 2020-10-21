/*! Configuration types deserialized from `.ron` files

* `book.ron`: A root file of an adbook project. Corresponds to `lib.rs` in Rust or `book.toml` in
mdbook.
* `toc.ron`: A file that lists child items in a source directory. Corresponds to `mod.rs` in Rust or
`SUMMARY.md` in mdbook.
!*/

use {
    serde::{Deserialize, Serialize},
    std::path::PathBuf,
};

/// Arguments to a command
pub type CmdOptions = Vec<(String, Vec<String>)>;

/// Deserialized from `book.ron` in the root of an `adbook` project
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct BookRon {
    /// Authors of the book
    pub authors: Vec<String>,
    /// Title of the book
    pub title: String,
    /// The source directory
    pub src_dir: PathBuf,
    /// The destination directory where source files are converted
    pub site_dir: PathBuf,
    /// Files or directories copied to `site` directory
    pub includes: Vec<PathBuf>,
    /// Additional options for `asciidoctor` command
    pub adoc_opts: CmdOptions,
}

/// Deserialized from `toc.ron` in directories in source directory of an `adbook` project
#[derive(Deserialize, Serialize, Debug)]
pub struct TocRon {
    pub items: Vec<(String, String)>,
}
