//! Converts AsciiDoc files using handlebars

pub mod adoc;
pub mod hbs;

use {
    anyhow::{Context, Result},
    handlebars::Handlebars,
    std::{fs, path::Path},
};

use crate::book::config::CmdOptions;

use self::adoc::AdocContext;

/// Converts AsciiDoc file to html just by running `asciidoctor`
pub fn convert_adoc(
    src_file: &Path,
    site_dir: &Path,
    dummy_dst_name: &str,
    opts: &CmdOptions,
) -> Result<String> {
    ensure!(
        src_file.is_file(),
        "Given invalid source file path: {}",
        src_file.display()
    );

    ensure!(
        site_dir.exists(),
        "Given non-existing site directory path: {}",
        site_dir.display()
    );

    // setup dummy context & builder for an article
    let src_dir = src_file.parent().unwrap();
    let mut acx = AdocContext::new(src_dir, site_dir, opts)?;
    let mut buf = String::with_capacity(5 * 1024);
    adoc::run_asciidoctor_buf(src_file, dummy_dst_name, &mut buf, &mut acx)?;

    Ok(buf)
}

/// Converts an AsciiDoc file to html using a handlebars template
///
/// * `src`: source file path.
/// * `dst`: destination file path. it may be virtual but it has to be supplied because it's used
/// for specifying output file path
/// * `hbs`: handlebars file
/// * `opts`: options provided with `asciidoctor`
pub fn convert_adoc_with_hbs(
    src_file: &Path,
    site_dir: &Path,
    dummy_dst_name: &str,
    opts: &CmdOptions,
    hbs_file: &Path,
) -> Result<String> {
    let hbs_template = fs::read_to_string(hbs_file)?;

    let mut opts = opts.clone();
    opts.push(("--embedded".to_string(), vec![]));
    let text = self::convert_adoc(src_file, site_dir, dummy_dst_name, &opts)?;

    // FIXME: stub handlebars runner
    let mut hbs = Handlebars::new();
    hbs.set_strict_mode(true);
    hbs.register_template_string("article", &hbs_template)?;

    let hbs_data = hbs::HbsTemplate { article: &text };

    let final_output = hbs
        .render("article", &hbs_data)
        .with_context(|| "Unable to render handlebars template")?;

    Ok(final_output)
}
