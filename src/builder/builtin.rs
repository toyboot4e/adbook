//! Builtin adbook builder

use {
    anyhow::{Context, Result},
    std::{fs, path::Path},
};

use crate::{
    book::{
        config::{Toc, TocItemContent},
        BookStructure,
    },
    builder::{BookBuilder, BuildConfig},
};

/// Builtin adbook builder
pub struct BuiltinBookBuilder {}

impl BuiltinBookBuilder {
    pub fn new() -> Self {
        Self {}
    }
}

impl BookBuilder for BuiltinBookBuilder {
    fn build_book_to_tmp_dir(
        &mut self,
        book: &BookStructure,
        cfg: &BuildConfig,
        out_dir: &Path,
    ) -> Result<()> {
        self.visit_toc(&book.toc, cfg, out_dir)?;

        Ok(())
    }
}

impl BuiltinBookBuilder {
    /// Depth-first walk
    ///
    /// depth-first serach: https://en.wikipedia.org/wiki/Depth-first_search
    fn visit_toc(&mut self, toc: &Toc, cfg: &BuildConfig, out_dir: &Path) -> Result<()> {
        trace!("visit toc: {}", toc.path.display());

        for item in &toc.items {
            match item.content {
                TocItemContent::File(ref file) => self.visit_file(file, cfg, out_dir)?,
                TocItemContent::SubToc(ref toc) => self.visit_toc(toc, cfg, out_dir)?,
            }
        }

        Ok(())
    }

    fn visit_file(&mut self, file: &Path, cfg: &BuildConfig, out_dir: &Path) -> Result<()> {
        trace!("visit file: {}", file.display());

        Ok(())
    }
}
