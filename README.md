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
[ci-badge]: https://github.com/qntx/h2m/actions/workflows/ci.yml/badge.svg
[ci-url]: https://github.com/qntx/h2m/actions/workflows/ci.yml
[license-badge]: https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg
[license-url]: LICENSE-MIT
[rust-badge]: https://img.shields.io/badge/rust-edition%202024-orange.svg
[rust-url]: https://doc.rust-lang.org/edition-guide/

**Fast, extensible HTML-to-Markdown converter with optional web search — CommonMark + GFM, plugin architecture.**

H2M converts HTML into clean Markdown with full CommonMark compliance and GitHub Flavored Markdown extensions. It uses a plugin-based rule system, supports reference-style links, relative URL resolution, and ships with an async CLI that can also **search the web** and pipe results through the same conversion pipeline (compatible with SearXNG, Brave Search, and Tavily).

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

### CLI Structure

H2M uses a subcommand tree:

```text
h2m <COMMAND> [OPTIONS] ...

Commands:
  convert  Convert HTML to Markdown (URL, file, stdin)
  search   Search the web and optionally scrape each hit to Markdown
```

### `convert` — HTML → Markdown

```bash
h2m convert https://example.com
h2m convert page.html
curl -s https://example.com | h2m convert
echo '<h1>Hi</h1>' | h2m convert
```

Content extraction:

```bash
h2m convert -r https://blog.example.com/post          # smart readable
h2m convert -s article https://blog.example.com/post  # CSS selector
h2m convert -s '#content' https://example.com         # by ID
```

JSON output (agents / programmatic use):

```bash
h2m convert --json https://example.com                # pretty JSON
h2m convert --json --extract-links https://example.com
h2m convert --json url1 url2 url3                     # NDJSON streaming
h2m convert --json --urls urls.txt -j 8 --delay 100
```

Formatting:

```bash
h2m convert --gfm https://example.com                 # tables, strikethrough, task lists
h2m convert --link-style referenced page.html         # reference-style links
h2m convert --heading-style setext page.html          # === / --- underlines
h2m convert --user-agent "MyBot/1.0" https://example.com
h2m convert -o output.md https://example.com
```

### `search` — Web search

H2M supports three search providers. Pick one via `--provider` or the
`H2M_SEARCH_PROVIDER` environment variable:

| Provider | Requires          | Free tier       | Notes                            |
| -------- | ----------------- | --------------- | -------------------------------- |
| SearXNG  | `H2M_SEARXNG_URL` | yes (self-host) | Default. Open-source meta-search |
| Brave    | `BRAVE_API_KEY`   | $5/month credit | Independent index                |
| Tavily   | `TAVILY_API_KEY`  | 1000 req/month  | AI-tuned snippets                |

Pure search (returns titles/URLs/descriptions):

```bash
# Point at any SearXNG instance (self-host or public)
export H2M_SEARXNG_URL=https://searx.example.org

h2m search "rust async trait"                    # pretty JSON response
h2m search "rust async trait" --json             # NDJSON (one hit per line)
h2m search "rust" --limit 5 --time-range week
h2m search "rust" --sources web,news --country us
h2m search "rust" --provider brave               # switch provider
h2m search "rust" --provider tavily --include-answer  # LLM-generated summary
```

Tips:

- **Windows + system proxy** — if your system proxy intercepts `localhost`
  requests (Clash/V2Ray/etc), set `NO_PROXY=127.0.0.1,localhost` before
  pointing `h2m` at a self-hosted SearXNG instance.
- **Brave pagination** — `--limit` up to 200 is supported (Brave caps at
  20 per page; `h2m` paginates transparently via `offset`).

Search + scrape (runs every hit through the full `convert` pipeline,
streams NDJSON ScrapeResults):

```bash
h2m search "rust async" --scrape                 # raw markdown per hit
h2m search "rust async" --scrape --gfm --readable
h2m search "rust async" --scrape --selector article
h2m search "rust" --scrape -j 8 --timeout 20     # parallel scrape
```

### JSON Output

**`convert` single URL** (pretty JSON):

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

**`search` response**:

```json
{
  "query": "rust async",
  "provider": "tavily",
  "answer": "Rust's async trait support stabilized in 1.75 ...",
  "web": [
    {
      "title": "Rust", "url": "https://rust-lang.org",
      "description": "...", "engine": "duckduckgo", "score": 0.92
    }
  ],
  "news": [],
  "images": [],
  "elapsedMs": 312
}
```

- `answer` — LLM-generated summary (Tavily `--include-answer` flag, opt-in).
- `score` — relevance in `[0, 1]` (Tavily only; other providers omit it).
- `engine` — upstream backend name (SearXNG only; aggregators omit it).

Fields marked `Option` are dropped from the JSON when absent, keeping output lean.

Multiple inputs (convert batch, or `search --scrape`) stream NDJSON — one JSON object per line.

### Library Usage

```rust
// One-liner with CommonMark defaults
let md = h2m::convert("<h1>Hello</h1><p>World</p>");
assert_eq!(md, "# Hello\n\nWorld");
```

```rust
// Full control with the builder
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

let result = scraper.scrape("https://example.com").await?;
println!("{}", result.markdown);

let urls = vec!["https://a.com".into(), "https://b.com".into()];
scraper.scrape_many_streaming(&urls, |result| {
    match result {
        Ok(r) => println!("{}", r.markdown),
        Err(e) => eprintln!("error: {e}"),
    }
}).await;
```

### Web Search

The `h2m-search` crate exposes the same provider abstraction the CLI uses:

```rust,no_run
use h2m_search::{SearchClient, SearchQuery};

let client = SearchClient::builder()
    .provider("searxng")
    .searxng_url("https://searx.example.org")
    .build()?;

let response = client
    .search(&SearchQuery::new("rust async").with_limit(5))
    .await?;

for hit in &response.web {
    println!("{} — {}", hit.title, hit.url);
}
# Ok::<_, Box<dyn std::error::Error>>(())
```

## Design

- **CommonMark + GFM** — full spec compliance with tables, strikethrough, task lists, reference-style links
- **Plugin architecture** — extend with custom rules via the `Rule` trait
- **Async batch pipeline** — `tokio` + `reqwest`, semaphore concurrency, streaming NDJSON (`scrape` feature)
- **Multi-provider search** — `SearchClient` enum with static dispatch, one Cargo feature per provider
- **Search + scrape composition** — `search --scrape` funnels hits through the same `Scraper` pipeline, reusing all formatting / extraction flags
- **JSON output** — nested camelCase metadata aligned with Firecrawl conventions
- **Smart readable extraction** — two-phase content detection: semantic selectors → noise stripping
- **Zero-copy fast paths** — `Cow<str>` escaping, zero `unsafe`, `Send + Sync`

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
