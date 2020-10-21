//! Handlebars application

use {
    anyhow::{Context, Result},
    handlebars::Handlebars,
    serde::Serialize,
    std::{fs, path::Path},
};

/// Variables provided to handlebars template
#[derive(Serialize)]
pub struct HbsTemplate<'a> {
    pub article: &'a str,
    // TODO: supply title and attributes
    // TODO: supply css etc.
}
