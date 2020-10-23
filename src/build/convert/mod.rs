//! Converts AsciiDoc files using `asciidoctor` and Handlebars
//!
//! # Placeholder strings for `asciidoctor` options
//!
//! `asciidoctor` options are supplied with the following placeholder strings:
//!
//! * `{src_dir}`
//! * `{dst_dir}`
//!
//! # Handlebars attribute
//!
//! `adbook` treats `hbs` AsciiDoc attribute as the path to a Handlebars template file.
//!
//! ```adoc
//! = Simple article
//! :hbs: hbs/simple.hbs
//! // translated as: src/hbs/simple.hbs
//!
//! This is a simple article templated with `simple.hbs`!
//! ```
//!
//! `adbook` doesn't provide an attribute that supplies a base directory to the `hbs` attribute.

mod adoc;
pub mod hbs;

use {
    anyhow::{Context, Result},
    std::{fmt::Write, fs, path::Path},
};

use crate::book::config::CmdOptions;

use self::adoc::AdocRunContext;

fn ensure_paths(src_file: &Path, src_dir: &Path, site_dir: &Path) -> Result<()> {
    ensure!(
        src_file.is_file(),
        "Given invalid source file path: {}",
        src_file.display()
    );

    ensure!(
        src_dir.is_dir(),
        "Given non-directory site directory path: {}",
        site_dir.display()
    );

    ensure!(
        site_dir.is_dir(),
        "Given non-directory site directory path: {}",
        site_dir.display()
    );

    Ok(())
}

/// Converts an AsciiDoc file to an html string just by running `asciidoctor`
///
/// * `dummy_dst_name`: used for debug log
/// * `opts`: options provided with `asciidoctor`
pub fn convert_adoc(
    src_file: &Path,
    src_dir: &Path,
    site_dir: &Path,
    dummy_dst_name: &str,
    opts: &CmdOptions,
) -> Result<String> {
    let mut buf = String::with_capacity(5 * 1024);
    self::convert_adoc_buf(&mut buf, src_file, src_dir, site_dir, dummy_dst_name, opts)?;
    Ok(buf)
}

/// Converts an AsciiDoc file to an html string and then maybe applies a Handlebars template
///
/// Be sure that the `buf` is always cleared.
///
/// * `dummy_dst_name`: used for debug log
/// * `opts`: options provided with `asciidoctor`
pub fn convert_adoc_buf(
    buf: &mut String,
    src_file: &Path,
    src_dir: &Path,
    site_dir: &Path,
    dummy_dst_name: &str,
    opts: &CmdOptions,
) -> Result<()> {
    self::ensure_paths(src_file, src_dir, site_dir)?;

    // extract metadata
    let metadata = {
        let text = fs::read_to_string(src_file).context("Unable to read source file")?;
        adoc::AdocMetadata::extract_with_base(&text, opts)
    };

    // should we use "embedded mode" of Asciidoctor?
    let mut opts = opts.clone();
    opts.push(("--embedded".to_string(), vec![]));

    // run Asciidoctor and write the output to `buf`
    let mut rcx = AdocRunContext::new(src_dir, site_dir, &opts)?;
    buf.clear();
    adoc::run_asciidoctor_buf(src_file, dummy_dst_name, buf, &mut rcx)?;

    // maybe apply handlebars
    if let Some(hbs_attr) = metadata.find_attr("hbs") {
        let hbs_file_path = {
            let hbs_name = hbs_attr
                .value()
                .ok_or_else(|| anyhow!("`hbs` attribute without path"))?;
            src_dir.join(hbs_name)
        };

        let output = hbs::render_hbs(&buf, metadata, &hbs_file_path)?;

        buf.clear();
        buf.write_str(&output)?;
    }

    Ok(())
}
