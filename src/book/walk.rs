//! [`BookVisitor`] and a driving procedure of it

use {
    anyhow::Result,
    std::path::{Path, PathBuf},
};

use crate::book::toc::{Toc, TocItemContent};

/// Supplied to [`BookVisitor`]
#[derive(Debug, Clone)]
pub struct BookVisitContext {
    pub src_dir: PathBuf,
    /// Context to mirror source files to destination
    pub dst_dir: PathBuf,
}

/// Converts each file in book
pub trait BookVisitor: Clone + Send + Sync {
    fn visit_file(&mut self, file: &Path, vcx: &BookVisitContext) -> Result<()>;
}

/// [Depth-first] iteration
///
/// [Depth-first]: https://en.wikipedia.org/wiki/Depth-first_search
pub fn pull_files_rec(toc: &Toc, xs: &mut Vec<PathBuf>) {
    for item in &toc.items {
        match item.content {
            TocItemContent::File(ref file) => {
                xs.push(file.clone());
            }
            TocItemContent::SubToc(ref toc) => {
                self::pull_files_rec(toc, xs);
            }
        };
    }
}

/// Walks a root [`Toc`] and converts files one by one
pub fn walk_book(v: &mut impl BookVisitor, root_toc: &Toc, src_dir: &Path, dst_dir: &Path) {
    let mut files = Vec::with_capacity(80);
    self::pull_files_rec(root_toc, &mut files);

    let vcx = BookVisitContext {
        src_dir: src_dir.to_path_buf(),
        dst_dir: dst_dir.to_path_buf(),
    };

    let results = files.iter().map(|file| v.visit_file(file, &vcx));

    let errors: Vec<_> = results
        .into_iter()
        .filter(|x| x.is_err())
        .map(|x| x.unwrap_err())
        .collect();

    // print errors if any
    crate::utils::print_errors(&errors, "while building the book");
}

/// Walks a root [`Toc`] and converts files in parallel
pub async fn walk_book_async<V: BookVisitor + 'static>(
    v: &mut V,
    root_toc: &Toc,
    src_dir: &Path,
    dst_dir: &Path,
) {
    let mut files = Vec::with_capacity(80);
    self::pull_files_rec(root_toc, &mut files);

    // collect `Future`s
    let xs = files.into_iter().map(|file| {
        let mut v = v.clone();
        let vcx = BookVisitContext {
            src_dir: src_dir.to_path_buf(),
            dst_dir: dst_dir.to_path_buf(),
        };
        // async move { v.visit_file(&file, &vcx) }
        async_std::task::spawn(async move { v.visit_file(&file, &vcx) })
    });

    let results = futures::future::join_all(xs).await;

    let errors: Vec<_> = results
        .into_iter()
        .filter(|x| x.is_err())
        .map(|x| x.unwrap_err())
        .collect();

    crate::utils::print_errors(&errors, "while building the book");
}
