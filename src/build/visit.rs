/*!
Implementation of [`crate::book::walk::BookVisitor`]

Maybe support more kinds of source files in the future?
*/

use {
    anyhow::{Context, Error, Result},
    std::{
        fs,
        io::prelude::*,
        path::{Path, PathBuf},
    },
};

use crate::{
    book::{walk::BookVisitor, BookStructure},
    build::{
        cache::CacheDiff,
        convert::{hbs::HbsContext, AdocRunContext},
    },
};

/// An `adbook` builder based on `asciidoctor`
#[derive(Debug, Clone)]
pub struct AdocBookVisitor {
    book: BookStructure,
    pub cache: CacheDiff,
    buf: String,
    // context to run `asciidoctor` and Handlebars
    acx: AdocRunContext,
    hcx: HbsContext,
    // context to setup output file path
    src_dir: PathBuf,
    dst_dir: PathBuf,
}

impl AdocBookVisitor {
    pub fn from_book(book: &BookStructure, dst_dir: &Path) -> Result<(Self, Vec<Error>)> {
        let cache = CacheDiff::load(book)?;

        let (hcx, errors) = HbsContext::from_book(book);
        trace!("handlebars context created");
        // trace!("{:#?}", hcx);

        let acx = AdocRunContext::from_book(book, dst_dir);
        trace!("asciidoc context created");
        // trace!("{:#?}", acx);

        Ok((
            Self {
                book: book.clone(),
                cache,
                buf: String::with_capacity(1024 * 5),
                acx,
                hcx,
                src_dir: book.src_dir_path(),
                dst_dir: dst_dir.to_path_buf(),
            },
            errors,
        ))
    }

    fn create_dst_file(&mut self, src_file: &Path) -> Result<PathBuf> {
        // filter files by extension
        match src_file.extension().and_then(|o| o.to_str()) {
            Some("adoc") => {}
            Some("md") => {
                bail!(".md file is not yet supported: {}", src_file.display());
            }
            Some("org") => {
                bail!(".org file is not yet supported: {}", src_file.display());
            }
            Some("txt") => {
                bail!(".txt file is not yet supported: {}", src_file.display());
            }
            Some("html") => {
                bail!(".html file is not yet supported: {}", src_file.display());
            }
            _ => {
                bail!("Unexpected kind of file: {}", src_file.display());
            }
        }

        // get relative path from source directory
        let rel = src_file
            .strip_prefix(&self.src_dir)
            .with_context(|| format!("File not in source directly: {}", src_file.display()))?;

        let dst_file = self.dst_dir.join(&rel).with_extension("html");

        let dst_dir = dst_file.parent().with_context(|| {
            format!(
                "Failed to get parent directory of `.adoc` file: {}",
                src_file.display()
            )
        })?;

        if !dst_dir.is_dir() {
            fs::create_dir_all(&dst_dir).with_context(|| {
                format!(
                    "Failed to create parent directory of `.adoc` file: {}",
                    src_file.display(),
                )
            })?;
        }

        Ok(dst_file)
    }

    fn convert_file_into_buf(&mut self, src_file: &Path, dst_file: &Path) -> Result<()> {
        let dst_name_for_debug = format!("{}", dst_file.display());

        crate::build::convert::convert_adoc_buf(
            &mut self.buf,
            src_file,
            &dst_name_for_debug,
            &self.acx,
            &self.hcx,
        )?;

        Ok(())
    }
}

unsafe impl Send for AdocBookVisitor {}

impl BookVisitor for AdocBookVisitor {
    /// Needs rebuild or we can just copy?
    fn can_skip_build(&self, src_file: &Path) -> bool {
        !self.cache.need_build(&self.book, src_file)
    }

    /// Build or just copy the source file.
    ///
    /// * `src_file`: absolute path to a source file
    fn visit_file(&mut self, src_file: &Path) -> Result<()> {
        let dst_file = self.create_dst_file(src_file)?;
        if self.can_skip_build(src_file) {
            // just copy
            self.buf.clear();
            let mut f = fs::File::open(src_file)?;
            // log::trace!("- skip: {}", src_file.display());
            f.read_to_string(&mut self.buf)?;
        } else {
            // convert
            log::trace!("- convert: {}", src_file.display());
            self.convert_file_into_buf(src_file, &dst_file)?;
        }

        fs::write(&dst_file, &self.buf).with_context(|| {
            format!(
                "Unexpected error when trying to get access to destination file:\n  {}",
                dst_file.display(),
            )
        })?;

        Ok(())
    }
}
