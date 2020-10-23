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
    /// # Place holder strings
    ///
    /// * `{src_dir}`: replaced to source directory
    /// * `{dst_dir}`: replaced to destination directory
    pub fn apply_options(&self, cmd: &mut Command) {
        // setup directory settings (base/destination directory)
        let src_dir_str = format!("{}", self.src_dir.display());
        let dst_dir_str = format!("{}", self.dst_dir.display());

        cmd.current_dir(&self.src_dir)
            .args(&["-B", &src_dir_str])
            .args(&["-D", &dst_dir_str]);

        // setup user options
        for (opt, args) in &self.opts {
            // case 1. option without argument
            if args.is_empty() {
                cmd.arg(opt);
                continue;
            }

            // case 2. (option with argument) specified n times
            // like, -a linkcss -a sectnums ..
            for arg in args {
                // setup placeholder string
                let arg = arg.replace(r#"{src_dir}"#, &src_dir_str);
                let arg = arg.replace(r#"{dst_dir}"#, &dst_dir_str);

                cmd.args(&[opt, &arg]);
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

/// Constructors
impl AdocAttr {
    /// "name" -> Deny("name")
    pub fn deny(name: impl Into<String>) -> Self {
        AdocAttr::Deny(name.into())
    }

    /// "name", "value"
    pub fn allow(name: impl Into<String>, value: impl Into<String>) -> Self {
        AdocAttr::Allow(name.into(), value.into())
    }

    /// "name" -> Allow("attr") | Deny("attr")
    pub fn from_name(name: &str) -> Self {
        if name.starts_with('!') {
            Self::deny(&name[1..])
        } else {
            Self::allow(name, "")
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
    // TODO: supply base attribute set from `book.ron`
    base: Option<Box<Self>>,
}

impl AdocMetadata {
    /// Tries to find an attribute with name. Duplicates are not conisdered
    pub fn find_attr(&self, name: &str) -> Option<&AdocAttr> {
        // from self
        if let Some(attr) = self.attrs.iter().find(|a| a.name() == name) {
            return Some(attr);
        }

        // from base
        if let Some(ref base) = self.base {
            return base.find_attr(name);
        }

        None
    }
}

/// Parsers
impl AdocMetadata {
    /// Sets the fallback [`AdocMetadata`]
    pub fn derive(&mut self, base: Self) {
        self.base = Some(Box::new(base));
    }

    /// Extracts metadata from AsciiDoc string and sets up fallback attributes from `asciidoctor`
    /// command line options
    pub fn extract_with_base(text: &str, cli_opts: &CmdOptions) -> Self {
        let mut meta = Self::extract(text);

        let base = Self::from_cmd_opts(cli_opts);
        meta.derive(base);

        meta
    }

    fn is_line_to_skip(ln: &str) -> bool {
        let ln = ln.trim();
        ln.is_empty() || ln.starts_with("//")
    }

    /// Extracts metadata from AsciiDoc string
    pub fn extract(text: &str) -> Self {
        // always skip "whitespace" lines
        let mut lines = text.lines().filter(|ln| !Self::is_line_to_skip(ln));

        // = Title
        let title = match lines.next() {
            Some(ln) if ln.starts_with("= ") => Some(ln[2..].trim().to_string()),
            _ => None,
        };

        // :attribute: value
        let mut attrs = Vec::with_capacity(10);
        while let Some(line) = lines.next() {
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

            if name.starts_with('!') {
                // :!attribute:
                attrs.push(AdocAttr::deny(&name[1..]));
            } else {
                // :attribute:
                attrs.push(AdocAttr::allow(*name, *value));
            }
        }

        Self {
            title,
            attrs,
            base: None,
        }
    }

    /// Extracts `asciidoctor` options that matches to `-a attr=value`
    pub fn from_cmd_opts(opts: &CmdOptions) -> Self {
        let attr_opts = match opts.iter().find(|(opt_name, _attr_opts)| opt_name == "-a") {
            Some((_opt_name, opts)) => opts,
            None => {
                return Self {
                    title: None,
                    attrs: vec![],
                    base: None,
                }
            }
        };

        let mut attrs = Vec::with_capacity(10);

        for opt in attr_opts.iter() {
            let eq_pos = opt
                .bytes()
                .enumerate()
                .find(|(_i, c)| *c == b'=')
                .map(|(i, _c)| i)
                .unwrap_or(0);

            // attr | !attr
            if eq_pos == 0 {
                attrs.push(AdocAttr::from_name(opt));
                continue;
            }

            // name=value | name@=value | name=value@
            // we'll just ignore `@` symbols; different from the original Asciidoctor, attributes
            // are always overridable by documents
            let mut name = &opt[0..eq_pos];
            if name.ends_with('@') {
                name = &name[0..name.len() - 1];
            }

            let mut value = &opt[eq_pos + 1..];
            if value.ends_with('@') {
                value = &value[0..value.len() - 1];
            }

            attrs.push(AdocAttr::allow(name, value));
        }

        Self {
            title: None,
            attrs,
            base: None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::{AdocAttr, AdocMetadata};

    const ARTICLE: &str = r###"
// ^ blank line

= Title here!

:revdate: Oct 23, 2020
// whitespace again

:author: someone
:!sectnums: these text are omitted

First paragraph!
"###;

    #[test]
    fn simple_metadata() {
        let metadata = AdocMetadata::extract(ARTICLE);

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

    #[test]
    fn base_test() {
        let mail = "someone@mail.domain";

        let cmd_opts = vec![(
            "-a".to_string(),
            vec!["sectnums".to_string(), format!("email={}", mail)],
        )];

        let deriving = AdocMetadata::extract_with_base(ARTICLE, &cmd_opts);

        assert_eq!(
            deriving.find_attr("sectnums"),
            Some(&AdocAttr::deny("sectnums"))
        );

        assert_eq!(
            deriving.find_attr("email"),
            Some(&AdocAttr::allow("email", mail))
        );
    }
}
