---
name: h2m
description: >-
  HTML-to-Markdown converter CLI with optional web search. Use when the user
  asks to convert HTML to Markdown, scrape web pages to Markdown, batch-scrape
  URLs, search the web (SearXNG / Brave / Tavily) and get structured results,
  or pipe search hits through a scrape-to-Markdown pipeline. The CLI exposes
  two subcommands: `convert` (URL / file / stdin → Markdown) and `search`
  (web search with optional `--scrape` that funnels every hit through the
  same converter). Supports CommonMark, GFM, JSON/NDJSON streaming output
  with nested camelCase metadata, readable extraction, CSS selectors, and
  relative URL resolution.
---

# H2M — HTML-to-Markdown Converter + Web Search

`h2m` is an async CLI that converts HTML into clean Markdown and runs web searches through provider abstractions. It speaks **CommonMark** and **GitHub Flavored Markdown** (tables, strikethrough, task lists), emits **JSON / NDJSON** with nested camelCase metadata for agent consumption, batch-scrapes with concurrency control, and offers **multi-provider web search** (SearXNG, Brave, Tavily) that can either return hit lists or pipe every URL through the converter in a single call.

## Installation

### One-line install (recommended)

**macOS / Linux:**

```sh
curl -fsSL https://sh.qntx.fun/labs/h2m | sh
```

**Windows (PowerShell):**

```powershell
irm https://sh.qntx.fun/labs/h2m/ps | iex
```

These scripts download the latest pre-built binary from GitHub Releases and add it to PATH. No Rust toolchain required.

### Via Cargo

```bash
cargo install h2m-cli
```

### Verify installation

```sh
h2m --version
h2m --help
```

## CLI Structure

H2M 0.6+ uses a subcommand tree (breaking change from 0.5):

```text
h2m <COMMAND> [OPTIONS] ...

Commands:
  convert  Convert HTML to Markdown (URL, file, stdin, pipe)
  search   Search the web and optionally scrape each hit to Markdown
```

Migration from 0.5: wrap every old `h2m <url>` invocation with `h2m convert <url>`. All flags are preserved.

## `h2m convert`

```text
h2m convert [OPTIONS] [INPUT]...
```

`INPUT` can be one or more **URLs** (`http://` or `https://`), **file paths**, `"-"` for stdin, or omitted to read from stdin. Multiple inputs trigger async batch mode.

### `convert` Flags

| Flag              | Short | Description                                                      | Default    |
| ----------------- | ----- | ---------------------------------------------------------------- | ---------- |
| `--json`          |       | JSON output (single → pretty JSON, batch → NDJSON streaming)     | off        |
| `--extract-links` |       | Extract every `<a href>` (included in JSON output)               | off        |
| `--urls`          |       | Read URLs from a file (one per line, `#` comments)               | none       |
| `--concurrency`   | `-j`  | Max concurrent HTTP requests for batch mode                      | `4`        |
| `--delay`         |       | Delay between requests in milliseconds (rate limiting)           | `0`        |
| `--timeout`       |       | HTTP request timeout in seconds                                  | `30`       |
| `--gfm`           | `-g`  | Enable GFM extensions (tables, strikethrough, task lists)        | off        |
| `--heading-style` |       | `atx` or `setext`                                                | `atx`      |
| `--bullet`        |       | `dash`, `plus`, or `star`                                        | `dash`     |
| `--fence`         |       | `backtick` or `tilde`                                            | `backtick` |
| `--em`            |       | `star` or `underscore`                                           | `star`     |
| `--strong`        |       | `stars` (`**`) or `underscores` (`__`)                           | `stars`    |
| `--hr`            |       | `dashes`, `stars`, or `underscores`                              | `dashes`   |
| `--link-style`    |       | `inlined` or `referenced`                                        | `inlined`  |
| `--link-ref`      |       | `full`, `collapsed`, or `shortcut`                               | `full`     |
| `--no-escape`     |       | Disable markdown character escaping                              | off        |
| `--domain`        |       | Base domain for resolving relative URLs (auto-detected for URLs) | auto       |
| `--selector`      | `-s`  | CSS selector to extract before converting                        | none       |
| `--readable`      | `-r`  | Smart readable extraction (semantic selectors → noise stripping) | off        |
| `--user-agent`    |       | Custom `User-Agent` header                                       | `h2m/x.y`  |
| `--output`        | `-o`  | Output file path (writes to stdout if omitted)                   | stdout     |

### `convert` Examples

Basic:

```bash
h2m convert https://example.com
h2m convert page.html
curl -s https://example.com | h2m convert
echo '<h1>Hello</h1>' | h2m convert
```

Content extraction:

```bash
h2m convert -r https://blog.example.com/post             # smart readable
h2m convert -s article https://blog.example.com/post     # CSS selector
h2m convert -s '#content' https://example.com            # by ID
curl -s https://example.com | h2m convert -r             # stdin + readable
```

`-r` uses a two-phase approach:

1. Phase 1 — semantic selectors (`article`, `main`, `[role="main"]`, …)
2. Phase 2 — strip noise (`nav`, `footer`, `aside`, `header`, ARIA roles) if no semantic wrapper is found

`-s` and `-r` are mutually exclusive.

JSON:

```bash
h2m convert --json https://example.com                   # pretty JSON
h2m convert --json --extract-links https://example.com   # with links
h2m convert --json url1 url2 url3                        # NDJSON streaming
h2m convert --json --urls urls.txt -j 8 --delay 100      # batch + concurrency
```

Formatting:

```bash
h2m convert --gfm https://example.com                    # tables, strikethrough
h2m convert --link-style referenced page.html            # reference-style links
h2m convert --link-style referenced --link-ref collapsed # [text][] style
h2m convert --heading-style setext page.html             # === / --- underlines
h2m convert --fence tilde page.html                      # ~~~ code fences
h2m convert --no-escape page.html
```

Output & misc:

```bash
h2m convert --domain example.com page.html               # resolve relative URLs
h2m convert --user-agent "MyBot/1.0" https://example.com
h2m convert -o output.md https://example.com
h2m convert -r --gfm -o article.md https://blog.example.com/post
```

### `convert` JSON Schema

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

- **`sourceUrl`** — the original requested URL
- **`url`** — the final URL after HTTP 3xx and meta-refresh redirects
- **`links`** — only present when `--extract-links` is set
- **`description`**, **`ogImage`** — omitted when not present in the page

Multiple URLs produce NDJSON (one JSON object per line). `--urls` reads from a file (lines starting with `#` are ignored).

## `h2m search`

```text
h2m search [OPTIONS] <QUERY>
```

Runs a web search. The default output is a **pretty JSON `SearchResponse`**; add `--json` for NDJSON (one hit per line); add `--scrape` to route every hit URL through the same scrape-to-Markdown pipeline as `convert`, streaming `ScrapeResult` NDJSON.

### Providers

| Provider | Selected by            | Required env        | Free tier       |
| -------- | ---------------------- | ------------------- | --------------- |
| SearXNG  | `--provider searxng` * | `H2M_SEARXNG_URL`   | yes (self-host) |
| Brave    | `--provider brave`     | `BRAVE_API_KEY`     | $5/month credit |
| Tavily   | `--provider tavily`    | `TAVILY_API_KEY`    | 1000 req/month  |

`*` default provider. Override the default with `H2M_SEARCH_PROVIDER=brave` etc.

**API keys are read from environment variables only** — never pass them on the command line. SearXNG also accepts `--searxng-url` as an override.

### `search` Flags

| Flag             | Short | Description                                                        | Default    |
| ---------------- | ----- | ------------------------------------------------------------------ | ---------- |
| `--provider`     | `-p`  | `searxng`, `brave`, or `tavily`                                    | `searxng`  |
| `--limit`        |       | Max results per source (1..=100)                                   | `10`       |
| `--sources`      |       | Comma-separated `web,news,images`                                  | `web`      |
| `--time-range`   |       | `day`, `week`, `month`, `year`                                     | none       |
| `--country`      |       | ISO 3166-1 alpha-2 country code (e.g. `us`, `cn`)                  | none       |
| `--language`     |       | ISO 639-1 language code (e.g. `en`, `zh`)                          | none       |
| `--safesearch`   |       | `off`, `moderate`, `strict`                                        | `moderate` |
| `--searxng-url`  |       | Override `H2M_SEARXNG_URL`                                         | env        |
| `--scrape`       |       | Scrape every hit URL and emit `ScrapeResult` NDJSON                | off        |
| `--json`         |       | NDJSON one hit per line (pure search mode)                         | off        |
| `--output`       | `-o`  | Output file path                                                   | stdout     |

When `--scrape` is set, all `convert` flags (`--gfm`, `--readable`, `--selector`, `--heading-style`, `--concurrency`, `--timeout`, `--user-agent`, `--extract-links`, etc.) are applied to the scraping stage.

### `search` Examples

Pure search:

```bash
export H2M_SEARXNG_URL=https://searx.example.org

h2m search "rust async trait"                    # pretty JSON response
h2m search "rust async trait" --json             # NDJSON streaming
h2m search "rust" --limit 5 --time-range week
h2m search "rust" --sources web,news --country us --language en
h2m search "rust" --provider brave               # switch provider
h2m search "rust" --safesearch off
```

Search + scrape (one-call pipeline):

```bash
h2m search "rust async" --scrape                             # raw markdown NDJSON
h2m search "rust async" --scrape --gfm --readable            # formatting applied
h2m search "rust async" --scrape --selector article          # scrape only <article>
h2m search "rust" --scrape -j 8 --timeout 20                 # parallel scrape
h2m search "rust" --scrape --extract-links                   # include link lists
```

### `search` JSON Schema

**Default** (pretty JSON `SearchResponse`):

```json
{
  "query": "rust async",
  "provider": "searxng",
  "web": [
    {
      "title": "Rust",
      "url": "https://rust-lang.org",
      "description": "A language empowering everyone.",
      "engine": "duckduckgo"
    }
  ],
  "news": [],
  "images": [],
  "elapsedMs": 312
}
```

**With `--json`** (NDJSON, one `SearchHit` per line):

```jsonl
{"title":"Rust","url":"https://rust-lang.org","description":"...","engine":"duckduckgo"}
{"title":"Rust Foundation","url":"https://foundation.rust-lang.org","description":"..."}
```

**With `--scrape`** (NDJSON, one `ScrapeResult` per line — identical shape to `convert --json url1 url2 ...`):

```jsonl
{"markdown":"# Rust\n\n...","metadata":{"title":"Rust","url":"https://rust-lang.org", ...}, ...}
{"markdown":"# Rust Foundation\n\n...","metadata":{...}, ...}
```

Errors in `--scrape` mode come as `{"error":"...","url":"..."}` lines, mixed in-order with successful results.

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

## Agent Best Practices

1. **Always use subcommands** — `h2m convert <url>` and `h2m search <query>`. The bare `h2m <url>` form from 0.5 no longer works.
2. **Prefer `--json`** for programmatic consumption. Schema is documented above and stable (camelCase, nested `metadata`).
3. **Default search = pretty JSON, search `--json` = NDJSON** — match this to your pipeline: scripts want NDJSON, interactive users want the pretty response.
4. **One-call search + convert** — `h2m search "..." --scrape` is strictly faster than composing `h2m search … | jq -r .url | xargs h2m convert` because it reuses the single async runtime and respects `-j` concurrency end-to-end.
5. **Batch convert** via multiple positional args or `--urls file.txt`. Default concurrency is 4; tune with `-j`. Use `--delay` for rate limiting.
6. **Use `--readable`** to strip nav/footer/boilerplate automatically. Use `--selector` when you need precise control.
7. **`--gfm`** is required when the source contains tables, strikethrough, or checkboxes — without it these render as raw HTML or are dropped.
8. **Check `sourceUrl` vs `url`** in JSON output to detect redirects.
9. **SearXNG setup**: self-host with `docker run -d -p 8888:8080 searxng/searxng:latest` and set `H2M_SEARXNG_URL=http://localhost:8888`. The provider requires JSON output — enable it in `settings.yml`:

    ```yaml
    search:
      formats:
        - html
        - json
    ```

10. **API-key providers** (`brave`, `tavily`) — set the relevant env var once in your shell; never pass keys via CLI flags.
11. **Reference-style links** (`--link-style referenced`) produce cleaner output for documents with many links.
12. **Pipe-friendly**: `h2m convert` reads stdin and writes stdout by default; `h2m search --scrape` streams NDJSON to stdout.
13. **Error handling**: exit code `1` on error. With `--json` or `--scrape`, errors come as JSON objects (`{"error":"...","url":"..."}`); without `--json`, errors go to stderr. Batch / scrape mode reports each failure independently without stopping others.
