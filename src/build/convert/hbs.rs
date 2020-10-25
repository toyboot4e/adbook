//! Handlebars application
//!
//! Temlates are supplied [`HbsData`].

use {
    anyhow::*,
    handlebars::Handlebars,
    serde::Serialize,
    std::{fs, path::Path},
};

use crate::{
    book::toc::{Toc, TocItemContent},
    build::convert::adoc::AdocMetadata,
};

// --------------------------------------------------------------------------------
// Context

#[derive(Debug, Clone)]
pub struct HbsContext {
    pub sidebar: Sidebar,
}

impl HbsContext {
    pub fn from_root_toc_ron(toc: &Toc, src_dir: &Path) -> Result<Self> {
        let sidebar = Sidebar::from_root_toc_ron(toc, src_dir)?;

        Ok(Self { sidebar })
    }
}

#[derive(Debug, Clone)]
pub struct Sidebar {
    entries: Vec<SidebarEntry>,
}

impl Sidebar {
    pub fn from_root_toc_ron(toc: &Toc, src_dir: &Path) -> Result<Self> {
        let mut items = Vec::with_capacity(40);
        crate::book::walk::flatten_toc_items(toc, &mut items);

        let mut items: Vec<_> = items
            .into_iter()
            .map(|item| match item.content {
                TocItemContent::File(path) => (item.name, path),
                _ => unreachable!(),
            })
            .collect();
        items.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let mut entries = Vec::with_capacity(items.len());
        for item in &items {
            let (name, file) = item;
            let url = file.strip_prefix(src_dir).with_context(|| {
                format!(
                    "File in ToC not relative to source directory\n  file: {}\n  src_dir: {}",
                    file.display(),
                    src_dir.display(),
                )
            })?;

            // TODO: enable arbitrary webpage root
            let url = format!("/{}", url.display());

            entries.push(SidebarEntry {
                name: name.to_string(),
                url: Some(url),
                children: None,
            });
        }

        Ok(Self { entries })
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct SidebarEntry {
    pub name: String,
    pub url: Option<String>,
    pub children: Option<Box<Vec<Self>>>,
}

// --------------------------------------------------------------------------------
// Data

/// Variables directly supplied to Handlebars templates
#[derive(Serialize)]
pub struct HbsData<'a> {
    /// html data
    pub h_title: String,
    pub h_author: String,
    /// Asciidoctor attribute
    pub a_title: Option<String>,
    pub a_article: &'a str,
    pub a_revdate: Option<String>,
    pub a_author: Option<String>,
    pub a_email: Option<String>,
    pub a_stylesheet: Option<String>,
    /// Handlebars template context
    pub sidebar_entries: Vec<SidebarEntry>,
}

impl<'a> HbsData<'a> {
    pub fn from_metadata(html: &'a str, metadata: &AdocMetadata) -> Self {
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

        let sidebar_entries = vec![];

        HbsData {
            // TODO: supply html title via `book.ron` using placeholder sutring
            h_title: metadata.title.clone().unwrap_or("".into()),
            h_author: attr("author", &metadata).unwrap_or("".into()),
            //
            a_title: metadata.title.clone(),
            a_article: html,
            a_revdate: attr("revdate", &metadata),
            a_author: attr("author", &metadata),
            a_email: attr("email", &metadata),
            a_stylesheet: css,
            //
            sidebar_entries,
        }
    }
}

// --------------------------------------------------------------------------------
// Procedure

/// Sets up [`Handlebars`] with partials (`.hbs` files that can be included from other `.hbs`
/// files)
pub fn init_hbs(hbs_dir: &Path) -> Result<Handlebars> {
    ensure!(
        hbs_dir.is_dir(),
        "Unable to find `hbs` directory in source directory"
    );

    let mut hbs = Handlebars::new();
    hbs.set_strict_mode(true);

    let partials_dir = hbs_dir.join("partials");
    ensure!(
        partials_dir.is_dir(),
        "Unable to find `hbs` partials directory at: {}",
        partials_dir.display(),
    );

    for entry in fs::read_dir(&partials_dir)? {
        let entry = entry.context("Unexpected entry")?;
        let partial = entry.path();

        // filter non-hbs files
        if matches!(partial.extension().and_then(|s| s.to_str()), Some(".hbs")) {
            continue;
        }

        // register the hbs file as a partial
        let name = partial
            .file_stem()
            .and_then(|s| s.to_str())
            .context("Unable to stringify partial hbs file path")?;
        let text = fs::read_to_string(&partial)
            .with_context(|| format!("Unable to load partial hbs file: {}", partial.display()))?;
        hbs.register_partial(name, &text)?;
    }

    Ok(hbs)
}

pub fn render_hbs<'a>(
    html: &str,
    src_name: &str,
    metadata: &AdocMetadata,
    hbs: &mut Handlebars,
    hbs_file: &Path,
    hcx: &HbsContext,
) -> Result<String> {
    let key = format!("{}", hbs_file.display());
    hbs.register_template_file(&key, hbs_file)
        .with_context(|| format!("Error when loading hbs file: {}", hbs_file.display()))?;

    let mut hbs_data = HbsData::from_metadata(html, metadata);
    hbs_data.sidebar_entries = hcx.sidebar.entries.clone();

    let output = hbs
        .render(&key, &hbs_data)
        .with_context(|| format!("Error when converting file {}", src_name))?;

    Ok(output)
}
