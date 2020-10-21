//! Book builder

pub mod visit;
pub mod walk;

use anyhow::Result;

use crate::{book::BookStructure, build::visit::AdocBookVisitor};

/// Builds an `adbook` project with a configuration
pub fn build_book(book: &BookStructure) -> Result<()> {
    let mut builder = AdocBookVisitor::new(book.book_ron.adoc_opts.clone());
    self::walk::build_book(&mut builder, book)
}
