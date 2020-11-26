# adbook

`adbook` is a tool for building book-like web pages

The name came from [mdbook](https://rust-lang.github.io/mdBook/), which was inspired by [GitBook](https://www.gitbook.com/).

## Demo

A demo site is avaiable [here](https://toyboot4e.github.io/adbook/).

The source files of the demo is [here](https://github.com/toyboot4e/adbook/tree/gh-pages).

## Installation

### Rust & adbook

After installing [Rust](https://www.rust-lang.org/), `adbook` is avaiable via crates.io:

```sh
$ cargo install adbook # -> `$HOME/.cargo/bin`
```

> Be sure to set your `PATH` to `$HOME/.cargo/bin`

### Ruby & Asciidoctor

You need [Ruby](https://www.ruby-lang.org/en/) and some package manager. I recommend [RVM](https://rvm.io/) for its speed.

[asciidoctor](https://asciidoctor.org) and [asciidoctor-diagram](https://asciidoctor.org/docs/asciidoctor-diagram/) can be installed as gems:

```sh
$ rvm install asciidoctor
$ rvm install asciidoctor-diagram
```
