//! HTML inspection utilities for extracting metadata from parsed documents.
//!
//! These functions operate on raw HTML strings and are always available
//! (no feature flags required).

use std::collections::HashSet;
use std::fmt::Write;

use scraper::Html;

/// Extracts the `<title>` text from an HTML document.
///
/// Returns `None` if the document has no `<title>` element or if it is empty.
///
/// # Examples
///
/// ```
/// let title = h2m::html::extract_title("<html><head><title>Hello</title></head></html>");
/// assert_eq!(title.as_deref(), Some("Hello"));
/// ```
#[must_use]
pub fn extract_title(html: &str) -> Option<String> {
    extract_title_doc(&Html::parse_document(html))
}

/// Extracts all `<a href="…">` link URLs from an HTML document.
///
/// Fragment-only links (`#section`) and empty `href` values are excluded.
///
/// # Examples
///
/// ```
/// let links = h2m::html::extract_links(r##"<a href="/about">About</a><a href="#top">Top</a>"##);
/// assert_eq!(links, vec!["/about"]);
/// ```
#[must_use]
pub fn extract_links(html: &str) -> Vec<String> {
    extract_links_doc(&Html::parse_document(html), None)
}

/// Extracts all `<a href="…">` link URLs, resolving relative URLs against
/// the given base URL.
///
/// # Examples
///
/// ```
/// let links = h2m::html::extract_links_with_base(
///     r##"<a href="/about">About</a>"##,
///     "https://example.com/page",
/// );
/// assert_eq!(links, vec!["https://example.com/about"]);
/// ```
#[must_use]
pub fn extract_links_with_base(html: &str, base_url: &str) -> Vec<String> {
    let base = url::Url::parse(base_url).ok();
    extract_links_doc(&Html::parse_document(html), base.as_ref())
}

/// Applies a CSS selector to an HTML document and returns the concatenated
/// inner HTML of all matching elements.
///
/// Returns the original HTML unchanged if the selector is invalid or matches
/// no elements.
///
/// # Examples
///
/// ```
/// let html = r#"<div id="a">Hello</div><div id="b">World</div>"#;
/// let result = h2m::html::select(html, "#a");
/// assert_eq!(result, "Hello");
/// ```
#[must_use]
pub fn select(html: &str, selector: &str) -> String {
    select_doc(&Html::parse_document(html), html, selector)
}

/// Attempts to detect and extract the main content area of an HTML document.
///
/// Tries common selectors (`article`, `main`, `[role="main"]`, etc.) in
/// priority order and returns the inner HTML of the first non-empty match.
/// Returns `None` if no main content area is detected.
///
/// # Examples
///
/// ```
/// let html = "<nav>menu</nav><article><p>Hello</p></article>";
/// assert_eq!(h2m::html::detect_main_content(html).as_deref(), Some("<p>Hello</p>"));
/// ```
#[must_use]
pub fn detect_main_content(html: &str) -> Option<String> {
    detect_main_content_doc(&Html::parse_document(html))
}

/// Extracts readable content from an HTML document.
///
/// Two-phase approach:
/// 1. Tries semantic selectors (`article`, `main`, `[role="main"]`, …).
/// 2. If none match, strips noise elements (`nav`, `footer`, `aside`, …)
///    and returns the cleaned HTML.
///
/// # Examples
///
/// ```
/// let html = "<nav>menu</nav><div><p>Hello world</p></div><footer>ft</footer>";
/// let result = h2m::html::readable_content(html);
/// assert!(!result.contains("menu"));
/// assert!(!result.contains("ft"));
/// assert!(result.contains("Hello world"));
/// ```
#[must_use]
pub fn readable_content(html: &str) -> String {
    let doc = Html::parse_document(html);
    readable_content_doc(&doc, html)
}

/// Strips noise elements (`nav`, `footer`, `aside`, …) from an HTML document.
///
/// Returns the cleaned HTML with navigational chrome removed.
///
/// # Examples
///
/// ```
/// let html = "<nav>menu</nav><p>content</p><footer>ft</footer>";
/// let result = h2m::html::strip_noise(html);
/// assert!(!result.contains("menu"));
/// assert!(!result.contains("ft"));
/// assert!(result.contains("content"));
/// ```
#[must_use]
pub fn strip_noise(html: &str) -> String {
    let doc = Html::parse_document(html);
    strip_noise_doc(&doc, html)
}

/// Extracts `<title>` text from a pre-parsed document.
pub(crate) fn extract_title_doc(doc: &Html) -> Option<String> {
    let sel = scraper::Selector::parse("title").ok()?;
    doc.select(&sel)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_owned())
        .filter(|t| !t.is_empty())
}

/// Extracts all `<a href>` link URLs from a pre-parsed document.
///
/// When `base` is provided, relative URLs are resolved to absolute.
pub(crate) fn extract_links_doc(doc: &Html, base: Option<&url::Url>) -> Vec<String> {
    let Ok(sel) = scraper::Selector::parse("a[href]") else {
        return Vec::new();
    };
    doc.select(&sel)
        .filter_map(|el| el.value().attr("href"))
        .filter(|href| !href.is_empty() && !href.starts_with('#'))
        .map(|href| resolve_href(href, base))
        .collect()
}

/// Applies a CSS selector to a pre-parsed document, returning matched inner
/// HTML or the `original` string if nothing matches.
pub(crate) fn select_doc(doc: &Html, original: &str, selector: &str) -> String {
    collect_inner_html(doc, selector).unwrap_or_else(|| original.to_owned())
}

/// Extracts the `lang` attribute from the `<html>` element.
pub(crate) fn extract_language_doc(doc: &Html) -> Option<String> {
    let sel = scraper::Selector::parse("html").ok()?;
    doc.select(&sel)
        .next()
        .and_then(|el| el.value().attr("lang"))
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}

/// Extracts the `<meta name="description">` content.
pub(crate) fn extract_description_doc(doc: &Html) -> Option<String> {
    extract_meta_attr(doc, r#"meta[name="description"]"#, "content")
}

/// Extracts the Open Graph image URL (`<meta property="og:image">`).
pub(crate) fn extract_og_image_doc(doc: &Html) -> Option<String> {
    extract_meta_attr(doc, r#"meta[property="og:image"]"#, "content")
}

/// CSS selectors tried in priority order for automatic main-content detection.
const MAIN_CONTENT_SELECTORS: &[&str] = &[
    "article",
    "[role=\"main\"]",
    "main",
    ".post-content",
    ".entry-content",
    "#content",
    ".content",
];

/// Elements that are almost always navigational/chrome noise, not content.
const NOISE_SELECTORS: &[&str] = &[
    "nav",
    "footer",
    "aside",
    "header",
    "[role=\"navigation\"]",
    "[role=\"banner\"]",
    "[role=\"contentinfo\"]",
    "[role=\"complementary\"]",
    "[role=\"search\"]",
    "[aria-hidden=\"true\"]",
];

/// Auto-detects main content from a pre-parsed document.
pub(crate) fn detect_main_content_doc(doc: &Html) -> Option<String> {
    MAIN_CONTENT_SELECTORS
        .iter()
        .find_map(|sel| collect_inner_html(doc, sel))
}

/// Extracts readable content from a pre-parsed document.
///
/// Two-phase approach:
/// 1. Try semantic selectors (`article`, `main`, `[role="main"]`, …).
/// 2. If none match, strip noise elements (`nav`, `footer`, `aside`, …)
///    and return the cleaned body HTML.
pub(crate) fn readable_content_doc(doc: &Html, original: &str) -> String {
    // Phase 1: semantic selectors
    if let Some(main) = detect_main_content_doc(doc) {
        return main;
    }
    // Phase 2: noise stripping fallback
    strip_noise_doc(doc, original)
}

/// Strips noise elements from a pre-parsed document, returning cleaned HTML.
///
/// Walks the DOM tree and re-serializes it while skipping subtrees that match
/// [`NOISE_SELECTORS`]. Returns `original` unchanged if no noise is found.
pub(crate) fn strip_noise_doc(doc: &Html, original: &str) -> String {
    let noise_ids = collect_noise_ids(doc);
    if noise_ids.is_empty() {
        return original.to_owned();
    }
    let mut buf = String::with_capacity(original.len());
    render_children(doc.tree.root(), &noise_ids, &mut buf);
    buf
}

/// Collects concatenated `inner_html()` from all elements matching `selector`.
///
/// Returns `None` if the selector is invalid or matches no non-empty content.
/// Shared by [`select_doc`] and [`detect_main_content_doc`].
fn collect_inner_html(doc: &Html, selector: &str) -> Option<String> {
    let sel = scraper::Selector::parse(selector).ok()?;
    let mut buf = String::new();
    for el in doc.select(&sel) {
        buf.push_str(&el.inner_html());
    }
    if buf.trim().is_empty() {
        None
    } else {
        Some(buf)
    }
}

/// Extracts a single attribute value from the first element matching a CSS
/// selector. Used by [`extract_description_doc`] and [`extract_og_image_doc`].
fn extract_meta_attr(doc: &Html, selector: &str, attr: &str) -> Option<String> {
    let sel = scraper::Selector::parse(selector).ok()?;
    doc.select(&sel)
        .next()
        .and_then(|el| el.value().attr(attr))
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}

/// Resolves a single href against an optional base URL.
fn resolve_href(href: &str, base_url: Option<&url::Url>) -> String {
    let Some(base) = base_url else {
        return href.to_owned();
    };
    if url::Url::parse(href).is_ok() {
        return href.to_owned();
    }
    base.join(href)
        .map_or_else(|_| href.to_owned(), |u| u.to_string())
}

/// Collects `ego_tree::NodeId`s of all elements matching [`NOISE_SELECTORS`].
fn collect_noise_ids(doc: &Html) -> HashSet<ego_tree::NodeId> {
    let mut ids = HashSet::new();
    for &sel_str in NOISE_SELECTORS {
        if let Ok(sel) = scraper::Selector::parse(sel_str) {
            for el in doc.select(&sel) {
                ids.insert(el.id());
            }
        }
    }
    ids
}

/// Recursively serializes DOM children, skipping subtrees in `skip`.
fn render_children(
    node: ego_tree::NodeRef<'_, scraper::Node>,
    skip: &HashSet<ego_tree::NodeId>,
    buf: &mut String,
) {
    for child in node.children() {
        if skip.contains(&child.id()) {
            continue;
        }
        match child.value() {
            scraper::Node::Text(text) => buf.push_str(text),
            scraper::Node::Element(el) => {
                let tag = el.name();
                let _ = write!(buf, "<{tag}");
                for (name, val) in el.attrs() {
                    let _ = write!(buf, r#" {name}="{val}""#);
                }
                buf.push('>');
                if !is_void_element(tag) {
                    render_children(child, skip, buf);
                    let _ = write!(buf, "</{tag}>");
                }
            }
            scraper::Node::Document | scraper::Node::Fragment => {
                render_children(child, skip, buf);
            }
            _ => {}
        }
    }
}

/// HTML void elements that have no closing tag.
fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_basic() {
        let html = "<html><head><title>  Test Page  </title></head><body></body></html>";
        assert_eq!(extract_title(html).as_deref(), Some("Test Page"));
    }

    #[test]
    fn title_missing() {
        assert_eq!(extract_title("<html><body>no title</body></html>"), None);
    }

    #[test]
    fn title_empty() {
        assert_eq!(
            extract_title("<html><head><title>  </title></head></html>"),
            None
        );
    }

    #[test]
    fn links_basic() {
        let html = r##"<a href="/a">A</a><a href="https://b.com">B</a><a href="#c">C</a>"##;
        let links = extract_links(html);
        assert_eq!(links, vec!["/a", "https://b.com"]);
    }

    #[test]
    fn links_empty_href_excluded() {
        assert!(extract_links(r#"<a href="">empty</a>"#).is_empty());
    }

    #[test]
    fn select_basic() {
        let html = r"<article>content</article><aside>nav</aside>";
        assert_eq!(select(html, "article"), "content");
    }

    #[test]
    fn select_no_match_returns_original() {
        let html = "<p>hello</p>";
        assert_eq!(select(html, ".missing"), html);
    }

    #[test]
    fn select_invalid_selector_returns_original() {
        let html = "<p>hello</p>";
        assert_eq!(select(html, ":::invalid"), html);
    }

    #[test]
    fn detect_main_content_article() {
        let html = "<nav>nav</nav><article><p>main</p></article>";
        assert_eq!(detect_main_content(html).as_deref(), Some("<p>main</p>"));
    }

    #[test]
    fn detect_main_content_none() {
        assert!(detect_main_content("<div>plain</div>").is_none());
    }
}
