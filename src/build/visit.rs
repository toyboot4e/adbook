/*!
Implementation of [`crate::book::walk::BookBuilder`]

TODO: Enable other source formats than Asciidoc
*/

use std::{fs, io::prelude::*, path::Path};

use anyhow::{anyhow, Context, Error, Result};

use crate::{
    book::{
        walk::{BookBuilder, BuildError, BuildOutput, BuildResult},
        BookStructure,
    },
    build::{
        cache::{CacheIndex, CacheIndexDiff},
        convert::{hbs::HbsContext, AdocRunContext},
    },
};

/// Book builder based on `asciidoctor`
#[derive(Debug, Clone)]
pub struct AdocBookBuilder {
    book: BookStructure,
    pub(crate) cache_diff: CacheIndexDiff,
    // context to run `asciidoctor` and Handlebars
    acx: AdocRunContext,
    hcx: HbsContext,
}

impl AdocBookBuilder {
    pub fn from_book(
        book: &BookStructure,
        cache_diff: CacheIndexDiff,
    ) -> Result<(Self, Vec<Error>)> {
        let acx = AdocRunContext::from_book(book)?;
        log::trace!("asciidoctor context created");
        // log::trace!("{:#?}", acx);

        let (hcx, errors) = HbsContext::from_book(book);
        log::trace!("handlebars context created");
        // log::trace!("{:#?}", hcx);

        Ok((
            Self {
                book: book.clone(),
                cache_diff,
                acx,
                hcx,
            },
            errors,
        ))
    }

    fn convert_file_into_buf(&mut self, buf: &mut String, src_file: &Path) -> Result<()> {
        crate::build::convert::convert_adoc_buf(buf, src_file, &self.acx, &self.hcx, &self.book)?;
        Ok(())
    }

    fn convert_file_impl(&mut self, src_file: &Path) -> Result<String> {
        let mut buf = String::with_capacity(1024 * 5);

        if self.can_skip_build(src_file) {
            // just copy
            let src_dir = self.book.src_dir_path();
            let rel_path = src_file.strip_prefix(&src_dir)?;

            let cache_dir = CacheIndex::locate_cache_dir(&self.book)?;
            let cached_file = cache_dir.join(rel_path).with_extension("html");

            let mut f = fs::File::open(&cached_file).with_context(|| {
                anyhow!(
                    "Unable to locate cached file at {}\nPlease run `adbook clear`",
                    cached_file.display()
                )
            })?;

            log::trace!("- skip: {}", src_file.display());
            f.read_to_string(&mut buf)?;
        } else {
            // convert
            log::trace!("- convert: {}", src_file.display());
            self.convert_file_into_buf(&mut buf, src_file)?;
        }

        Ok(buf)
    }
}

unsafe impl Send for AdocBookBuilder {}

impl BookBuilder for AdocBookBuilder {
    fn can_skip_build(&self, src_file: &Path) -> bool {
        !self.cache_diff.need_build(&self.book, src_file)
    }

    fn convert_file(&mut self, src_file: &Path) -> BuildResult {
        match self.convert_file_impl(src_file) {
            Ok(output) => Ok(BuildOutput {
                string: output,
                src_file: src_file.to_path_buf(),
            }),
            Err(err) => Err(BuildError {
                err,
                src_file: src_file.to_path_buf(),
            }),
        }
    }
}
