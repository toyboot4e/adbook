//! [`BookVisitor`] and a driving procedure of it

use {
    anyhow::Result,
    std::path::{Path, PathBuf},
};

use crate::book::toc::{Toc, TocItemContent};

/// Supplied to [`BookVisitor`]
pub struct BookVisitContext {
    pub errors: Vec<anyhow::Error>,
    pub src_dir: PathBuf,
    pub dst_dir: PathBuf,
}

/// Converts each file in book
pub trait BookVisitor {
    fn visit_file(&mut self, file: &Path, vcx: &mut BookVisitContext) -> Result<()>;
}

/// Walks a root [`Toc`] and converts each file using [`BookVisitor`] into a destination directory
pub fn walk_book(
    v: &mut impl BookVisitor,
    root_toc: &Toc,
    src_dir: &Path,
    dst_dir: &Path,
) -> Result<()> {
    // visit context
    let mut vcx = BookVisitContext {
        errors: Vec::with_capacity(10),
        src_dir: src_dir.to_path_buf(),
        dst_dir: dst_dir.to_path_buf(),
    };

    self::walk_toc(v, root_toc, &mut vcx)?;

    // print errors if any
    crate::utils::print_errors(&vcx.errors, "while building the book");

    Ok(())
}

/// [Depth-first] walk
///
/// [Depth-first]: https://en.wikipedia.org/wiki/Depth-first_search
fn walk_toc(v: &mut impl BookVisitor, toc: &Toc, vcx: &mut BookVisitContext) -> Result<()> {
    trace!("walk toc: {}", toc.path.display());

    for item in &toc.items {
        let res = match item.content {
            TocItemContent::File(ref file) => v.visit_file(file, vcx),
            TocItemContent::SubToc(ref toc) => self::walk_toc(v, toc, vcx),
        };

        match res {
            Ok(_) => {}
            Err(err) => vcx.errors.push(err),
        }
    }

    Ok(())
}
