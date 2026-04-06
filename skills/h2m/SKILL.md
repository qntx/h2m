---
name: h2m
description: >-
  HTML-to-Markdown converter CLI tool. Use when the user asks to convert HTML
  to Markdown, scrape web pages to Markdown, batch-scrape multiple URLs, extract
  structured data from web pages via JSON output, or transform HTML files.
  Supports CommonMark, GFM (tables, strikethrough, task lists), async batch
  scraping with concurrency control, JSON/NDJSON structured output with nested
  camelCase metadata, reference-style links, CSS selector extraction, smart
  readable extraction, and relative URL resolution.
---

# H2M — HTML-to-Markdown Converter

`h2m` is an async CLI tool that converts HTML into clean Markdown. It supports **CommonMark** and **GitHub Flavored Markdown** (tables, strikethrough, task lists), **JSON/NDJSON structured output** with nested camelCase metadata for agent consumption, **async batch scraping** with concurrency control, reference-style links, CSS selector extraction, smart readable extraction, and relative-to-absolute URL resolution.

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
```

## CLI Structure

```text
h2m [OPTIONS] [INPUT]...
```

`INPUT` can be one or more **URLs** (`http://` or `https://`), **file paths**, `"-"` for stdin, or omitted to read from stdin. Multiple inputs trigger async batch mode.

## CLI Flags

| Flag               | Short | Description                                                      | Default     |
| ------------------ | ----- | ---------------------------------------------------------------- | ----------- |
| `--json`           |       | JSON output (single → pretty JSON, batch → NDJSON streaming)     | off         |
| `--extract-links`  |       | Extract all links from the page (included in JSON output)        | off         |
| `--urls`           |       | Read URLs from a file (one per line, `#` comments supported)     | none        |
| `--concurrency`    | `-j`  | Max concurrent HTTP requests for batch mode                      | `4`         |
| `--delay`          |       | Delay between requests in milliseconds (rate limiting)           | `0`         |
| `--timeout`        |       | HTTP request timeout in seconds                                  | `30`        |
| `--gfm`            | `-g`  | Enable GFM extensions (tables, strikethrough, task lists)        | off         |
| `--heading-style`  |       | Heading style: `atx` or `setext`                                 | `atx`       |
| `--bullet`         |       | Bullet character for unordered lists: `dash`, `plus`, or `star`  | `dash`      |
| `--fence`          |       | Code fence style: `backtick` or `tilde`                          | `backtick`  |
| `--em`             |       | Emphasis delimiter: `star` or `underscore`                       | `star`      |
| `--strong`         |       | Strong delimiter: `stars` (`**`) or `underscores` (`__`)         | `stars`     |
| `--hr`             |       | Horizontal rule style: `dashes`, `stars`, or `underscores`       | `dashes`    |
| `--link-style`     |       | Link style: `inlined` or `referenced`                            | `inlined`   |
| `--link-ref`       |       | Reference link style: `full`, `collapsed`, or `shortcut`         | `full`      |
| `--no-escape`      |       | Disable markdown character escaping                              | off         |
| `--domain`         |       | Base domain for resolving relative URLs (auto-detected for URLs) | auto        |
| `--selector`       | `-s`  | CSS selector to extract before converting                        | none        |
| `--readable`       | `-r`  | Smart readable extraction (semantic selectors → noise stripping) | off         |
| `--output`         | `-o`  | Output file path (writes to stdout if omitted)                   | stdout      |

## Usage Examples

### Basic

```bash
h2m https://example.com
h2m page.html
curl -s https://example.com | h2m
echo '<h1>Hello</h1>' | h2m
```

### Content Extraction

```bash
h2m -r https://blog.example.com/post             # smart readable
h2m -s article https://blog.example.com/post     # CSS selector
h2m -s '#content' https://example.com            # by ID
curl -s https://example.com | h2m -r             # stdin + readable
```

`-r` (`--readable`) uses a two-phase approach:

1. **Phase 1**: Tries semantic selectors (`article`, `main`, `[role="main"]`, …)
2. **Phase 2**: If none found, strips noise (`nav`, `footer`, `aside`, `header`, ARIA roles)

`-s` (`--selector`) and `-r` are mutually exclusive.

### JSON Output

```bash
h2m --json https://example.com                   # pretty JSON
h2m --json --extract-links https://example.com   # with links
h2m --json url1 url2 url3                        # NDJSON streaming
h2m --json --urls urls.txt -j 8 --delay 100      # batch + concurrency
h2m --json --timeout 60 https://slow-site.com    # custom timeout
```

JSON output schema:

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
  "links": ["https://example.com/about", "https://example.com/contact"]
}
```

- **`sourceUrl`** — the original requested URL
- **`url`** — the final URL after HTTP 3xx and meta-refresh redirects
- **`links`** — only present when `--extract-links` is set
- **`description`**, **`ogImage`** — omitted when not present in the page

Multiple URLs produce NDJSON (one JSON object per line). `--urls` reads from a file (lines starting with `#` are ignored).

### Formatting

```bash
h2m --gfm https://example.com                    # tables, strikethrough, task lists
h2m --link-style referenced page.html            # reference-style links
h2m --link-style referenced --link-ref collapsed # [text][] style
h2m --heading-style setext page.html             # === / --- underlines
h2m --fence tilde page.html                      # ~~~ code fences
h2m --bullet star page.html                      # * instead of -
h2m --hr underscores page.html                   # ___ instead of ---
h2m --em underscore --strong underscores page.html
h2m --no-escape page.html                        # disable markdown escaping
```

### Other Options

```bash
h2m --domain example.com page.html               # resolve relative URLs
h2m --user-agent "MyBot/1.0" https://example.com
h2m -o output.md https://example.com             # save to file
h2m -r --gfm -o article.md https://blog.example.com/post
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

## Agent Best Practices

1. **Use `--json` for structured output** — always prefer `--json` when consuming h2m output programmatically. JSON uses **camelCase** field names and a nested `metadata` object containing title, description, language, ogImage, sourceUrl, url, statusCode, contentType, and elapsedMs. For batch operations, output is NDJSON (one JSON per line) for easy streaming.
2. **Use `--json --extract-links`** to get both the converted markdown and all page links in one call — useful for crawling or building site maps.
3. **Check `sourceUrl` vs `url`** — `sourceUrl` is the original request; `url` is the final URL after HTTP 3xx and meta-refresh redirects. Compare them to detect redirects.
4. **Batch multiple URLs** by passing them as arguments or via `--urls file.txt`. Requests run concurrently (default 4, configurable with `-j`). Use `--delay` for rate limiting.
5. **Use `--readable` for web scraping** to automatically extract main content and strip navigation, footers, and boilerplate. Use `--selector` when you need precise control over which element to extract.
6. **Use `--gfm`** when the source HTML contains tables, strikethrough, or checkboxes — without it, these elements are passed through as raw HTML or ignored.
7. **Domain is auto-detected** when the input is a URL. Only set `--domain` manually when piping HTML from stdin or converting local files with relative URLs.
8. **Reference-style links** (`--link-style referenced`) produce cleaner output for documents with many links, keeping the prose readable and link URLs in a footer section.
9. **Pipe-friendly**: `h2m` reads from stdin when no input is given, and writes to stdout by default. Use `-o` to save directly to a file.
10. **CSS selectors** support any valid CSS selector syntax: tag names (`article`), IDs (`#content`), classes (`.post-body`), combinators (`main > article`), etc.
11. **Error handling**: CLI exits with code 1 on errors. With `--json`, errors are output as JSON objects (`{"error": "...", "url": "..."}`). Without `--json`, errors go to stderr. In batch mode, individual failures don't stop other URLs from being processed.
