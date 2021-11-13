/*!
Files/directories that are created when initializing a book directory

TODO: Auto gen
*/

use std::{fs, io, path::Path};

pub mod files {
    //! Init files in bytes

    pub static BOOK: &[u8] = include_bytes!("../../init/book.ron");

    pub static EDITOR_CONFIG: &[u8] = include_bytes!("../../init/.editorconfig");
    pub static GIT_IGNORE: &[u8] = include_bytes!("../../init/.gitignore");

    pub mod site {}

    pub mod src {
        pub static INDEX_RON: &[u8] = include_bytes!("../../init/src/index.ron");
        pub static INDEX_ADOC: &[u8] = include_bytes!("../../init/src/index.adoc");
        pub static ARTICLE: &[u8] = include_bytes!("../../init/src/article.adoc");

        pub mod static_ {
            pub mod img {}
        }

        pub mod theme {
            pub static FAVICON: &[u8] = include_bytes!("../../init/src/theme/favicon.svg");
            pub mod hbs {
                pub static ARTICLE: &[u8] = include_bytes!("../../init/src/theme/hbs/article.hbs");

                pub mod partials {
                    pub static SIDEBAR: &[u8] =
                        include_bytes!("../../init/src/theme/hbs/partials/sidebar.hbs");
                    pub static SIDEBAR_ITEM: &[u8] =
                        include_bytes!("../../init/src/theme/hbs/partials/sidebar_item.hbs");
                }
            }
            pub mod css {
                pub static ALL: &[u8] = include_bytes!("../../init/src/theme/css/all.css");
                pub static ARTICLE: &[u8] = include_bytes!("../../init/src/theme/css/article.css");
                pub static TERM: &[u8] = include_bytes!("../../init/src/theme/css/term.css");

                pub mod partials {
                    pub static TERM_ADOC: &[u8] =
                        include_bytes!("../../init/src/theme/css/partials/term_adoc.css");
                    pub static PRISM_OKIDIA: &[u8] =
                        include_bytes!("../../init/src/theme/css/partials/prism_okidia.css");
                }
            }
            pub mod js {
                pub static PRISM: &[u8] = include_bytes!("../../init/src/theme/js/prism.js");
            }
        }
    }
}

/// List of init files relative to root directory
static LIST: &'static [(&str, &[u8])] = {
    use files::src;

    &[
        (".gitignore", files::GIT_IGNORE),
        (".editorconfig", files::EDITOR_CONFIG),
        ("book.ron", files::BOOK),
        ("site", &[]),
        ("src", &[]),
        ("src/index.ron", src::INDEX_RON),
        ("src/index.adoc", src::INDEX_RON),
        ("src/article.adoc", src::ARTICLE),
    ]
};

/// List of theme files relative to `src` directory
static THEME_ITEMS: &'static [(&str, &[u8])] = {
    use files::src::theme::{self, css, hbs, js};

    &[
        //
        ("theme", &[]),
        ("theme/favicon.svg", theme::FAVICON),
        //
        ("theme/hbs", &[]),
        ("theme/hbs/article.hbs", hbs::ARTICLE),
        ("theme/hbs/partials", &[]),
        ("theme/hbs/partials/sidebar.hbs", hbs::partials::SIDEBAR),
        (
            "theme/hbs/partials/sidebar_item.hbs",
            hbs::partials::SIDEBAR_ITEM,
        ),
        //
        ("theme/css", &[]),
        ("theme/css/all.css", css::ALL),
        ("theme/css/article.css", css::ARTICLE),
        ("theme/css/term.css", css::TERM),
        ("theme/css/partials", &[]),
        ("theme/css/partials/term_adoc.css", css::partials::TERM_ADOC),
        (
            "theme/css/partials/prism_okidia.css",
            css::partials::PRISM_OKIDIA,
        ),
        //
        ("theme/js", &[]),
        ("theme/js/prism.js", js::PRISM),
    ]
};

/// Non-recursive directory creation
fn gen_dir(path: &Path) -> io::Result<bool> {
    if !path.exists() {
        fs::create_dir(path)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn gen_file(path: &Path, bytes: impl AsRef<[u8]>) -> io::Result<bool> {
    if !path.exists() {
        fs::write(path, bytes)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Generates initial file structure without the `theme` directory
pub fn gen_init_files(base_dir: &Path) -> std::io::Result<()> {
    for (rel_path, bytes) in LIST.iter() {
        let path = base_dir.join(rel_path);
        log::trace!("{}", path.display());

        if bytes.is_empty() {
            gen_dir(&path)?;
        } else {
            gen_file(&path, bytes)?;
        }
    }

    // create `src/static/img`
    let path = base_dir.join("src/static");
    gen_dir(&path)?;
    let path = base_dir.join("src/static/img");
    gen_dir(&path)?;

    Ok(())
}

pub fn copy_default_theme(target_dir: &Path) -> std::io::Result<()> {
    // create `theme` directory
    let path = target_dir.join("theme");
    gen_dir(&path)?;

    for (rel_path, bytes) in THEME_ITEMS.iter() {
        let path = target_dir.join(rel_path);
        log::trace!("copy builtin theme item {}", path.display());

        if bytes.is_empty() {
            gen_dir(&path)?;
        } else {
            gen_file(&path, bytes)?;
        }
    }

    Ok(())
}
