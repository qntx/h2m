//! CSS selector application and content selection.

use scraper::Html;

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

/// Applies a CSS selector to a pre-parsed document, returning matched inner
/// HTML or the `original` string if nothing matches.
pub(crate) fn select_doc(doc: &Html, original: &str, selector: &str) -> String {
    collect_inner_html(doc, selector).unwrap_or_else(|| original.to_owned())
}

/// Collects concatenated `inner_html()` from all elements matching `selector`.
///
/// Returns `None` if the selector is invalid or matches no non-empty content.
pub(super) fn collect_inner_html(doc: &Html, selector: &str) -> Option<String> {
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
