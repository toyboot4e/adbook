//! Builtin adbook builder

use {
    anyhow::{Context, Error, Result},
    std::{
        fs,
        io::prelude::*,
        path::{Path, PathBuf},
        process::Command,
    },
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
        Command::new("asciidoctor")
            .output()
            .with_context(|| "Error when trying to validate `asciidoctor` in PATH")?;

        let mut bcx = BuildContext {
            errors: Vec::with_capacity(10),
            book: book.clone(),
            cfg: cfg.clone(),
            out_dir: out_dir.to_path_buf(),
        };

        self.visit_toc(&book.toc, &mut bcx)?;

        Ok(())
    }
}

#[derive(Debug)]
struct BuildContext {
    errors: Vec<Error>,
    book: BookStructure,
    cfg: BuildConfig,
    out_dir: PathBuf,
}

impl BuiltinBookBuilder {
    /// Depth-first walk
    ///
    /// depth-first serach: https://en.wikipedia.org/wiki/Depth-first_search
    fn visit_toc(&mut self, toc: &Toc, bcx: &mut BuildContext) -> Result<()> {
        trace!("visit toc: {}", toc.path.display());

        let parent_dir = toc.path.parent().unwrap();
        for item in &toc.items {
            let res = match item.content {
                TocItemContent::File(ref file) => self.visit_file(file, &parent_dir, bcx),
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
    fn visit_file(&mut self, file: &Path, parent_dir: &Path, bcx: &mut BuildContext) -> Result<()> {
        trace!("visit file: {}", file.display());

        match file.extension().and_then(|o| o.to_str()) {
            Some("adoc") => self.visit_adoc(file, parent_dir, bcx)?,
            Some("md") => {
                bail!(".md file is not yet handled: {}", file.display());
            }
            _ => {
                bail!("Unexpected kind of file: {}", file.display());
            }
        }

        Ok(())
    }

    /// Gets destination path and runs [`self::convert_adoc`]
    fn visit_adoc(
        &mut self,
        src_file: &Path,
        parent_dir: &Path,
        bcx: &mut BuildContext,
    ) -> Result<()> {
        let rel = match src_file.strip_prefix(parent_dir) {
            Ok(r) => r,
            Err(_err) => bail!(
                "Tried to read child item but it was not located relative to parent directory. Item: `{}`, parent dir: `{}`",
                src_file.display(),
                parent_dir.display()
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

        self.convert_adoc(src_file, &dst_file, bcx)?;

        Ok(())
    }

    /// The meat of the builder; it actually converts an `.adoc` file using `asciidoctor` in PATH
    fn convert_adoc(&mut self, src: &Path, dst: &Path, bcx: &mut BuildContext) -> Result<()> {
        trace!(
            "Converting `.adoc` file from `{}` to `{}`",
            src.display(),
            dst.display()
        );

        let mut cmd = Command::new("asciidoctor");

        let src_dir = bcx.book.src_dir_path();
        let src_dir_str = format!("{}", src_dir.display());
        let dst_dir = bcx.book.site_dir_path();
        let dst_dir_str = format!("{}", dst_dir.display());

        // output to stdout
        cmd.current_dir(&src_dir).arg(src).args(&["-o", "-"]);

        cmd.args(&["-B", &src_dir_str])
            .args(&["-D", &dst_dir_str])
            .arg("--trace")
            .arg("-t") // time
            .arg("--no-header-footer")
            .args(&["-a", "noheader"])
            .args(&["-a", "nofooter"]);

        let output = cmd.output().with_context(|| {
            format!(
                "Unexpected error when converting an adoc file:\n  src: {}\n  dst: {}",
                src.display(),
                dst.display()
            )
        })?;

        let mut out = fs::File::create(dst).with_context(|| {
            format!(
                "Unexpected error when trying to write to destination file:\n  {}",
                dst.display(),
            )
        })?;

        // let html_string = String::from_utf8(output.stdout)
        //     .with_context(|| format!(
        //         "Unexpected error when trying to create UTF8 string from the output of asciidoctor run over file:\n  {}",
        //         src.display()
        // ))?;

        out.write_all(&output.stdout)?;

        Ok(())
    }
}
