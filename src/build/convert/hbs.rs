/*!
Handlebars application

HBS templates are supplied [`HbsInput`].
*/

use {
    anyhow::*,
    handlebars::Handlebars,
    serde::Serialize,
    std::{
        fs,
        io::{BufRead, BufReader},
        path::Path,
    },
};

use crate::{
    book::{toc::TocItem, BookStructure},
    build::convert::adoc::AdocMetadata,
};

// --------------------------------------------------------------------------------
// Context

#[derive(Serialize, Debug, Clone)]
pub struct SidebarItem {
    pub name: String,
    pub url: Option<String>,
    pub children: Option<Box<Vec<Self>>>,
    pub active: bool,
    pub depth: usize,
}

#[derive(Debug, Clone)]
pub struct Sidebar {
    items: Vec<SidebarItem>,
    /// Items up to the level is open by default
    fold_level: Option<usize>,
}

impl Sidebar {
    /// Reads the first line of a file and if it starts with `= ` it is the title
    fn get_title(title: &str, file: &Path) -> Result<String> {
        if !title.is_empty() {
            return Ok(title.to_string());
        }

        let f = fs::File::open(&file)
            .with_context(|| anyhow!("Unable to open file {}", file.display()))?;
        let mut f = BufReader::new(f);

        let mut buf = String::with_capacity(200);
        f.read_line(&mut buf)
            .with_context(|| anyhow!("Unable to peek file {}", file.display()))?;

        if buf.starts_with("= ") {
            Ok(buf[2..].trim().to_string())
        } else {
            Ok("<untitled>".to_string())
        }
    }

    /// Gets an URL for an article in the sidebar
    ///
    /// The `base_url_str` is in for of `/path/to/dir`.
    pub fn get_url(src_dir: &Path, src_file: &Path, base_url_str: &str) -> Result<String> {
        let url = src_file
            .strip_prefix(src_dir)
            .with_context(|| {
                anyhow!(
                    "Unable to strip prefix: `{}` from `{}`",
                    src_dir.display(),
                    src_file.display()
                )
            })?
            .with_extension("html");

        let url = format!("{}/{}", base_url_str, url.display());
        Ok(url)
    }

    pub fn from_book(book: &BookStructure) -> (Self, Vec<Error>) {
        let mut errors = Vec::with_capacity(20);

        let summary_item = {
            let (name, file) = (&book.toc.name, &book.toc.summary);

            let name = match Self::get_title(name, file) {
                Ok(name) => name,
                Err(err) => {
                    errors.push(err);
                    "<ERROR>".to_string()
                }
            };

            TocItem::File(name, book.toc.summary.to_owned())
        };

        let items = std::iter::once(&summary_item).chain(&book.toc.items);
        let items: Vec<SidebarItem> = {
            Self::collect_sidebar_items(
                items,
                &book.src_dir_path(),
                &book.book_ron.base_url,
                &mut errors,
                0,
            )
        };
        // log::trace!("items: {:#?}", items);

        (
            Self {
                items,
                fold_level: book.book_ron.fold_level,
            },
            errors,
        )
    }

    /// Highlight the sidebar item with that url
    pub fn set_active_url(&mut self, url: &str) {
        for item in self.items.iter_mut() {
            item.active = matches!(&item.url, Some(u) if u == url);
            for child in item.children.iter_mut().flat_map(|xs| xs.iter_mut()) {
                child.active = matches!(&child.url, Some(u) if u == url);
            }
        }
    }

    fn collect_sidebar_items<'a>(
        items: impl Iterator<Item = &'a TocItem>,
        src_dir: &Path,
        base_url_str: &str,
        errors: &mut Vec<Error>,
        depth: usize,
    ) -> Vec<SidebarItem> {
        items
            .filter_map(
                |item| match Self::map_item(item, src_dir, base_url_str, errors, depth) {
                    Ok(item) => Some(item),
                    Err(err) => {
                        errors.push(err);
                        None
                    }
                },
            )
            .collect()
    }

    fn map_item(
        item: &TocItem,
        src_dir: &Path,
        base_url_str: &str,
        errors: &mut Vec<Error>,
        depth: usize,
    ) -> Result<SidebarItem> {
        match &item {
            TocItem::File(name, file) => Ok(SidebarItem {
                name: Self::get_title(name, file)?,
                url: Some(Self::get_url(src_dir, file, base_url_str)?),
                children: None,
                active: false,
                depth,
            }),
            TocItem::Dir(toc) => {
                let children = Self::collect_sidebar_items(
                    toc.items.iter(),
                    src_dir,
                    base_url_str,
                    errors,
                    depth + 1,
                );
                // add preface
                Ok(SidebarItem {
                    name: Self::get_title(&toc.name, &toc.summary)?,
                    url: Some(Self::get_url(src_dir, &toc.summary, base_url_str)?),
                    children: Some(Box::new(children)),
                    active: false,
                    depth,
                })
            }
        }
    }
}

/// Context to generate [`HbsInput`]
#[derive(Debug, Clone)]
pub struct HbsContext {
    // pub src_dir: PathBuf,
    // pub base_url: String,
    sidebar: Sidebar,
}

impl HbsContext {
    pub fn from_book(book: &BookStructure) -> (Self, Vec<Error>) {
        let (sidebar, errors) = Sidebar::from_book(book);

        let me = Self {
            // src_dir: book.src_dir_path(),
            // base_url: book.book_ron.base_url.clone(),
            sidebar,
        };

        (me, errors)
    }

    /// Creates sidebar context for an article (highlight the article)
    pub fn sidebar_for_url(&self, url: &str) -> Sidebar {
        let mut s = self.sidebar.clone();
        s.set_active_url(url);
        s
    }
}

// --------------------------------------------------------------------------------
// Data

/// Variables directly supplied to Handlebars templates
#[derive(Serialize, Debug, Clone)]
pub struct HbsInput<'a> {
    /// Used in Handlebars template for hard coding paths
    pub base_url: String,
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

impl<'a> HbsInput<'a> {
    /// WARN: be sure to set `sidebar_items` later
    pub fn new(html: &'a str, meta: &AdocMetadata, base_url: &str, sidebar: Sidebar) -> Self {
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

        HbsInput {
            base_url: base_url.to_string(),
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

/// Setup [`Handlebars`] with user theme files
pub fn init_hbs_user(hbs_dir: &Path) -> Result<Handlebars> {
    ensure!(
        hbs_dir.is_dir(),
        "Unable to find handlebars directory in source directory"
    );

    let mut hbs = Handlebars::new();
    hbs.set_strict_mode(true);

    let partials_dir = hbs_dir.join("partials");
    ensure!(
        partials_dir.is_dir(),
        "Unable to find handlebars partials directory at: {}",
        partials_dir.display(),
    );

    for entry in fs::read_dir(&partials_dir)? {
        let entry = entry.context("Unexpected entry")?;
        let partial = entry.path();

        // filter non-hbs files
        if matches!(partial.extension().and_then(|s| s.to_str()), Some(".hbs")) {
            continue;
        }

        // NOTE: the name is used as key to specify partial files!
        let name = partial
            .file_stem()
            .and_then(|s| s.to_str())
            .context("Unable to stringify partial hbs file path")?;

        // register the hbs file as a partial
        let text = fs::read_to_string(&partial)
            .with_context(|| format!("Unable to load partial hbs file: {}", partial.display()))?;

        hbs.register_partial(name, &text)?;
    }

    Ok(hbs)
}

/// Setup [`Handlebars`] with default theme files
pub fn init_hbs_default() -> Result<Handlebars<'static>> {
    let mut hbs = Handlebars::new();
    hbs.set_strict_mode(true);

    use crate::book::init::files::src::theme::hbs;

    // NOTE: the name is used as key to specify partial files!
    let text = std::str::from_utf8(hbs::partials::SIDEBAR)?;
    hbs.register_partial("sidebar", &text)?;
    let text = std::str::from_utf8(hbs::partials::SIDEBAR_ITEM)?;
    hbs.register_partial("sidebar_item", &text)?;

    Ok(hbs)
}

pub fn render_hbs_user<'a>(
    hbs: &mut Handlebars,
    hbs_input: &HbsInput,
    src_file_name: &str,
    hbs_file: &Path,
) -> Result<String> {
    let key = format!("{}", hbs_file.display());
    hbs.register_template_file(&key, hbs_file)
        .with_context(|| format!("Error when loading hbs file: {}", hbs_file.display()))?;

    let output = hbs
        .render(&key, &hbs_input)
        .with_context(|| format!("Error when converting file {}", src_file_name))?;

    Ok(output)
}

pub fn render_hbs_default<'a>(
    hbs: &mut Handlebars,
    hbs_input: &HbsInput,
    src_file_name: &str,
) -> Result<String> {
    use crate::book::init::files::src::theme::hbs;
    let key = "ARTICLE";

    hbs.register_template_string(key, std::str::from_utf8(hbs::ARTICLE).unwrap())
        .with_context(|| format!("Error when loading builtin hbs template"))?;

    let output = hbs
        .render(&key, &hbs_input)
        .with_context(|| format!("Error when converting file {}", src_file_name))?;

    Ok(output)
}
