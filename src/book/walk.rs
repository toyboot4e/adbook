/*!
Book builder
*/

use std::{
    fmt,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use indicatif::{ProgressBar, ProgressStyle};

use crate::book::{
    index::{Index, IndexItem},
    BookStructure,
};

/// Converter of each source file in the book
pub trait BookBuilder: Clone + Send + Sync {
    /// Can we just copy this file from the previous build?
    ///
    /// * `src_file`: canonizalied path to a source file
    fn can_skip_build(&self, src_file: &Path) -> bool;
    /// Build or just copy the source file.
    ///
    /// * `src_file`: canonizalied path to a source file
    fn convert_file(&mut self, src_file: &Path) -> BuildResult;
}

pub type BuildResult = std::result::Result<BuildOutput, BuildError>;

/// Output string + metadata
#[derive(Debug, Clone)]
pub struct BuildOutput {
    pub string: String,
    pub src_file: PathBuf,
}

/// Error + metadata
#[derive(Debug)]
pub struct BuildError {
    pub err: anyhow::Error,
    /// Absolute path to the source file
    pub src_file: PathBuf,
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.err.fmt(f)
    }
}

pub fn walk_book_await_collect<V: BookBuilder + 'static>(
    builder: &mut V,
    book: &BookStructure,
    log: bool,
) -> Vec<BuildOutput> {
    let results = futures::executor::block_on(walk_book_async(builder, &book, log));

    let mut outputs = Vec::new();
    let mut errors = Vec::new();

    for res in results {
        match res {
            Ok(output) => outputs.push(output),
            Err(err) => errors.push(err.err),
        }
    }

    crate::utils::print_errors(&errors, "while building the book");

    outputs
}

/// Walks a root [`Index`] and converts files in parallel
///
/// NOTE: Make sure to `flush` after calling this method so that the user can read log output.
pub async fn walk_book_async<V: BookBuilder + 'static>(
    builder: &mut V,
    book: &BookStructure,
    log: bool,
) -> Vec<BuildResult> {
    let src_files_unfiltered = self::list_src_files(&book);

    let mut can_skip_all = false;

    let src_files = src_files_unfiltered
        .into_iter()
        .map(|src_file| {
            can_skip_all |= !builder.can_skip_build(&src_file);
            src_file
        })
        .collect::<Vec<_>>();

    if !can_skip_all || src_files.is_empty() {
        if log {
            println!("No file to build");
        }
        return Vec::new();
    }

    // progress bar
    let pb = {
        let pb = ProgressBar::new(src_files.len() as u64);

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
        let tasks = src_files
            .into_iter()
            .map(|src_file| {
                let mut builder = builder.clone();
                let pb = Arc::clone(&pb);

                async_std::task::spawn(async move {
                    let res = builder.convert_file(&src_file);

                    let pb = pb.lock().expect("unable to lock progress bar");
                    pb.inc(1);

                    res
                })
            })
            .collect::<Vec<_>>();

        futures::future::join_all(tasks).await
    };

    let pb = pb.lock().expect("unable to lock progress bar");
    if log {
        let elasped = pb.elapsed();
        let msg = format!("{:.2} seconds", elasped.as_secs_f32());
        pb.finish_with_message(msg);
    } else {
        pb.finish();
    }

    results
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
