//! `asciidoctor` runner

use {
    anyhow::{Context, Error, Result},
    std::{
        path::{Path, PathBuf},
        process::Command,
    },
    thiserror::Error,
};

use crate::book::{
    config::{BookRon, CmdOptions, Toc, TocItemContent},
    BookStructure,
};

/// Structure for error printing
///
/// TODO: refactor and prefer it to anyhow::Error
#[derive(Debug, Error)]
pub enum BuildError {
    #[error("Failed to convert file: {0}\n{1}")]
    FailedToConvert(PathBuf, String),
}

/// Context for running a book/article builder
///
/// TODO: separate ArticleBuildContext and BookBuildContext
#[derive(Debug)]
pub struct BuildContext {
    pub errors: Vec<Error>,
    pub book: BookStructure,
    pub out_dir: PathBuf,
}

impl BuildContext {
    /// Creates a context for building an article
    ///
    /// * `src`: source file path
    /// * `site_dir`: destination directory path
    /// * `opts`: options to `asciidoctor`
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

    /// Applies `asciidoctor` options listed in `book.ron` to [`std::process::Command`]
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

/// Sets up `asciidoctor` command
pub fn asciidoctor(src_file: &Path, bcx: &mut BuildContext) -> Result<Command> {
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
    src_file: &Path,
    dummy_dst_name: &str,
    bcx: &mut BuildContext,
) -> Result<std::process::Output> {
    trace!(
        "Converting adoc: `{}` -> `{}`",
        src_file.display(),
        dummy_dst_name,
    );

    let mut cmd =
        self::asciidoctor(src_file, bcx).context("when setting up `asciidoctor` options")?;

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
    src_file: &Path,
    dummy_dst_name: &str,
    out: &mut String,
    bcx: &mut BuildContext,
) -> Result<()> {
    let output = self::run_asciidoctor(src_file, dummy_dst_name, bcx)?;

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
