/*!
Converts AsciiDoc files using `asciidoctor` and Handlebars

# Placeholder strings for `asciidoctor` options

In `adbook`, `asciidoctor` options are supplied with the following placeholder strings:

* `{base_url}`: base url in this form: `/base/url`. useful when supplying absolute path
* `{src_dir}`: path to source directory
* `{dst_dir}`: path to destination directory

We can use them for document attributes:

```adoc
:imagesdir: {base_url}/static/img
:imagesoutdir: {src_dir}/static/img
```

Usually those paths are globally specified in `book.ron`.

# Handlebars attribute

`adbook` specially treats `hbs` AsciiDoc attribute as the path to a Handlebars template file:

```adoc
= Simple article
:hbs: theme/hbs/simple.hbs
// translated to: {src_dir}/theme/hbs/simple.hbs
```

`hbs` is always relative to the source directory and no base directory is supplied.
*/

mod adoc;
pub mod hbs;

use {
    anyhow::{Context, Result},
    std::{fmt::Write, fs, path::Path},
};

pub use self::adoc::AdocRunContext;
use self::hbs::{HbsContext, HbsInput};

use crate::book::BookStructure;

/// Converts an AsciiDoc file to an html string just by running `asciidoctor`
///
/// * `dummy_dst_name`: used for debug log
/// * `opts`: options provided with `asciidoctor`
pub fn convert_adoc(
    src_file: &Path,
    dummy_dst_name: &str,
    acx: &AdocRunContext,
    hcx: &HbsContext,
    book: &BookStructure,
) -> Result<String> {
    let mut buf = String::with_capacity(5 * 1024);
    self::convert_adoc_buf(&mut buf, src_file, dummy_dst_name, acx, hcx, book)?;
    Ok(buf)
}

/// Converts an AsciiDoc file to an html string and then applies a Handlebars template
///
/// Be sure that the `buf` is always cleared.
pub fn convert_adoc_buf(
    buf: &mut String,
    src_file: &Path,
    dst_name_for_debug: &str,
    acx: &AdocRunContext,
    hcx: &HbsContext,
    book: &BookStructure,
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

    // we use "embedded mode" of `asciidoctor` if we'll apply Handlebars template later
    let mut acx = acx.clone();
    if metadata.find_attr("hbs").is_some() {
        acx.set_embedded_mode(true);
    }

    // run `asciidoctor` and write the output to `buf`
    buf.clear();
    adoc::run_asciidoctor_buf(buf, src_file, dst_name_for_debug, &acx)?;

    // maybe apply Handlebars template
    if let Some(hbs_attr) = metadata.find_attr("hbs") {
        let src_file_name = format!("{}", src_file.display());

        let hbs_file_path = {
            let hbs_name = hbs_attr
                .value()
                .ok_or_else(|| anyhow!("`hbs` attribute without path"))?;
            hcx.src_dir.join(hbs_name)
        };

        // `.hbs` files are always located just under `hbs_dir`
        let hbs_input = HbsInput::new(buf, &metadata, &hcx.base_url, hcx.sidebar.clone());
        let output = if book.book_ron.use_default_theme {
            // use default theme
            let mut hbs = hbs::init_hbs_default()?;
            hbs::render_hbs_default(&mut hbs, &hbs_input, &src_file_name)?
        } else {
            // use user theme
            let mut hbs = hbs::init_hbs_user(hbs_file_path.parent().unwrap())?;
            hbs::render_hbs_user(&mut hbs, &hbs_input, &src_file_name, &hbs_file_path)?
        };

        buf.clear();
        buf.write_str(&output)?;
    }

    Ok(())
}
