/*!
Configuration types deserialized from `.ron` files

See the [demo files] to know the details.

[demo files]: https://github.com/toyboot4e/adbook/tree/gh-pages
*/

use {
    serde::{Deserialize, Serialize},
    std::path::PathBuf,
};

/// Deserialized from `book.ron` in the root of an `adbook` project
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BookRon {
    /// Use it to supply absolute paths (use `{base_url}/path` instead of `/path`)
    // FIXME: remove the trailing slash on deserializing
    pub base_url: String,
    /// The source directory
    pub src_dir: PathBuf,
    /// The destination directory where source files are converted
    pub site_dir: PathBuf,
    /// Authors of the book
    pub authors: Vec<String>,
    /// Title of the book
    pub title: String,
    /// Files or directories copied to site directory
    pub includes: Vec<PathBuf>,
    /// Whether we copy and use the default `src/theme` directory or not
    pub use_default_theme: bool,
    /// Files to convert but not in sidebar. Typically `404.adoc`
    pub converts: Vec<PathBuf>,
    /// Additional options for `asciidoctor` command
    pub adoc_opts: CmdOptions,
}

/// Deserialized from `toc.ron` in sub directories in a source directory of an `adbook` project
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TocRon {
    /// (name, file) that describes this directory
    pub summary: (String, PathBuf),
    /// Child items
    pub items: Vec<TocRonItem>,
}

/// `File` | `Dir`
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum TocRonItem {
    /// `(title, url)`. If `title` is left as empty (`""`), the sidebar title is extracted from the
    /// source file.
    File(String, PathBuf),
    Dir(PathBuf),
}

/// Arguments to a command
///
/// `[("--one-option", ["a", "b"]), ("--another", []), ..]`.
pub type CmdOptions = Vec<(String, Vec<String>)>;
