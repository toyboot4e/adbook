//! Converts AsciiDoc files using `asciidoctor` and Handlebars

mod adoc;
pub mod hbs;

use {
    anyhow::{Context, Result},
    handlebars::Handlebars,
    std::{fs, path::Path},
};

use crate::book::config::CmdOptions;

use self::adoc::AdocRunContext;

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

/// Converts an AsciiDoc file to an html string just by running `asciidoctor`
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

    // setup dummy context & builder for an article
    let mut rcx = AdocRunContext::new(src_dir, site_dir, opts)?;
    adoc::run_asciidoctor_buf(src_file, dummy_dst_name, buf, &mut rcx)?;

    Ok(())
}

/// Converts an AsciiDoc file to an html string using a Handlebars template
///
/// * `dummy_dst_name`: used for debug log
/// * `opts`: options provided with `asciidoctor`
pub fn convert_adoc_with_hbs(
    src_file: &Path,
    src_dir: &Path,
    site_dir: &Path,
    dummy_dst_name: &str,
    opts: &CmdOptions,
    hbs_file: &Path,
) -> Result<String> {
    // get "embedded" version of `asciidoctor` output
    let mut opts = opts.clone();
    opts.push(("--embedded".to_string(), vec![]));

    let src_str = fs::read_to_string(src_file)?;
    // TODO: use `&str`
    let html = self::convert_adoc(src_file, src_dir, site_dir, dummy_dst_name, &opts)?;

    // render handlebars template
    let output = hbs::render_hbs(&html, &src_str, hbs_file).with_context(|| {
        format!(
            "Unable to render handlebars template\n     hbs: {}\n    adoc: {}",
            hbs_file.display(),
            src_file.display(),
        )
    })?;

    Ok(output)
}
