//! `asciidoctor` runner

use {
    anyhow::{Context, Error, Result},
    std::{
        io,
        path::{Path, PathBuf},
        process::Command,
    },
    thiserror::Error,
};

use crate::book::config::CmdOptions;

// TODO:
// pub type Result<T> = std::result::Result<T, AdocError>;

/// Structure for error printing
///
/// TODO: refactor and prefer it to anyhow::Error
#[derive(Debug, Error)]
pub enum AdocError {
    #[error("Failed to convert file: {0}\n{1}")]
    FailedToConvert(PathBuf, String),
}

/// Context for running `asciidoctor`, actually just options
#[derive(Debug)]
pub struct AdocContext {
    pub errors: Vec<Error>,
    /// `-B` option (base directory)
    pub src_dir: PathBuf,
    /// `-D` option (destination directory)
    pub dst_dir: PathBuf,
    /// Other options
    pub opts: CmdOptions,
}

impl AdocContext {
    pub fn new(src_dir: &Path, site_dir: &Path, opts: &CmdOptions) -> io::Result<Self> {
        let src_dir = src_dir.canonicalize()?;
        let site_dir = site_dir.canonicalize()?;

        Ok(Self {
            errors: Vec::with_capacity(10),
            src_dir,
            dst_dir: site_dir,
            opts: opts.clone(),
        })
    }

    /// Applies `asciidoctor` options
    pub fn apply_options(&self, cmd: &mut Command) {
        // setup directory settings (base/destination directory)
        let src_dir = self.src_dir.clone();
        let dst_dir = self.dst_dir.clone();

        let src_dir_str = format!("{}", src_dir.display());
        let dst_dir_str = format!("{}", dst_dir.display());

        cmd.current_dir(&src_dir)
            .args(&["-B", &src_dir_str])
            .args(&["-D", &dst_dir_str]);

        // setup user options
        for (opt, args) in &self.opts {
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
pub fn asciidoctor(src_file: &Path, acx: &mut AdocContext) -> Result<Command> {
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

    // require asciidoctor-diagram
    cmd.args(&["-r", "asciidoctor-diagram"]);

    // prefer verbose output
    cmd.arg("--trace").arg("--verbose");

    // apply directory settings and user options (often ones defined in `book.ron`)
    acx.apply_options(&mut cmd);

    Ok(cmd)
}

/// Runs `asciidoctor` and returns the output
pub fn run_asciidoctor(
    src_file: &Path,
    dummy_dst_name: &str,
    acx: &mut AdocContext,
) -> Result<std::process::Output> {
    trace!(
        "Converting adoc: `{}` -> `{}`",
        src_file.display(),
        dummy_dst_name,
    );

    let mut cmd =
        self::asciidoctor(src_file, acx).context("when setting up `asciidoctor` options")?;

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
    acx: &mut AdocContext,
) -> Result<()> {
    let output = self::run_asciidoctor(src_file, dummy_dst_name, acx)?;

    // ensure the conversion succeeded or else report it as an error
    ensure!(
        output.status.success(),
        AdocError::FailedToConvert(
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
        eprintln!(
            "Asciidoctor stderr while converting {}:",
            src_file.display()
        );
        let err = String::from_utf8(output.stderr)
            .unwrap_or("<non-UTF8 stderr by `asciidoctor`>".to_string());
        eprintln!("{}", &err);
    }

    Ok(())
}