//! Preset files embedded to `adbook` binary
//!
//! # Example
//!
//! Print `article.adoc` embedded in `adbook`:
//!
//! ```sh
//! $ adbook preset a # article.adoc
//! ```

/// a | article | article.adoc
pub const ARTICLE_ADOC: &[u8] = include_bytes!("article.adoc");

/// b | book | book.ron
pub const BOOK_RON: &[u8] = include_bytes!("book.ron");

/// t | toc | toc.ron
pub const TOC_RON: &[u8] = include_bytes!("toc.ron");
