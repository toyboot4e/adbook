/*!
Book builder
*/

use {
    anyhow::*,
    indicatif::{ProgressBar, ProgressStyle},
    std::{
        path::{Path, PathBuf},
        sync::{Arc, Mutex},
    },
};

use crate::book::{
    index::{Index, IndexItem},
    BookStructure,
};

/// Converter of each source file in the book
pub trait BookBuilder: Clone + Send + Sync {
    /// Can we just copy this file from the previous build?
    ///
    /// * `src_file`: absolute path to a source file
    fn can_skip_build(&self, src_file: &Path) -> bool;
    /// Build or just copy the source file.
    ///
    /// * `src_file`: absolute path to a source file
    fn visit_file(&mut self, src_file: &Path) -> Result<()>;
}

fn list_src_files(book: &BookStructure) -> Vec<PathBuf> {
    // note that paths in `Index` are already canonicalized (can can be passed to visitors directly)

    /// [Depth-first] iteration
    ///
    /// [Depth-first]: https://en.wikipedia.org/wiki/Depth-first_search
    fn list_files_rec(index: &Index, files: &mut Vec<PathBuf>) {
        files.push(index.summary.clone());
        for item in &index.items {
            match item {
                IndexItem::File(_name, path) => {
                    files.push(path.clone());
                }
                IndexItem::Dir(index) => {
                    list_files_rec(index, files);
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

    // `index.ron` files
    list_files_rec(&book.index, &mut files);

    files
}

/// Walks a root [`Index`] and converts files in parallel
///
/// NOTE: make sure to `flush` after calling this method
pub async fn walk_book_async<V: BookBuilder + 'static>(v: &mut V, book: &BookStructure, log: bool) {
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

    if filtered.is_empty() {
        if log {
            println!("No file to build");
        }
        return;
    }

    // progress bar
    let pb = {
        let pb = ProgressBar::new(filtered.len() as u64);

        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                .progress_chars("##-"),
        );

        // show progress bar
        pb.inc(0);

        Arc::new(Mutex::new(pb))
    };

    let results = {
        let xs = filtered
            .into_iter()
            .map(|src_file| {
                // TODO: maybe don't clone?
                let mut v = v.clone();
                let pb = Arc::clone(&pb);
                async_std::task::spawn(async move {
                    let res = v.visit_file(&src_file);

                    let pb = pb.lock().expect("unable to lock progress bar");
                    pb.inc(1);

                    res
                })
            })
            .collect::<Vec<_>>();

        futures::future::join_all(xs).await
    };

    for err in results.into_iter().filter_map(|x| x.err()) {
        errors.push(err);
    }

    crate::utils::print_errors(&errors, "while building the book");

    let pb = pb.lock().expect("unable to lock progress bar");
    if log {
        let elasped = pb.elapsed();
        let msg = format!("{:.2} seconds", elasped.as_secs_f32());
        pb.finish_with_message(msg);
    } else {
        pb.finish();
    }
}
