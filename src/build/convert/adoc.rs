//! `asciidoctor` runner and metadata extracter

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

// --------------------------------------------------------------------------------
// `asciidoctor` runner

// TODO:
// pub type Result<T> = std::result::Result<T, AdocError>;

/// Structure for error printing
///
/// TODO: refactor and prefer it to anyhow::Error
#[derive(Debug, Error, Clone)]
pub enum AdocError {
    #[error("Failed to convert file: {0}\n{1}")]
    FailedToConvert(PathBuf, String),
}

/// Context for running `asciidoctor`, actually just options
#[derive(Debug)]
pub struct AdocRunContext {
    pub errors: Vec<Error>,
    /// `-B` option (base directory)
    pub src_dir: PathBuf,
    /// `-D` option (destination directory)
    pub dst_dir: PathBuf,
    /// Other options
    pub opts: CmdOptions,
}

impl AdocRunContext {
    pub fn new(src_dir: &Path, site_dir: &Path, opts: &CmdOptions) -> io::Result<Self> {
        let src_dir = src_dir.canonicalize()?;
        let site_dir = site_dir.canonicalize()?;

        Ok(AdocRunContext {
            errors: Vec::with_capacity(10),
            src_dir,
            dst_dir: site_dir,
            opts: opts.clone(),
        })
    }

    /// Applies `asciidoctor` options
    ///
    /// Place holder strings:
    ///
    /// * `${src_dir}`: replaced to the source directory
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
pub fn asciidoctor(src_file: &Path, rcx: &mut AdocRunContext) -> Result<Command> {
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
    rcx.apply_options(&mut cmd);

    Ok(cmd)
}

/// Runs `asciidoctor` and returns the output
pub fn run_asciidoctor(
    src_file: &Path,
    dummy_dst_name: &str,
    rcx: &mut AdocRunContext,
) -> Result<std::process::Output> {
    trace!(
        "Converting adoc: `{}` -> `{}`",
        src_file.display(),
        dummy_dst_name,
    );

    let mut cmd =
        self::asciidoctor(src_file, rcx).context("when setting up `asciidoctor` options")?;

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
    rcx: &mut AdocRunContext,
) -> Result<()> {
    let output = self::run_asciidoctor(src_file, dummy_dst_name, rcx)?;

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

// --------------------------------------------------------------------------------
// Metadata extracter

/// Attribute of an Asciidoctor document
///
/// Different from Asciidoctor, document attributes specified with command line arguments are always
/// overwritable by default.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdocAttr {
    /// :!<attribute>:
    Deny(String),
    /// :<attribute>: value
    Allow(String, String),
}

impl AdocAttr {
    pub fn deny(name: impl Into<String>) -> Self {
        AdocAttr::Deny(name.into())
    }

    pub fn allow(name: impl Into<String>, value: impl Into<String>) -> Self {
        AdocAttr::Allow(name.into(), value.into())
    }

    pub fn name(&self) -> &str {
        match self {
            AdocAttr::Deny(name) => name,
            AdocAttr::Allow(name, _value) => name,
        }
    }

    pub fn value(&self) -> Option<&str> {
        match self {
            AdocAttr::Deny(_name) => None,
            AdocAttr::Allow(_name, value) => Some(value),
        }
    }
}

/// Asciidoctor metadata (basically document attributes)
///
/// Because `asciidoctor --embedded` does not output document header (and the document title), we
/// have to extract document attributes manually.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdocMetadata {
    pub title: Option<String>,
    attrs: Vec<AdocAttr>,
    base: Option<Box<Self>>,
}

impl AdocMetadata {
    /// Tries to find an attribute with name. Duplicates are not conisdered
    pub fn find_attr(&self, name: &str) -> Option<&AdocAttr> {
        if let Some(attr) = self.attrs.iter().find(|a| a.name() == name) {
            return Some(attr);
        }

        if let Some(ref base) = self.base {
            return base.find_attr(name);
        }

        None
    }
}

impl AdocMetadata {
    pub fn extract(text: &str) -> Self {
        let mut lines = text.lines();

        // = Title
        let title = match lines.next() {
            Some(ln) if ln.starts_with("= ") => Some(ln[2..].trim().to_string()),
            _ => None,
        };

        // :attribute: value
        let mut attrs = Vec::with_capacity(10);
        while let Some(line) = lines.next() {
            if line.trim().is_empty() {
                continue;
            }

            // locate two colons (`:`)
            let mut colons = line.bytes().enumerate().filter(|(_i, c)| *c == b':');

            // first `:`
            match colons.next() {
                Some(_) => {}
                None => continue,
            }

            // second `:`
            let pos = match colons.next() {
                Some((i, _c)) => i,
                None => continue,
            };

            // :attribute: value
            let name = &line[1..pos].trim();
            let value = &line[pos + 1..].trim();

            // :!attribute:
            if name.starts_with('!') {
                attrs.push(AdocAttr::Deny(name[1..].to_string()));
            } else {
                attrs.push(AdocAttr::Allow(name.to_string(), value.to_string()));
            }
        }

        Self {
            title,
            attrs,
            base: None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::{AdocAttr, AdocMetadata};

    #[test]
    fn simple_metadata() {
        let article = r###"= Title here!
:revdate: Oct 23, 2020
:author: someone
:!sectnums: these text are omitted

First paragraph!
"###;

        let metadata = AdocMetadata::extract(article);

        assert_eq!(
            metadata,
            AdocMetadata {
                title: Some("Title here!".to_string()),
                attrs: vec![
                    AdocAttr::allow("revdate", "Oct 23, 2020"),
                    AdocAttr::allow("author", "someone"),
                    AdocAttr::deny("sectnums"),
                ],
                base: None,
            }
        );

        assert_eq!(
            metadata.find_attr("author"),
            Some(&AdocAttr::allow("author", "someone"))
        );
    }
}
