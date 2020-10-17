/*! `adbook` is a tool for building book-like web pages

The name came from [mdbook], which was inspired by [GitBook].

[mdbook]: https://rust-lang.github.io/mdBook/
[GitBook]: https://www.gitbook.com/

# Installation

The following command will install `adbook `in `$HOME/.cargo/bin` (on macOS or Linux):

```sh
$ cargo install adbook
```

To get started, create a new book project with `cargo new path/for/your/book/directory`.

> I'm 0% sure about Windows though..

# Supported source file formats

* AsciiDoc via [AsciiDoctor] in `PATH`
* markdown via TODO: what?

[AsciiDoctor]: https://asciidoctor.org/

# File structure in adbook projects

adbook project has such a file structure:

```
.
├── book.ron  #
├── site      # `.html` files are outputted here
└── src       # source files are put here
    └── toc.ron
```

Configuration files are written in [Ron]:

* `book.ron` maps to [`BookRon`].
* `toc.ron` maps to [`TocRon`].

[Ron]: https://github.com/ron-rs/ron
[`BookRon`]: crate::book::config::BookRon
[`TocRon`]: crate::book::config::TocRon
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
