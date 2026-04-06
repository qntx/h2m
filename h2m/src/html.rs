//! HTML inspection utilities for extracting metadata from parsed documents.
//!
//! These functions operate on raw HTML strings and are always available
//! (no feature flags required).

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
    let doc = Html::parse_document(html);
    let sel = scraper::Selector::parse("title").ok()?;
    doc.select(&sel)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_owned())
        .filter(|t| !t.is_empty())
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
    let doc = Html::parse_document(html);
    let Ok(sel) = scraper::Selector::parse("a[href]") else {
        return Vec::new();
    };
    doc.select(&sel)
        .filter_map(|el| el.value().attr("href"))
        .filter(|href| !href.is_empty() && !href.starts_with('#'))
        .map(str::to_owned)
        .collect()
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
    let document = Html::parse_document(html);
    let Ok(parsed) = scraper::Selector::parse(selector) else {
        return html.to_owned();
    };

    let mut extracted = String::new();
    for element in document.select(&parsed) {
        extracted.push_str(&element.inner_html());
    }

    if extracted.is_empty() {
        return html.to_owned();
    }

    extracted
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
}
