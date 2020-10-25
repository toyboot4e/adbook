//! Handlebars application
//!
//! Temlates are supplied [`HbsData`].

use {
    anyhow::*,
    handlebars::Handlebars,
    serde::Serialize,
    std::{
        fs,
        path::{Path, PathBuf},
    },
};

use crate::{
    book::{
        toc::{Toc, TocItem, TocItemContent},
        BookStructure,
    },
    build::convert::adoc::AdocMetadata,
};

// --------------------------------------------------------------------------------
// Context

/// Context to generate [`HbsData`]
#[derive(Debug, Clone)]
pub struct HbsContext {
    pub src_dir: PathBuf,
    pub sidebar: Sidebar,
}

impl HbsContext {
    pub fn from_book(book: &BookStructure) -> (Self, Vec<Error>) {
        let (sidebar, errors) = Sidebar::from_root_toc_ron(&book.toc, &book.src_dir_path());

        let me = Self {
            src_dir: book.src_dir_path(),
            sidebar,
        };

        (me, errors)
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct SidebarItem {
    pub name: String,
    pub url: Option<String>,
    pub children: Option<Box<Vec<Self>>>,
}

#[derive(Debug, Clone)]
pub struct Sidebar {
    items: Vec<SidebarItem>,
}

impl Sidebar {
    pub fn from_root_toc_ron(toc: &Toc, src_dir: &Path) -> (Self, Vec<Error>) {
        let mut errors = Vec::with_capacity(20);
        let items: Vec<SidebarItem> = Self::map_toc(toc, src_dir, &mut errors);

        (Self { items }, errors)
    }

    fn map_toc(toc: &Toc, src_dir: &Path, errors: &mut Vec<Error>) -> Vec<SidebarItem> {
        toc.items
            .iter()
            .filter_map(|item| match Self::map_item(item, src_dir, errors) {
                Ok(item) => Some(item),
                Err(err) => {
                    errors.push(err);
                    None
                }
            })
            .collect()
    }

    fn map_item(item: &TocItem, src_dir: &Path, errors: &mut Vec<Error>) -> Result<SidebarItem> {
        let name = item.name.clone();
        match &item.content {
            TocItemContent::File(ref file) => {
                let url = file.strip_prefix(src_dir)?.with_extension("html");
                let url = format!("/{}", url.display());

                Ok(SidebarItem {
                    name,
                    url: Some(url),
                    children: None,
                })
            }
            TocItemContent::SubToc(toc) => {
                let children = Self::map_toc(&toc, src_dir, errors);

                // TODO: add URL corresponding to the toc
                // let url = path.strip_prefix(src_dir)?;
                // let url = format!("/{}", url.display());
                let url = None;

                Ok(SidebarItem {
                    name,
                    url,
                    children: Some(Box::new(children)),
                })
            }
        }
    }
}

// --------------------------------------------------------------------------------
// Data

/// Variables directly supplied to Handlebars templates
#[derive(Serialize, Debug, Clone)]
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
    pub sidebar_items: Vec<SidebarItem>,
}

impl<'a> HbsData<'a> {
    /// WARN: be sure to set `sidebar_items` later
    pub fn new(html: &'a str, meta: &AdocMetadata, sidebar: Sidebar) -> Self {
        fn attr(name: &str, metadata: &AdocMetadata) -> Option<String> {
            metadata
                .find_attr(name)
                .and_then(|a| a.value().map(|s| s.to_string()))
        }

        let css = attr("stylesheet", &meta).map(|rel| {
            if let Some(base) = attr("stylesdir", &meta) {
                // the css file path is supplied with base directory path!
                format!("{}/{}", base, rel)
            } else {
                rel
            }
        });

        HbsData {
            // TODO: supply html title via `book.ron` using placeholder sutring
            h_title: meta.title.clone().unwrap_or("".into()),
            h_author: attr("author", &meta).unwrap_or("".into()),
            //
            a_title: meta.title.clone(),
            a_article: html,
            a_revdate: attr("revdate", &meta),
            a_author: attr("author", &meta),
            a_email: attr("email", &meta),
            a_stylesheet: css,
            //
            sidebar_items: sidebar.items,
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

    let hbs_data = HbsData::new(html, metadata, hcx.sidebar.clone());

    let output = hbs
        .render(&key, &hbs_data)
        .with_context(|| format!("Error when converting file {}", src_name))?;

    Ok(output)
}
