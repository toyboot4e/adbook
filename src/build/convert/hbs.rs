//! Handlebars application
//!
//! Temlates are supplied [`HbsData`].

use {
    anyhow::{Context, Result},
    handlebars::Handlebars,
    serde::Serialize,
    std::path::Path,
};

use crate::build::convert::adoc::AdocMetadata;

/// Data supplied to Handlebars templates
#[derive(Default, Serialize)]
pub struct HbsData<'a> {
    a_title: Option<String>,
    a_article: &'a str,
    a_revdate: Option<String>,
    a_author: Option<String>,
    a_email: Option<String>,
}

/// * TODO: retained mode
/// * TODO: return both output and error
pub fn render_hbs(html: &str, adoc: &str, hbs_file: &Path) -> Result<(String)> {
    let key = "adbook_template";

    let hbs = {
        let mut hbs = Handlebars::new();
        hbs.set_strict_mode(true);
        hbs.register_template_file(key, hbs_file)?;
        hbs
    };

    let hbs_data = {
        let metadata = AdocMetadata::extract(adoc);

        fn attr(name: &str, metadata: &AdocMetadata) -> Option<String> {
            metadata
                .find_attr("revdate")
                .and_then(|a| a.value().map(|s| s.to_string()))
        }

        HbsData {
            a_title: metadata.title.clone(),
            a_article: html,
            a_revdate: attr("revdate", &metadata),
            a_author: attr("author", &metadata),
            a_email: attr("email", &metadata),
        }
    };

    let output = hbs.render(key, &hbs_data)?;

    Ok(output)
}
