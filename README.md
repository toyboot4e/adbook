# adbook

`adbook` is a tool for building book-like web pages

The name came from [mdBook](https://rust-lang.github.io/mdBook/).

## Demo

A demo site is avaiable [here](https://toyboot4e.github.io/adbook/).

The source files of the demo is [here](https://github.com/toyboot4e/adbook/tree/gh-pages).

## Installation

### Rust & adbook

After installing [Rust](https://www.rust-lang.org/), `adbook` is avaiable via crates.io:

```sh
$ cargo install adbook # -> `$HOME/.cargo/bin`
```

> Make sure `$HOME/.cargo/bin` is added to your `PATH`

### Ruby, Asciidoctor and Asciidoctor extensions

You need [Ruby](https://www.ruby-lang.org/en/) and some package manager. I'd recommend [RVM](https://rvm.io/).

[asciidoctor](https://asciidoctor.org) and [asciidoctor-diagram](https://asciidoctor.org/docs/asciidoctor-diagram/) can be installed as gems:

```sh
$ rvm install asciidoctor
$ rvm install asciidoctor-diagram
```
