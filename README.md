<!-- markdownlint-disable MD033 MD041 MD036 -->

# H2M

A fast, extensible HTML-to-Markdown converter for Rust.

## Features

- **CommonMark compliant** — headings, paragraphs, emphasis, strong, code, links, images, lists, blockquotes, horizontal rules, line breaks
- **GFM extensions** — tables (with alignment), strikethrough, task lists
- **Reference-style links** — full, collapsed, and shortcut styles
- **Domain resolution** — resolve relative URLs to absolute
- **Plugin architecture** — extend with custom rules via the `Rule` trait
- **Keep/Remove** — selectively preserve or strip HTML tags
- **Zero-copy fast paths** — `Cow<str>` for escaping and whitespace normalization
- **`Send + Sync`** — safe to share across threads

## Quick start

### Library

```rust
// One-liner with CommonMark defaults
let md = h2m::convert("<h1>Hello</h1><p>World</p>").unwrap();
assert_eq!(md, "# Hello\n\nWorld");
```

```rust
// Full control with builder
use h2m::{Converter, Options};
use h2m::plugins::Gfm;
use h2m::rules::CommonMark;

let converter = Converter::builder()
    .options(Options::default())
    .use_plugin(CommonMark)
    .use_plugin(Gfm)
    .domain("example.com")
    .build();

let md = converter.convert(r#"<a href="/about">About</a>"#).unwrap();
assert_eq!(md, "[About](http://example.com/about)");
```

### CLI

```sh
# Convert a URL directly
h2m https://example.com

# Extract only the article content
h2m --selector article https://blog.example.com/post

# Local file with GFM + referenced links, save to file
h2m --gfm --link-style referenced page.html -o output.md

# Pipe from stdin
curl -s https://example.com | h2m --selector main
```

Run `h2m --help` for all options.

## Architecture

```
h2m (library)
├── rules/          CommonMark built-in rules (heading, link, list, ...)
├── plugins/        GFM extensions (table, strikethrough, task list)
├── converter.rs    Builder + frozen converter + DOM traversal
├── context.rs      Traversal state + list pre-pass
├── escape.rs       Markdown character escaping
├── whitespace.rs   Whitespace normalization
└── utils.rs        DOM helpers + URL resolution

h2m-cli (binary)
└── main.rs         CLI with URL/file/stdin input, CSS selector, all options
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project shall be dual-licensed as above, without any additional terms or conditions.

---

<div align="center">

A **[QNTX](https://qntx.fun)** open-source project.

<a href="https://qntx.fun"><img alt="QNTX" width="369" src="https://raw.githubusercontent.com/qntx/.github/main/profile/qntx-banner.svg" /></a>

<!--prettier-ignore-->
Code is law. We write both.

</div>
