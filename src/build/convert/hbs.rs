//! Handlebars application
//!
//! Temlates are supplied [`HbsData`].

use {
    anyhow::{Context, Result},
    handlebars::Handlebars,
    serde::Serialize,
    std::path::Path,
};

use crate::{book::config::CmdOptions, build::convert::adoc::AdocMetadata};

/// Variables supplied to Handlebars templates
#[derive(Default, Serialize)]
pub struct HbsData<'a> {
    // html data
    h_title: String,
    h_author: String,
    // Asciidoctor attributes
    a_title: Option<String>,
    a_article: &'a str,
    a_revdate: Option<String>,
    a_author: Option<String>,
    a_email: Option<String>,
    a_stylesheet: Option<String>,
}

/// * TODO: retained mode
/// * TODO: return both output and error
pub fn render_hbs(html: &str, metadata: AdocMetadata, hbs_file: &Path) -> Result<String> {
    let key = "adbook_template";

    let hbs = {
        let mut hbs = Handlebars::new();
        hbs.set_strict_mode(true);
        hbs.register_template_file(key, hbs_file)?;
        hbs
    };

    let hbs_data = {
        fn attr(name: &str, metadata: &AdocMetadata) -> Option<String> {
            metadata
                .find_attr(name)
                .and_then(|a| a.value().map(|s| s.to_string()))
        }

        let css = attr("stylesheet", &metadata).map(|rel| {
            if let Some(base) = attr("stylesdir", &metadata) {
                format!("{}/{}", base, rel)
            } else {
                rel
            }
        });

        HbsData {
            // TODO: supply html title via `book.ron` using placeholder sutring
            h_title: metadata.title.clone().unwrap_or("".into()),
            h_author: attr("author", &metadata).unwrap_or("".into()),
            a_title: metadata.title.clone(),
            a_article: html,
            a_revdate: attr("revdate", &metadata),
            a_author: attr("author", &metadata),
            a_email: attr("email", &metadata),
            a_stylesheet: css,
        }
    };

    let output = hbs.render(key, &hbs_data)?;

    Ok(output)
}
