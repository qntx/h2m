<!-- markdownlint-disable MD033 MD041 MD036 -->

# H2M

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![CI][ci-badge]][ci-url]
[![License][license-badge]][license-url]
[![Rust][rust-badge]][rust-url]

[crates-badge]: https://img.shields.io/crates/v/h2m.svg
[crates-url]: https://crates.io/crates/h2m
[docs-badge]: https://img.shields.io/docsrs/h2m.svg
[docs-url]: https://docs.rs/h2m
[ci-badge]: https://github.com/qntx-labs/h2m/actions/workflows/rust.yml/badge.svg
[ci-url]: https://github.com/qntx-labs/h2m/actions/workflows/rust.yml
[license-badge]: https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg
[license-url]: LICENSE-MIT
[rust-badge]: https://img.shields.io/badge/rust-edition%202024-orange.svg
[rust-url]: https://doc.rust-lang.org/edition-guide/

**Fast, extensible HTML-to-Markdown converter for Rust — CommonMark + GFM, plugin architecture, zero `unsafe`.**

H2M converts HTML into clean Markdown with full CommonMark compliance and GitHub Flavored Markdown extensions. It uses a plugin-based rule system, supports reference-style links, relative URL resolution, and ships with an async CLI powered by `tokio` for high-concurrency batch fetching.

<p align="center">
  <img src="demo.gif" alt="H2M CLI Demo"/>
</p>

## Quick Start

### Install the CLI

**Shell** (macOS / Linux):

```sh
curl -fsSL https://sh.qntx.fun/labs/h2m | sh
```

**PowerShell** (Windows):

```powershell
irm https://sh.qntx.fun/labs/h2m/ps | iex
```

Or via Cargo:

```bash
cargo install h2m-cli
```

### CLI Usage

```bash
# Convert a URL directly
h2m https://example.com

# Extract only the article content
h2m --selector article https://blog.example.com/post

# Local file with GFM + referenced links, save to file
h2m --gfm --link-style referenced page.html -o output.md

# Pipe from stdin
curl -s https://example.com | h2m --selector main

# JSON output for programmatic / agent consumption
h2m --json https://example.com

# Batch convert multiple URLs (NDJSON streaming output)
h2m --json url1 url2 url3

# Batch from file with concurrency control
h2m --json --urls urls.txt -j 8 --delay 100

# All formatting options
h2m --gfm --heading-style setext --strong underscores --fence tilde page.html
```

### JSON Output

Single URL produces a pretty-printed JSON object:

```json
{
  "url": "https://example.com",
  "domain": "example.com",
  "title": "Example Domain",
  "markdown": "# Example Domain\n\n...",
  "elapsed_ms": 234,
  "content_length": 1256
}
```

Multiple URLs produce NDJSON (one JSON object per line), ideal for streaming pipelines.

### Library Usage

```rust
// One-liner with CommonMark defaults
let md = h2m::convert("<h1>Hello</h1><p>World</p>");
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

let md = converter.convert(r#"<a href="/about">About</a>"#);
assert_eq!(md, "[About](http://example.com/about)");
```

### Async Fetching (feature = `"fetch"`)

Enable the `fetch` feature for async HTTP fetching with built-in concurrency control, rate limiting, and streaming output:

```rust,no_run
use h2m::fetch::Fetcher;

let fetcher = Fetcher::builder()
    .concurrency(8)
    .gfm(true)
    .extract_links(true)
    .build()?;

// Single fetch
let result = fetcher.fetch("https://example.com").await?;
println!("{}", result.markdown);

// Batch with streaming callback
let urls = vec!["https://a.com".into(), "https://b.com".into()];
fetcher.fetch_many_streaming(&urls, |result| {
    match result {
        Ok(r) => println!("{}", r.markdown),
        Err(e) => eprintln!("error: {e}"),
    }
}).await;
```

## Design

- **CommonMark compliant** — headings, paragraphs, emphasis, strong, code blocks, links, images, lists, blockquotes, horizontal rules, line breaks
- **GFM extensions** — tables (with column alignment), strikethrough, task lists
- **Reference-style links** — full (`[text][1]`), collapsed (`[text][]`), and shortcut (`[text]`) styles
- **Domain resolution** — resolve relative URLs to absolute via the `url` crate (WHATWG compliant)
- **Plugin architecture** — extend with custom rules via the `Rule` trait; register with `Converter::builder().use_plugin()`
- **Async HTTP pipeline** — `tokio` + `reqwest` with semaphore-based concurrency, rate limiting, and streaming NDJSON output (feature-gated)
- **JSON / NDJSON output** — structured output for agent/programmatic consumption; single result → JSON, batch → NDJSON
- **HTML utilities** — `html::extract_title()`, `html::extract_links()`, `html::select()` for metadata extraction without full conversion
- **Keep / Remove** — selectively preserve raw HTML tags or strip them entirely
- **CSS selector extraction** — `--selector` flag to convert only matching elements
- **Zero-copy fast paths** — `Cow<str>` for escaping and whitespace normalization; no allocation when input needs no transformation
- **`Send + Sync`** — `Converter` is immutable after build, safe to share across threads (compile-time assertion)
- **Strict linting** — Clippy `pedantic` + `nursery` + `correctness` (deny), zero warnings

## Conversion Examples

**Input HTML:**

```html
<h1>Title</h1>
<p>A <strong>bold</strong> and <em>italic</em> paragraph with <a href="https://example.com">a link</a>.</p>
<ul>
  <li>First item</li>
  <li>Second item</li>
</ul>
<pre><code class="language-rust">fn main() {}</code></pre>
```

**Output Markdown:**

```markdown
# Title

A **bold** and *italic* paragraph with [a link](https://example.com).

- First item
- Second item

​```rust
fn main() {}
​```
```

## Custom Rules

Extend the converter with your own rules by implementing the `Rule` trait:

```rust
use h2m::{Converter, Rule, Action, Context};
use h2m::rules::CommonMark;
use scraper::ElementRef;

struct HighlightRule;

impl Rule for HighlightRule {
    fn tags(&self) -> &'static [&'static str] { &["mark"] }

    fn apply(&self, content: &str, _el: &ElementRef<'_>, _ctx: &mut Context) -> Action {
        Action::Replace(format!("=={content}=="))
    }
}

let converter = Converter::builder()
    .use_plugin(CommonMark)
    .build();
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
