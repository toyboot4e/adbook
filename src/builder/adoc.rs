//! Builtin adbook renderer for AsciiDoc

use anyhow::Result;

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
    fn build_book(&mut self, book: &BookStructure, cfg: &BuildConfig) -> Result<()> {
        //

        Ok(())
    }
}
