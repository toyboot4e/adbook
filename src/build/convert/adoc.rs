/*!
`asciidoctor` runner and metadata extracter
*/

use {
    anyhow::{bail, ensure, Context, Result},
    std::{
        path::{Path, PathBuf},
        process::Command,
    },
    thiserror::Error,
};

use crate::book::{config::CmdOptions, BookStructure};

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

/// Context for running `asciidoctor`
///
/// # String interpolation
///
/// [`Self::replace_placeholder_strings`] does it.
///
/// # Asciidoctor options that are not used
///
/// ## `asciidoctor -B`
///
/// It's used to supply (virtual) directory, especially when the input is stdin. The
/// directory path is used for the "safe mode".
///
/// ## `asciidoctor -D`
///
/// It's for specifying output file path and good for mirroing one directory to another:
///
/// ```sh
/// $ asciidoctor -D out -R . '**/*.adoc'
/// ```
#[derive(Debug, Clone)]
pub struct AdocRunContext {
    /// Source directory
    src_dir: String,
    /// Destination directory
    dst_dir: String,
    /// `asciidoctor -a` (attributes) or other options
    opts: CmdOptions,
    /// Used to modify `asciidoctor` attributes supplied to `.adoc` files
    base_url: String,
}

impl AdocRunContext {
    pub fn from_book(book: &BookStructure, dst_dir: &Path) -> Self {
        let src_dir = format!("{}", book.src_dir_path().display());
        let dst_dir = format!("{}", dst_dir.display());

        Self {
            src_dir,
            dst_dir,
            opts: book.book_ron.adoc_opts.clone(),
            base_url: book.book_ron.base_url.to_string(),
        }
    }

    /// Embedded mode: output without header (including title) and footer
    pub fn set_embedded_mode(&mut self, b: bool) {
        if b {
            self.opts.push(("--embedded".to_string(), vec![]));
        } else {
            self.opts = self
                .opts
                .clone()
                .into_iter()
                .filter(|(name, _values)| name == "--embedded")
                .collect();
        }
    }

    /// Applies `asciidoctor` options defined in `book.ron`
    pub fn apply_options(&self, cmd: &mut Command) {
        // setup directory settings
        cmd.current_dir(&self.src_dir).args(&["-B", &self.src_dir]);

        // we're outputting to stdout, `-D` does nothing:
        // cmd.args(&["-D", &self.dst_dir]);

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
                let arg = self.replace_placeholder_strings(arg);
                cmd.args(&[opt, &arg]);
            }
        }
    }

    fn replace_placeholder_strings(&self, arg: &str) -> String {
        let arg = arg.replace(r#"{base_url}"#, &self.base_url);
        let arg = arg.replace(r#"{src_dir}"#, &self.src_dir);
        let arg = arg.replace(r#"{dst_dir}"#, &self.dst_dir);

        arg
    }
}

/// Creates a slash-delimited string from a canonicalized path
///
/// `canonizalize` creates UNC path on Windows, which is not supported by `asciidoctor`.
fn normalize(path: &Path) -> Result<String> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        path.canonicalize()
            .with_context(|| "Unable to canonicallize source file path")?
    };

    // FIXME:
    Ok(format!("{}", path.display()))
}

/// Sets up `asciidoctor` command
pub fn asciidoctor(src_file: &Path, acx: &AdocRunContext) -> Result<Command> {
    ensure!(
        src_file.exists(),
        "Given non-existing file as conversion source"
    );

    let mut cmd = Command::new("asciidoctor");

    // output to stdout
    cmd.arg(&normalize(src_file)?).args(&["-o", "-"]);

    // require `asciidoctor-diagram`
    cmd.args(&["-r", "asciidoctor-diagram"]);

    // prefer verbose output
    cmd.arg("--trace").arg("--verbose");

    // apply directory settings and user options (often ones defined in `book.ron`)
    acx.apply_options(&mut cmd);

    Ok(cmd)
}

/// Runs `asciidoctor` command and returns the output
pub fn run_asciidoctor(
    src_file: &Path,
    dummy_dst_name: &str,
    acx: &AdocRunContext,
) -> Result<std::process::Output> {
    // trace!(
    //     "Converting adoc: `{}` -> `{}`",
    //     src_file.display(),
    //     dummy_dst_name,
    // );

    let mut cmd =
        self::asciidoctor(src_file, acx).context("when setting up `asciidoctor` options")?;

    // trace!("{:?}", cmd);

    let output = match cmd.output() {
        Ok(output) => output,
        Err(err) => {
            bail!(
                "when running `asciidoctor`:\n  src: {}\n  dst: {}\n  cmd: {:?}\n  stdout: {:?}",
                // src_file.display(), dummy_dst_name,
                normalize(src_file)?,
                dummy_dst_name,
                cmd,
                err
            )
        }
    };

    Ok(output)
}

/// Runs `asciidoctor` command and writes the output to a string buffer
pub fn run_asciidoctor_buf(
    buf: &mut String,
    src_file: &Path,
    dummy_dst_name: &str,
    acx: &AdocRunContext,
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
    buf.push_str(text);

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
// Metadata extraction

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

/// Asciidoctor metadata supplied to Handlebars data
///
/// We have to extract them manually because `asciidoctor --embedded` doesn't generate document
/// title and header.
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
    pub fn extract_with_base(adoc_text: &str, acx: &AdocRunContext) -> Self {
        let mut meta = Self::extract(adoc_text, acx);

        let base = Self::from_cmd_opts(&acx.opts, acx);
        meta.derive(base);

        meta
    }

    /// "Whitespace" line or comment lines are skipped when extracting header and attributes
    fn is_line_to_skip(ln: &str) -> bool {
        let ln = ln.trim();
        ln.is_empty() || ln.starts_with("//")
    }

    /// Extracts metadata from AsciiDoc string
    ///
    /// Replaces placeholder strings in attribute values.
    pub fn extract(text: &str, acx: &AdocRunContext) -> Self {
        let mut lines = text
            .lines()
            .filter(|ln| !Self::is_line_to_skip(ln))
            .peekable();

        // = Title
        let title = match lines.peek() {
            Some(ln) if ln.starts_with("= ") => {
                let ln = lines.next().unwrap();
                Some(ln[2..].trim().to_string())
            }
            _ => None,
        };

        // :attribute: value
        let mut attrs = Vec::with_capacity(10);
        while let Some(line_str) = lines.next() {
            // locate two colons (`:`)
            let mut colons = line_str.bytes().enumerate().filter(|(_i, c)| *c == b':');

            // first `:`
            match colons.next() {
                // line starting with `:`
                Some((ix, _c)) if ix == 0 => {}
                // line not starting with `:`
                Some((_ix, _c)) => continue,
                None => break,
            }

            // second `:`
            let pos = match colons.next() {
                Some((i, _c)) => i,
                None => continue,
            };

            use std::str::from_utf8;
            let line = line_str.as_bytes();

            // :attribute: value
            let name = match from_utf8(&line[1..pos]) {
                Ok(name) => name.trim(),
                Err(_err) => {
                    eprintln!("Bug! AdocMetadata error line: {}", line_str);
                    continue;
                }
            };

            let value = match from_utf8(&line[pos + 1..]) {
                Ok(v) => v.trim(),
                Err(_err) => {
                    eprintln!("Bug! AdocMetadata error line: {}", line_str);
                    continue;
                }
            };

            if name.starts_with('!') {
                // :!attribute:
                attrs.push(AdocAttr::deny(&name[1..]));
            } else {
                // :attribute: value
                let value = acx.replace_placeholder_strings(value);
                attrs.push(AdocAttr::allow(name, value));
            }
        }

        Self {
            title,
            attrs,
            base: None,
        }
    }

    /// Extracts `asciidoctor` options that matches to `-a attr=value`
    pub fn from_cmd_opts(opts: &CmdOptions, acx: &AdocRunContext) -> Self {
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

            let value = acx.replace_placeholder_strings(value);
            attrs.push(AdocAttr::allow(name, &value));
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
    use super::{AdocAttr, AdocMetadata, AdocRunContext};

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
        // dummy
        let acx = AdocRunContext {
            src_dir: ".".to_string(),
            dst_dir: ".".to_string(),
            opts: vec![],
            base_url: "".to_string(),
        };

        let metadata = AdocMetadata::extract(ARTICLE, &acx);

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

        // dummy
        let acx = AdocRunContext {
            src_dir: ".".to_string(),
            dst_dir: ".".to_string(),
            opts: cmd_opts,
            base_url: "".to_string(),
        };

        let deriving = AdocMetadata::extract_with_base(ARTICLE, &acx);

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
