//! Visitor of book structure and temporary output files

use {
    anyhow::{Context, Result},
    std::{fs, path::Path, process::Command},
};

use crate::{
    book::{
        config::{Toc, TocItemContent},
        BookStructure,
    },
    build::{
        adoc::{self, BuildContext},
        guard::BookBuilder,
    },
};

/// An `adbook` builder based on `asciidoctor`
///
/// * TODO: separate visitor
pub struct AdocBuilder {}

impl AdocBuilder {
    pub fn new() -> Self {
        AdocBuilder {}
    }
}

impl BookBuilder for AdocBuilder {
    fn build_book_to_tmp_dir(&mut self, book: &BookStructure, out_dir: &Path) -> Result<()> {
        Command::new("asciidoctor")
            .output()
            .with_context(|| "Error when trying to validate `asciidoctor` in PATH")?;

        let mut bcx = BuildContext {
            errors: Vec::with_capacity(10),
            book: book.clone(),
            out_dir: out_dir.to_path_buf(),
        };

        self.visit_toc(&book.toc, &mut bcx)?;

        // print errors if any
        crate::utils::print_errors(&bcx.errors, "while building the book");

        Ok(())
    }
}

impl AdocBuilder {
    /// Depth-first walk
    ///
    /// depth-first serach: https://en.wikipedia.org/wiki/Depth-first_search
    fn visit_toc(&mut self, toc: &Toc, bcx: &mut BuildContext) -> Result<()> {
        trace!("visit toc: {}", toc.path.display());

        for item in &toc.items {
            let res = match item.content {
                TocItemContent::File(ref file) => self.visit_file(file, bcx),
                TocItemContent::SubToc(ref toc) => self.visit_toc(toc, bcx),
            };

            match res {
                Ok(_) => {}
                Err(err) => bcx.errors.push(err),
            }
        }

        Ok(())
    }

    /// Tries to convert given file at path
    fn visit_file(&mut self, file: &Path, bcx: &mut BuildContext) -> Result<()> {
        trace!("visit file: {}", file.display());

        // TODO: spawn thread
        match file.extension().and_then(|o| o.to_str()) {
            Some("adoc") => self.visit_adoc(file, bcx)?,
            Some("md") => {
                bail!(".md file is not yet handled: {}", file.display());
            }
            _ => {
                bail!("Unexpected kind of file: {}", file.display());
            }
        }

        Ok(())
    }

    /// Gets destination path and kicks `asciidoctor` runner
    fn visit_adoc(&mut self, src_file: &Path, bcx: &mut BuildContext) -> Result<()> {
        // relative path from source directory
        let rel = match src_file.strip_prefix(bcx.book.src_dir_path()) {
            Ok(r) => r,
            Err(_err) => bail!(
                "Fail that is not in source directly found: {}",
                src_file.display(),
            ),
        };

        let dst_file = bcx.out_dir.join(&rel).with_extension("html");

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

        let mut buf = String::with_capacity(5 * 1024);
        let dst_name = format!("{}", dst_file.display());
        adoc::run_asciidoctor_buf(src_file, &dst_name, &mut buf, bcx)?;

        fs::write(&dst_file, &buf).with_context(|| {
            format!(
                "Unexpected error when trying to get access to destination file:\n  {}",
                dst_file.display(),
            )
        })?;

        Ok(())
    }
}
