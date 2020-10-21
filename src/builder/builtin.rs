//! Builtin adbook builder

use {
    anyhow::{Context, Error, Result},
    std::{
        fs,
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
    /// * `opts`: options to `asciidoctor`
    ///
    /// TODO: separate ArticleBuildContext and BookBuildContext
    pub fn single_article(src_file: &Path, site_dir: &Path, opts: CmdOptions) -> Result<Self> {
        use crate::book::config::TocItem;

        let src_dir = src_file
            .parent()
            .ok_or(anyhow!("Given invalid source file path"))?;

        let root_dir = src_dir.to_path_buf();

        // we have to use canoncalized path
        // (or else relative paths are interpreted as relative paths from the root directory)

        let src_dir = src_dir
            .canonicalize()
            .context("Unable to canonicalize source directory path")?;

        let site_dir = site_dir
            .canonicalize()
            .context("Unable to canonicalize site directory path")?;

        let book_ron = BookRon {
            src_dir: src_dir.to_path_buf(),
            site_dir: site_dir.to_path_buf(),
            adoc_opts: opts,
            ..Default::default()
        };

        // dummy
        let toc = Toc {
            path: src_file.parent().unwrap().join("_dummy_toc_.ron"),
            items: vec![TocItem {
                name: "dummy_name".to_string(),
                content: TocItemContent::File(src_file.to_path_buf()),
            }],
        };

        let dummy_book = crate::book::BookStructure {
            root: root_dir,
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
        let dst_name = format!("{}", dst_file.display());
        self.run_asciidoctor_buf(src_file, &dst_name, &mut buf, bcx)?;

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
    /// Sets up `asciidoctor` command
    pub fn asciidoctor(&mut self, src_file: &Path, bcx: &mut BuildContext) -> Result<Command> {
        ensure!(
            src_file.exists(),
            "Given non-existing file as conversion source"
        );

        let src_file = if src_file.is_absolute() {
            src_file.to_path_buf()
        } else {
            src_file
                .canonicalize()
                .with_context(|| "Unable to canonicallize source file path")?
        };

        let mut cmd = Command::new("asciidoctor");

        // output to stdout
        cmd.arg(&src_file).args(&["-o", "-"]);

        // prefer verbose output
        cmd.arg("--trace").arg("--verbose");

        // setup directory settings (base (source)/destination directory)
        {
            let src_dir = bcx.book.src_dir_path();
            let dst_dir = bcx.book.site_dir_path();

            let src_dir_str = format!("{}", src_dir.display());
            let dst_dir_str = format!("{}", dst_dir.display());

            cmd.current_dir(&src_dir)
                .args(&["-B", &src_dir_str])
                .args(&["-D", &dst_dir_str]);
        }

        // apply user options (often ones defined in `book.ron`)
        bcx.apply_adoc_opts(&mut cmd);

        Ok(cmd)
    }

    /// Runs `asciidoctor` and returns the output
    ///
    /// * `src`: source file path
    /// * `dummy_dst_name`: only for debug log
    /// * `out`: actuall handle to destination
    /// * `bcx`: command line arguments and source/site directory paths
    pub fn run_asciidoctor(
        &mut self,
        src_file: &Path,
        dummy_dst_name: &str,
        bcx: &mut BuildContext,
    ) -> Result<std::process::Output> {
        trace!(
            "Converting adoc: `{}` -> `{}`",
            src_file.display(),
            dummy_dst_name,
        );

        let mut cmd = self
            .asciidoctor(src_file, bcx)
            .context("when setting up `asciidoctor` options")?;

        let output = cmd.output().with_context(|| {
            format!(
                "when running `asciidoctor`:\n  src: {}\n  dst: {}\n  cmd: {:?}",
                src_file.display(),
                dummy_dst_name,
                cmd
            )
        })?;

        Ok(output)
    }

    /// Runs `asciidoctor` and write the output to buffer if it's suceceded
    pub fn run_asciidoctor_buf(
        &mut self,
        src_file: &Path,
        dummy_dst_name: &str,
        out: &mut String,
        bcx: &mut BuildContext,
    ) -> Result<()> {
        let output = self.run_asciidoctor(src_file, dummy_dst_name, bcx)?;

        // ensure the conversion succeeded or else report it as an error
        ensure!(
            output.status.success(),
            BuildError::FailedToConvert(
                src_file.to_path_buf(),
                String::from_utf8(output.stderr)
                    .unwrap_or("<non-UTF8 stderr by `asciidoctor`>".to_string())
            )
        );

        // finally output to the buffer
        let text = std::str::from_utf8(&output.stdout)
            .with_context(|| "Unable to decode stdout of `asciidoctor` as UTF8")?;
        out.push_str(text);

        // stderr
        if !output.stderr.is_empty() {
            eprintln!("stderr:");
            let err = String::from_utf8(output.stderr)
                .unwrap_or("<non-UTF8 stderr by `asciidoctor`>".to_string());
            eprintln!("{}", &err);
        }

        Ok(())
    }
}
