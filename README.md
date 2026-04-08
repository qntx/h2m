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
[ci-badge]: https://github.com/qntx/h2m/actions/workflows/rust.yml/badge.svg
[ci-url]: https://github.com/qntx/h2m/actions/workflows/rust.yml
[license-badge]: https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg
[license-url]: LICENSE-MIT
[rust-badge]: https://img.shields.io/badge/rust-edition%202024-orange.svg
[rust-url]: https://doc.rust-lang.org/edition-guide/

**Fast, extensible HTML-to-Markdown converter for Rust — CommonMark + GFM, plugin architecture, zero `unsafe`.**

H2M converts HTML into clean Markdown with full CommonMark compliance and GitHub Flavored Markdown extensions. It uses a plugin-based rule system, supports reference-style links, relative URL resolution, and ships with an async CLI powered by `tokio` for high-concurrency batch scraping.

<p align="center">
  <img src="demo.gif" alt="H2M CLI Demo"/>
</p>

## Quick Start

### Install the CLI

**Shell** (macOS / Linux):

```sh
curl -fsSL https://sh.qntx.fun/h2m | sh
```

**PowerShell** (Windows):

```powershell
irm https://sh.qntx.fun/h2m/ps | iex
```

Or via Cargo:

```bash
cargo install h2m-cli
```

### CLI Usage

```bash
h2m https://example.com
h2m page.html
curl -s https://example.com | h2m
```

Content extraction:

```bash
h2m -r https://blog.example.com/post             # smart readable
h2m -s article https://blog.example.com/post     # CSS selector
h2m -s '#content' https://example.com            # by ID
```

JSON output (for agents / programmatic use):

```bash
h2m --json https://example.com                   # pretty JSON
h2m --json --extract-links https://example.com   # with links
h2m --json url1 url2 url3                        # NDJSON streaming
h2m --json --urls urls.txt -j 8 --delay 100      # batch + concurrency
```

Formatting:

```bash
h2m --gfm https://example.com                    # tables, strikethrough, task lists
h2m --link-style referenced page.html            # reference-style links
h2m --heading-style setext page.html             # === / --- underlines
h2m --user-agent "MyBot/1.0" https://example.com
h2m -o output.md https://example.com
```

### JSON Output

Single URL produces a pretty-printed JSON object:

```json
{
  "markdown": "# Example Domain\n\n...",
  "metadata": {
    "title": "Example Domain",
    "description": "This domain is for use in illustrative examples.",
    "language": "en",
    "ogImage": "https://example.com/og.png",
    "sourceUrl": "https://example.com",
    "url": "https://example.com/",
    "statusCode": 200,
    "contentType": "text/html; charset=UTF-8",
    "elapsedMs": 234
  },
  "links": ["https://example.com/about"]
}
```

`sourceUrl` is the original request; `url` is the final URL after redirects. `links` only appears with `--extract-links`.

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
    .use_plugin(&CommonMark)
    .use_plugin(&Gfm)
    .domain("example.com")
    .build();

let md = converter.convert(r#"<a href="/about">About</a>"#);
assert_eq!(md, "[About](https://example.com/about)");
```

### Async Scraping

Enable the `scrape` feature for async HTTP scraping with built-in concurrency control, rate limiting, and streaming output:

```rust,no_run
use h2m::scrape::Scraper;

let scraper = Scraper::builder()
    .concurrency(8)
    .gfm(true)
    .extract_links(true)
    .build()?;

// Single scrape
let result = scraper.scrape("https://example.com").await?;
println!("{}", result.markdown);

// Batch with streaming callback
let urls = vec!["https://a.com".into(), "https://b.com".into()];
scraper.scrape_many_streaming(&urls, |result| {
    match result {
        Ok(r) => println!("{}", r.markdown),
        Err(e) => eprintln!("error: {e}"),
    }
}).await;
```

## Design

- **CommonMark + GFM** — full spec compliance with tables, strikethrough, task lists, reference-style links
- **Plugin architecture** — extend with custom rules via the `Rule` trait
- **Async batch pipeline** — `tokio` + `reqwest`, semaphore concurrency, streaming NDJSON (`scrape` feature)
- **JSON output** — nested camelCase metadata (title, description, language, ogImage, sourceUrl/url, statusCode, contentType, elapsedMs) for agent/programmatic consumption
- **Smart readable extraction** — two-phase content detection: semantic selectors → noise stripping (`nav`, `footer`, `aside`, `header`, ARIA roles)
- **Smart scraping** — configurable User-Agent, HTTP 3xx + HTML meta-refresh redirect following (including `<noscript>`-wrapped)
- **Zero-copy fast paths** — `Cow<str>` escaping, zero `unsafe`, `Send + Sync`

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

## Supported HTML Elements

### CommonMark (built-in)

| Element                                  | Markdown Output                           |
| ---------------------------------------- | ----------------------------------------- |
| `<h1>`-`<h6>`                            | `# Heading` (ATX) or underline (Setext)   |
| `<p>`, `<div>`, `<section>`, `<article>` | Block paragraph                           |
| `<strong>`, `<b>`                        | `**bold**`                                |
| `<em>`, `<i>`                            | `*italic*`                                |
| `<code>`, `<kbd>`, `<samp>`, `<tt>`      | `` `inline code` ``                       |
| `<pre><code>`                            | Fenced code block with language detection |
| `<a href="...">`                         | `[text](url)` or reference-style          |
| `<img src="..." alt="...">`              | `![alt](src "title")`                     |
| `<ul>`, `<ol>`, `<li>`                   | Bullet/numbered lists with nesting        |
| `<blockquote>`                           | `> quoted text`                           |
| `<hr>`                                   | `---`                                     |
| `<br>`                                   | Hard line break                           |
| `<iframe>`                               | `[iframe](url)`                           |

### GFM Extensions (with `--gfm`)

| Element                     | Markdown Output               |
| --------------------------- | ----------------------------- |
| `<table>`                   | GFM pipe table with alignment |
| `<del>`, `<s>`, `<strike>`  | `~~strikethrough~~`           |
| `<input type="checkbox">`   | `[x]` or `[ ]` (task list)    |

### Auto-removed

| Element      | Behavior                    |
| ------------ | --------------------------- |
| `<script>`   | Removed (content stripped)  |
| `<style>`    | Removed (content stripped)  |
| `<noscript>` | Removed (content stripped)  |

## Custom Rules

Extend the converter with your own rules by implementing the `Rule` trait:

```rust
use h2m::{Converter, Rule, Action, Context};
use h2m::rules::CommonMark;
use scraper::ElementRef;

#[derive(Debug)]
struct HighlightRule;
impl Rule for HighlightRule {
    fn tags(&self) -> &'static [&'static str] { &["mark"] }

    fn apply(&self, content: &str, _el: &ElementRef<'_>, _ctx: &mut Context<'_>) -> Action {
        Action::Replace(format!("=={content}=="))
    }
}

let mut builder = Converter::builder()
    .use_plugin(CommonMark);
builder.add_rule(HighlightRule);
let converter = builder.build();

let md = converter.convert("<p>This is <mark>important</mark></p>");
assert!(md.contains("==important=="));
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
