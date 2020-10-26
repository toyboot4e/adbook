/*! `adbook` is a tool for building book-like web pages

The name came from [mdbook], which was inspired by [GitBook].

[mdbook]: https://rust-lang.github.io/mdBook/
[GitBook]: https://www.gitbook.com/
!*/

// Globally importing macros: `info!`, `warn!`, etc.
#[macro_use]
extern crate log;

// Globally importing macros: `anyhow!`, `bail!` and `ensure!`
#[macro_use]
extern crate anyhow;

pub mod book;
pub mod build;
pub mod cli;
pub mod utils;
