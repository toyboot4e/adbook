# adbook

`adbook` is a tool for building book-like web pages

The name came from [mdbook](https://rust-lang.github.io/mdBook/), which was inspired by [GitBook](https://www.gitbook.com/).

## Installation

You need `adbook`, [asciidoctor](https://asciidoctor.org) and [asciidoctor-diagram](https://asciidoctor.org/docs/asciidoctor-diagram/).

### adbook

```sh
$ cargo install adbook # -> `$HOME/.cargo/bin`
```

### Ruby & Asciidoctor

If you have [RVM](https://rvm.io/) in your computer:

```sh
$ rvm install asciidoctor
$ rvm install asciidoctor-diagram
```

> I recommand RVM because commands installed with `gem` are somehow slow (at least on macOS).
