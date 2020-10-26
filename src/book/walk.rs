//! [`BookVisitor`] and a driving procedure of it

use {
    anyhow::Result,
    std::path::{Path, PathBuf},
};

use crate::book::toc::{Toc, TocItem};

/// Converts each file in book
pub trait BookVisitor: Clone + Send + Sync {
    fn visit_file(&mut self, file: &Path) -> Result<()>;
}

/// [Depth-first] iteration
///
/// [Depth-first]: https://en.wikipedia.org/wiki/Depth-first_search
pub fn pull_files_rec(toc: &Toc, files: &mut Vec<PathBuf>) {
    files.push(toc.preface.clone());
    for item in &toc.items {
        match item {
            TocItem::File(_name, path) => {
                files.push(path.clone());
            }
            TocItem::Dir(toc) => {
                self::pull_files_rec(toc, files);
            }
        };
    }
}

/// Walks a root [`Toc`] and converts files one by one
pub fn walk_book(v: &mut impl BookVisitor, root_toc: &Toc) {
    let mut files = Vec::with_capacity(80);
    self::pull_files_rec(root_toc, &mut files);

    let results = files.iter().map(|file| v.visit_file(file));

    let errors: Vec<_> = results
        .into_iter()
        .filter(|x| x.is_err())
        .map(|x| x.unwrap_err())
        .collect();

    // print errors if any
    crate::utils::print_errors(&errors, "while building the book");
}

/// Walks a root [`Toc`] and converts files in parallel
pub async fn walk_book_async<V: BookVisitor + 'static>(v: &mut V, root_toc: &Toc) {
    let mut files = Vec::with_capacity(80);
    self::pull_files_rec(root_toc, &mut files);

    // collect `Future`s
    let xs = files.into_iter().map(|file| {
        let mut v = v.clone();
        // async move { v.visit_file(&file, &vcx) }
        async_std::task::spawn(async move { v.visit_file(&file) })
    });

    let results = futures::future::join_all(xs).await;

    let errors: Vec<_> = results
        .into_iter()
        .filter(|x| x.is_err())
        .map(|x| x.unwrap_err())
        .collect();

    crate::utils::print_errors(&errors, "while building the book");
}
