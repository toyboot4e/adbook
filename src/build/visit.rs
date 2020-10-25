//! Implementation of [`crate::book::walk::BookVisitor`]

use {
    anyhow::{Context, Error, Result},
    std::{
        fs,
        path::{Path, PathBuf},
    },
};

use crate::{
    book::{walk::BookVisitor, BookStructure},
    build::convert::{hbs::HbsContext, AdocRunContext},
};

/// An `adbook` builder based on `asciidoctor`
#[derive(Debug, Clone)]
pub struct AdocBookVisitor {
    buf: String,
    // context to run `asciidoctor` and Handlebars
    acx: AdocRunContext,
    hcx: HbsContext,
    // context to setup output file path
    src_dir: PathBuf,
    dst_dir: PathBuf,
}

impl AdocBookVisitor {
    pub fn from_book(book: &BookStructure, dst_dir: &Path) -> (Self, Vec<Error>) {
        let (hcx, errors) = HbsContext::from_book(book);
        trace!("hcx created: {:#?}", hcx);

        let acx = AdocRunContext::from_book(book, dst_dir);
        trace!("acx created: {:#?}", acx);

        (
            Self {
                buf: String::with_capacity(1024 * 5),
                acx,
                hcx,
                src_dir: book.src_dir_path(),
                dst_dir: dst_dir.to_path_buf(),
            },
            errors,
        )
    }
}

unsafe impl Send for AdocBookVisitor {}

impl BookVisitor for AdocBookVisitor {
    /// Gets destination path and kicks `asciidoctor` runner
    fn visit_file(&mut self, src_file: &Path) -> Result<()> {
        match src_file.extension().and_then(|o| o.to_str()) {
            Some("adoc") => {}
            Some("md") => {
                bail!(".md file is not yet handled: {}", src_file.display());
            }
            _ => {
                bail!("Unexpected kind of file: {}", src_file.display());
            }
        }

        // relative path from source directory
        let rel = match src_file.strip_prefix(&self.src_dir) {
            Ok(r) => r,
            Err(_err) => bail!(
                "Fail that is not in source directly found: {}",
                src_file.display(),
            ),
        };

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

        let dummy_dst_name = format!("{}", dst_file.display());

        crate::build::convert::convert_adoc_buf(
            &mut self.buf,
            src_file,
            &dummy_dst_name,
            &self.acx,
            &self.hcx,
        )?;

        fs::write(&dst_file, &self.buf).with_context(|| {
            format!(
                "Unexpected error when trying to get access to destination file:\n  {}",
                dst_file.display(),
            )
        })?;

        Ok(())
    }
}
