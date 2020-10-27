//! Files/directories that are created when initializing a book directory
//!
//! I wanted to do it automatically, but not sure how to do it.

use std::path::Path;

pub mod files {
    //! Init files in bytes

    pub static BOOK: &[u8] = include_bytes!("../../init/book.ron");

    pub static EDITOR_CONFIG: &[u8] = include_bytes!("../../init/.editorconfig");
    pub static GIT_IGNORE: &[u8] = include_bytes!("../../init/.gitignore");

    pub mod site {}

    pub mod src {
        pub static TOC: &[u8] = include_bytes!("../../init/src/toc.ron");
        pub static INDEX: &[u8] = include_bytes!("../../init/src/index.adoc");
        pub static ARTICLE: &[u8] = include_bytes!("../../init/src/article.adoc");

        pub mod static_ {
            pub mod img {}
        }

        pub mod theme {
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
                pub static TERM: &[u8] = include_bytes!("../../init/src/theme/css/term.css");
                pub mod partials {
                    pub static ADOC: &[u8] =
                        include_bytes!("../../init/src/theme/css/partials/adoc.css");
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

/// List of init files
pub static LIST: &[(&str, &[u8]); 23] = {
    use files::src::{
        self,
        theme::{css, hbs, js},
    };

    &[
        // 3
        ("book.ron", files::BOOK),
        (".gitignore", files::GIT_IGNORE),
        (".editorconfig", files::EDITOR_CONFIG),
        // 1
        ("site", &[]),
        // 6
        ("src", &[]),
        ("src/toc.ron", src::TOC),
        ("src/index.adoc", src::INDEX),
        ("src/article.adoc", src::ARTICLE),
        ("src/static", &[]),
        ("src/static/img", &[]),
        // 6
        ("src/theme", &[]),
        ("src/theme/hbs", &[]),
        ("src/theme/hbs/article.hbs", hbs::ARTICLE),
        ("src/theme/hbs/partials", &[]),
        ("src/theme/hbs/partials/sidebar.hbs", hbs::partials::SIDEBAR),
        (
            "src/theme/hbs/partials/sidebar_item.hbs",
            hbs::partials::SIDEBAR_ITEM,
        ),
        // 5
        ("src/theme/css", &[]),
        ("src/theme/css/term.css", css::TERM),
        ("src/theme/css/partials", &[]),
        ("src/theme/css/partials/adoc.css", css::partials::ADOC),
        (
            "src/theme/css/partials/prism_okidia.css",
            css::partials::PRISM_OKIDIA,
        ),
        // 2
        ("src/theme/js", &[]),
        ("src/theme/js/prism.js", js::PRISM),
    ]
};

/// Generates init file structure
pub fn gen_init_files(base_dir: &Path) -> std::io::Result<()> {
    use std::{fs, io};

    // helpers
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

    for (rel_path_src, bytes) in LIST.iter() {
        let path = base_dir.join(rel_path_src);
        log::trace!("{}", path.display());
        if bytes.is_empty() {
            gen_dir(&path)?;
        } else {
            gen_file(&path, bytes)?;
        }
    }

    Ok(())
}
