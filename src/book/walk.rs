//! [`BookVisitor`] and a driving procedure of it

use {
    anyhow::Result,
    indicatif::{ProgressBar, ProgressStyle},
    std::{
        path::{Path, PathBuf},
        sync::{Arc, Mutex},
    },
};

use crate::book::{
    toc::{Toc, TocItem},
    BookStructure,
};

/// Converts each file in book
pub trait BookVisitor: Clone + Send + Sync {
    /// Needs rebuild or we can just copy?
    fn can_skip_build(&self, src_file: &Path) -> bool;
    /// Build or just copy the source file.
    ///
    /// * `src_file`: absolute path to a source file
    fn visit_file(&mut self, src_file: &Path) -> Result<()>;
}

fn list_src_files(book: &BookStructure) -> Vec<PathBuf> {
    // note that paths in `Toc` are already canonicalized (can can be passed to visitors directly)

    /// [Depth-first] iteration
    ///
    /// [Depth-first]: https://en.wikipedia.org/wiki/Depth-first_search
    fn list_files_rec(toc: &Toc, files: &mut Vec<PathBuf>) {
        files.push(toc.summary.clone());
        for item in &toc.items {
            match item {
                TocItem::File(_name, path) => {
                    files.push(path.clone());
                }
                TocItem::Dir(toc) => {
                    list_files_rec(toc, files);
                }
            };
        }
    }

    let mut files = Vec::with_capacity(80);

    // converts
    let src_dir = book.src_dir_path();
    for p in &book.book_ron.converts {
        let path = src_dir.join(p);
        files.push(path);
    }

    // `toc.ron` files
    list_files_rec(&book.toc, &mut files);

    files
}

// /// Walks a root [`Toc`] and converts files one by one
// pub fn walk_book(v: &mut impl BookVisitor, book: &BookStructure) {
//     let files = self::list_src_files(&book);
//     let results = files.iter().map(|file| v.visit_file(file));
//
//     let errors: Vec<_> = results
//         .into_iter()
//         .filter(|x| x.is_err())
//         .map(|x| x.unwrap_err())
//         .collect();
//
//     // print errors if any
//     crate::utils::print_errors(&errors, "while building the book");
// }

/// Walks a root [`Toc`] and converts files in parallel
pub async fn walk_book_async<V: BookVisitor + 'static>(v: &mut V, book: &BookStructure) {
    let src_files = self::list_src_files(&book);

    // collect `Future`s
    let mut errors = Vec::with_capacity(16);

    let filtered = src_files
        .into_iter()
        .filter(|src_file| {
            // TODO: maybe don't clone?
            let mut v = v.clone();
            if v.can_skip_build(&src_file) {
                if let Err(err) = v.visit_file(&src_file) {
                    errors.push(err);
                }
                false
            } else {
                true
            }
        })
        .collect::<Vec<_>>();

    // progress bar
    let pb = ProgressBar::new(filtered.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .progress_chars("##-"),
    );

    let pb = Arc::new(Mutex::new(pb));
    let xs = filtered
        .into_iter()
        .map(|src_file| {
            // TODO: maybe don't clone?
            let mut v = v.clone();
            let pb = Arc::clone(&pb);
            async_std::task::spawn(async move {
                let res = v.visit_file(&src_file);
                pb.lock().expect("unable to get progress bar").inc(1);
                res
            })
        })
        .collect::<Vec<_>>();

    let results = futures::future::join_all(xs).await;

    for err in results.into_iter().filter_map(|x| x.err()) {
        errors.push(err);
    }

    crate::utils::print_errors(&errors, "while building the book");
}
