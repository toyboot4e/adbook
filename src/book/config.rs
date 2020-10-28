//! Configuration types deserialized from `.ron` files

use {
    serde::{Deserialize, Serialize},
    std::path::PathBuf,
};

/// Deserialized from `book.ron` in the root of an `adbook` project
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BookRon {
    /// Use it to supply absolute paths (use `/{base_url/path` instead of `/path`)
    pub base_url: String,
    /// The source directory
    pub src_dir: PathBuf,
    /// The destination directory where source files are converted
    pub site_dir: PathBuf,
    /// Authors of the book
    pub authors: Vec<String>,
    /// Title of the book
    pub title: String,
    /// Files or directories copied to `site` directory
    pub includes: Vec<PathBuf>,
    /// Files to convert but not in sidebar. Typically `404.adoc`
    pub converts: Vec<PathBuf>,
    /// Additional options for `asciidoctor` command
    pub adoc_opts: CmdOptions,
}

/// Deserialized from `toc.ron` in directories in source directory of an `adbook` project
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TocRon {
    /// (name, file) that describes this directory
    pub preface: (String, PathBuf),
    /// Child items
    pub items: Vec<TocRonItem>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum TocRonItem {
    /// (name, url)
    File(String, PathBuf),
    Dir(PathBuf),
}

/// Arguments to a command
///
/// `[("--one-option", ["a", "b"]), ("--another", []), ..]`.
pub type CmdOptions = Vec<(String, Vec<String>)>;
