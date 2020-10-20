//! Builtin adbook builder

use {
    anyhow::{Context, Error, Result},
    std::{
        fs,
        io::prelude::*,
        path::{Path, PathBuf},
        process::Command,
    },
    thiserror::Error,
};

use crate::{
    book::{
        config::{BookRon, CmdOptions, Toc, TocItemContent},
        BookStructure,
    },
    builder::BookBuilder,
};

/// Builtin adbook builder
pub struct BuiltinBookBuilder {}

impl BuiltinBookBuilder {
    pub fn new() -> Self {
        Self {}
    }
}

impl BookBuilder for BuiltinBookBuilder {
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

/// Context for running a book/article builder
#[derive(Debug)]
pub struct BuildContext {
    errors: Vec<Error>,
    book: BookStructure,
    out_dir: PathBuf,
}

impl BuildContext {
    /// Creates a context for building an article
    ///
    /// * `src`: source file path
    /// * `site_dir`: destination directory path
    pub fn single_article(src: &Path, site_dir: &Path, opts: CmdOptions) -> Result<Self> {
        use crate::book::config::TocItem;

        let src_dir = src.parent().ok_or(anyhow!("invalid dst"))?;

        let book_ron = BookRon {
            src_dir: src_dir.to_path_buf(),
            site_dir: site_dir.to_path_buf(),
            adoc_opts: opts,
            ..Default::default()
        };

        let toc = Toc {
            path: src.parent().unwrap().join("_dummy_toc_.ron"),
            items: vec![TocItem {
                name: "dummy_name".to_string(),
                content: TocItemContent::File(src.to_path_buf()),
            }],
        };

        let dummy_book = crate::book::BookStructure {
            root: src.to_path_buf(),
            book_ron,
            toc,
        };

        Ok(Self {
            errors: Vec::with_capacity(10),
            book: dummy_book,
            out_dir: site_dir.to_path_buf(),
        })
    }

    /// Applies `asciidoctor` options listed in `book.ron` to [`std::proces::Command`]
    pub fn apply_adoc_opts(&self, cmd: &mut Command) {
        let src_dir = self.book.src_dir_path();
        let src_dir_str = format!("{}", src_dir.display());

        for (opt, args) in &self.book.book_ron.adoc_opts {
            if args.is_empty() {
                cmd.arg(opt);
            } else {
                // translated as (opt, arg)+
                for arg in args {
                    // setup placeholder string
                    let arg = arg.replace(r#"${src_dir}"#, &src_dir_str);

                    cmd.args(&[opt, &arg]);
                }
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("Failed to convert file: {0}\n{1}")]
    FailedToConvert(PathBuf, String),
}

impl BuiltinBookBuilder {
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
        self.run_asciidoctor_to_buf(src_file, &dst_file, &mut buf, bcx)?;

        fs::write(&dst_file, &buf).with_context(|| {
            format!(
                "Unexpected error when trying to get access to destination file:\n  {}",
                dst_file.display(),
            )
        })?;

        Ok(())
    }
}

impl BuiltinBookBuilder {
    /// The meat of the builder; it actually converts an `.adoc` file using `asciidoctor` in PATH
    /// and outputs to given [`String`] buffer
    ///
    /// * `src`: source file path
    /// * `dst`: destination path, metadata, not actually used for conversion
    /// * `out`: actuall handle to destination
    /// * `bcx`: command line arguments and source/site directory paths
    pub fn run_asciidoctor_to_buf(
        &mut self,
        src: &Path,
        dst: &Path,
        out: &mut String,
        bcx: &mut BuildContext,
    ) -> Result<()> {
        trace!(
            "Converting adoc: `{}` -> `{}`",
            src.display(),
            dst.display()
        );

        let mut cmd = {
            let mut cmd = Command::new("asciidoctor");

            let src_dir = bcx.book.src_dir_path();
            let src_dir_str = format!("{}", src_dir.display());
            let dst_dir = bcx.book.site_dir_path();
            let dst_dir_str = format!("{}", dst_dir.display());

            // output to stdout
            cmd.arg(src).args(&["-o", "-"]);

            // setup directory settings (base (source)/destination directory)
            cmd.current_dir(&src_dir)
                .args(&["-B", &src_dir_str])
                .args(&["-D", &dst_dir_str]);

            cmd.arg("--trace").arg("--verbose");

            // use "embedded" document without frame
            // REMARK: it doesn't contain revdate, author name etc.
            // TODO: collect AsciiDoc attributes and create header manually
            // (maybe using a templating engine)
            // cmd.arg("--no-header-footer");
            // cmd.args(&["-a", "showtitle"]);

            // apply options set by user (`book.ron`)
            bcx.apply_adoc_opts(&mut cmd);

            cmd
        };

        let output = cmd.output().with_context(|| {
            format!(
                "Error while running `asciidoctor`:\n  src: {}\n  dst: {}",
                src.display(),
                dst.display()
            )
        })?;

        // ensure the conversion succeeded or else report is as an error
        ensure!(
            output.status.success(),
            BuildError::FailedToConvert(
                src.to_path_buf(),
                String::from_utf8(output.stderr)
                    .unwrap_or("<undecodable UTF8 output by asciidoctor?".to_string())
            )
        );

        // TODO: templating

        let text = std::str::from_utf8(&output.stdout)
            .with_context(|| "Unable to decode asciidoctor output to UTF8")?;
        out.push_str(text);

        Ok(())
    }
}
