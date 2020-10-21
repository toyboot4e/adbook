//! Preset files embedded to `adbook` binary

pub const BOOK_RON: &[u8] = include_bytes!("book.ron");
pub const TOC_RON: &[u8] = include_bytes!("toc.ron");
pub const ARTICLE_ADOC: &[u8] = include_bytes!("article.adoc");
