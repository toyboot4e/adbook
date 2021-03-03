/*!
`adbook` is a simple SSG powered by [asciidoctor]

The name came from [mdBook], which was inspired by [GitBook].

[asciidoctor]: https://asciidoctor.org/
[mdBook]: https://rust-lang.github.io/mdBook/
[GitBook]: https://www.gitbook.com/
!*/

// Importing macros globally in this crate:

// `info!`, `warn!`, etc.
#[macro_use]
extern crate log;

// `anyhow!`, `bail!` and `ensure!`
#[macro_use]
extern crate anyhow;

pub mod book;
pub mod build;
pub mod cli;
pub mod utils;
