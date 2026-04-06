---
name: h2m
description: >-
  HTML-to-Markdown converter CLI tool. Use when the user asks to convert HTML
  to Markdown, scrape a web page to Markdown, extract content from URLs, or
  transform HTML files. Supports CommonMark, GFM (tables, strikethrough, task
  lists), reference-style links, CSS selector extraction, and relative URL
  resolution.
---

# H2M — HTML-to-Markdown Converter

`h2m` is a CLI tool that converts HTML into clean Markdown. It supports **CommonMark** and **GitHub Flavored Markdown** (tables, strikethrough, task lists), reference-style links, CSS selector extraction, and relative-to-absolute URL resolution.

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
h2m [OPTIONS] [INPUT]
```

`INPUT` can be a **URL** (`http://` or `https://`), a **file path**, `"-"` for stdin, or omitted to read from stdin.

## CLI Flags

| Flag              | Short | Description                                                      | Default     |
| ----------------- | ----- | ---------------------------------------------------------------- | ----------- |
| `--gfm`           | `-g`  | Enable GFM extensions (tables, strikethrough, task lists)        | off         |
| `--heading-style` |       | Heading style: `atx` or `setext`                                 | `atx`       |
| `--bullet`        |       | Bullet character for unordered lists: `-`, `+`, or `*`           | `-`         |
| `--fence`         |       | Code fence style: `backtick` or `tilde`                          | `backtick`  |
| `--em`            |       | Emphasis delimiter: `*` or `_`                                   | `*`         |
| `--strong`        |       | Strong delimiter: `stars` (`**`) or `underscores` (`__`)         | `stars`     |
| `--hr`            |       | Horizontal rule string                                           | `---`       |
| `--link-style`    |       | Link style: `inlined` or `referenced`                            | `inlined`   |
| `--link-ref`      |       | Reference link style: `full`, `collapsed`, or `shortcut`         | `full`      |
| `--no-escape`     |       | Disable markdown character escaping                              | off         |
| `--domain`        |       | Base domain for resolving relative URLs (auto-detected for URLs) | auto        |
| `--selector`      | `-s`  | CSS selector to extract before converting                        | none        |
| `--output`        | `-o`  | Output file path (writes to stdout if omitted)                   | stdout      |

## Usage Examples

### Basic Conversion

```bash
# Convert a URL directly (domain auto-detected)
h2m https://example.com

# Convert a local HTML file
h2m page.html

# Pipe from stdin
echo '<h1>Hello</h1><p>World</p>' | h2m

# Pipe from curl
curl -s https://example.com | h2m
```

### CSS Selector Extraction

```bash
# Extract only the article content
h2m --selector article https://blog.example.com/post

# Extract by ID
h2m --selector '#content' https://example.com

# Extract main element
curl -s https://example.com | h2m --selector main
```

### GFM Extensions

```bash
# Enable tables, strikethrough, task lists
h2m --gfm https://github.com/user/repo

# GFM with referenced links
h2m --gfm --link-style referenced https://example.com
```

### Formatting Options

```bash
# Setext headings (=== and --- underlines for h1/h2)
h2m --heading-style setext page.html

# Tilde code fences instead of backticks
h2m --fence tilde page.html

# Underscore emphasis and strong
h2m --em _ --strong underscores page.html

# Custom horizontal rule
h2m --hr '***' page.html

# All options combined
h2m --gfm --heading-style setext --strong underscores --fence tilde page.html
```

### Reference-Style Links

```bash
# Full reference: [text][1] with [1]: url footer
h2m --link-style referenced --link-ref full page.html

# Collapsed: [text][] with [text]: url footer
h2m --link-style referenced --link-ref collapsed page.html

# Shortcut: [text] with [text]: url footer
h2m --link-style referenced --link-ref shortcut page.html
```

### Domain Resolution

```bash
# Auto-detected when input is a URL
h2m https://example.com
# relative "/about" becomes "http://example.com/about"

# Manually set for local files or stdin
h2m --domain example.com page.html
curl -s https://example.com | h2m --domain example.com
```

### Output to File

```bash
# Save to file
h2m https://example.com -o output.md

# Combine with selector
h2m --selector article --gfm https://blog.example.com/post -o article.md
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

1. **Use `--selector` for web scraping** to extract only the relevant content (e.g. `article`, `main`, `#content`) and avoid converting navigation, footers, and boilerplate.
2. **Use `--gfm`** when the source HTML contains tables, strikethrough, or checkboxes — without it, these elements are passed through as raw HTML or ignored.
3. **Domain is auto-detected** when the input is a URL. Only set `--domain` manually when piping HTML from stdin or converting local files with relative URLs.
4. **Reference-style links** (`--link-style referenced`) produce cleaner output for documents with many links, keeping the prose readable and link URLs in a footer section.
5. **Pipe-friendly**: `h2m` reads from stdin when no input is given, and writes to stdout by default. Use `-o` to save directly to a file.
6. **CSS selectors** support any valid CSS selector syntax: tag names (`article`), IDs (`#content`), classes (`.post-body`), combinators (`main > article`), etc.
7. **Error handling**: CLI exits with code 1 on errors (network failure, invalid file, invalid selector). Warnings (e.g. selector matched nothing) go to stderr; markdown output goes to stdout.
