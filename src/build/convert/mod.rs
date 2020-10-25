//! Converts AsciiDoc files using `asciidoctor` and Handlebars
//!
//! # Placeholder strings for `asciidoctor` options
//!
//! In `adbook`, `asciidoctor` options are supplied with the following placeholder strings:
//!
//! * `{base_url}`: base url. useful when supplying absolute path
//! * `{src_dir}`: source directory
//! * `{dst_dir}`: destination directory
//!
//! # Handlebars attribute
//!
//! `adbook` treats `hbs` AsciiDoc attribute as the path to a Handlebars template file:
//!
//! ```adoc
//! = Simple article
//! :hbs: hbs/simple.hbs
//! // translated as: src/hbs/simple.hbs
//!
//! This is a simple article templated with `simple.hbs`!
//! ```
//!
//! `hbs` is always **relative to the root directory**; no base directory is supplied.

mod adoc;
pub mod hbs;

use {
    anyhow::{Context, Result},
    std::{fmt::Write, fs, path::Path},
};

pub use self::adoc::AdocRunContext;
use self::hbs::HbsContext;

/// Converts an AsciiDoc file to an html string just by running `asciidoctor`
///
/// * `dummy_dst_name`: used for debug log
/// * `opts`: options provided with `asciidoctor`
pub fn convert_adoc(
    src_file: &Path,
    dummy_dst_name: &str,
    acx: &AdocRunContext,
    hcx: &HbsContext,
) -> Result<String> {
    let mut buf = String::with_capacity(5 * 1024);
    self::convert_adoc_buf(&mut buf, src_file, dummy_dst_name, acx, hcx)?;
    Ok(buf)
}

/// Converts an AsciiDoc file to an html string and then applies a Handlebars template
///
/// Be sure that the `buf` is always cleared.
///
/// * `dummy_dst_name`: used for debug log
/// * `opts`: options provided with `asciidoctor`
pub fn convert_adoc_buf(
    buf: &mut String,
    src_file: &Path,
    dummy_dst_name: &str,
    acx: &AdocRunContext,
    hcx: &HbsContext,
) -> Result<()> {
    ensure!(
        src_file.is_file(),
        "Given invalid source file path: {}",
        src_file.display()
    );

    // extract metadata
    let metadata = {
        let adoc_text = fs::read_to_string(src_file).context("Unable to read source file")?;
        adoc::AdocMetadata::extract_with_base(&adoc_text, &acx)
    };

    // should we use "embedded mode" of Asciidoctor?
    let mut acx = acx.clone();
    if metadata.find_attr("hbs").is_some() {
        acx.set_embedded_mode(true);
    }

    // run Asciidoctor and write the output to `buf`
    buf.clear();
    adoc::run_asciidoctor_buf(buf, src_file, dummy_dst_name, &acx)?;

    // maybe apply handlebars
    if let Some(hbs_attr) = metadata.find_attr("hbs") {
        let src_name = format!("{}", src_file.display());

        let hbs_file_path = {
            let hbs_name = hbs_attr
                .value()
                .ok_or_else(|| anyhow!("`hbs` attribute without path"))?;
            hcx.src_dir.join(hbs_name)
        };

        // `.hbs` files are always located just under `hbs_dir`
        let output = {
            let hbs_dir = hbs_file_path.parent().unwrap();
            let mut hbs = hbs::init_hbs(&hbs_dir)?;
            hbs::render_hbs(buf, &src_name, &metadata, &mut hbs, &hbs_file_path, hcx)?
        };

        buf.clear();
        buf.write_str(&output)?;
    }

    Ok(())
}
