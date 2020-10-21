/*! `adbook` is a tool for building book-like web pages

The name came from [mdbook], which was inspired by [GitBook].

[mdbook]: https://rust-lang.github.io/mdBook/
[GitBook]: https://www.gitbook.com/

# Installation

## adbook

Install `adbook `in `$HOME/.cargo/bin` (on macOS or Linux):

```sh
$ cargo install adbook
```

To get started, create a new adbook project with `cargo new path/to/the/new/book/directory`.

> I'm 0% sure about Windows though..

## Asciidoctor

You'll need:

* [asciidoctor](https://asciidoctor.org)
* [asciidoctor-diagram](https://asciidoctor.org/docs/asciidoctor-diagram/)

> I recommend using `rvm` over `gem` because it's faster (at least on macOS).

# File structure in adbook projects

adbook project has such a file structure:

```sh
.
├── book.ron  # configuration file
├── site      # `.html` files are outputted here
└── src       # source files are put here
```

Configuration files are written in [Ron]:

* `book.ron` maps to [`BookRon`].
* `toc.ron` maps to [`TocRon`].

[Ron]: https://github.com/ron-rs/ron
[`BookRon`]: crate::book::config::BookRon
[`TocRon`]: crate::book::config::TocRon

`adbook` will look into `src/toc.ron` and searches files in it, recursively:

```sh
└── src
    ├── a.adoc
    ├── sub_directory
    │   ├── b.adoc
    │   └── toc.ron  # alternative to `SUMMARY.md` in `mdbook` or `mod.rs` in Cargo
    └── toc.ron      # alternative to `SUMMARY.md` in `mdbook` or `lib.rs` in Cargo
```
!*/

// Globally importing `info!`, `warn!`, etc.
#[macro_use]
extern crate log;

// Globally importing `anyhow!`, `bail!` and `ensure!`
#[macro_use]
extern crate anyhow;

pub mod book;
pub mod builder;
pub mod cli;
pub mod preset;
pub mod utils;
