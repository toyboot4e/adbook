//! Builtin adbook renderer for AsciiDoc

use {
    anyhow::{Context, Result},
    std::{fs, path::Path},
};

use crate::{
    book::BookStructure,
    builder::{BookBuilder, BuildConfig},
};

pub struct AdocBuilder {}

impl AdocBuilder {
    pub fn new() -> Self {
        Self {}
    }
}

impl BookBuilder for AdocBuilder {
    fn build_book_to_tmp_dir(
        &mut self,
        book: &BookStructure,
        cfg: &BuildConfig,
        out_dir: &Path,
    ) -> Result<()> {
        Ok(())
    }
}
