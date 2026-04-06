//! Metadata extraction: title, links, language, description, og:image.

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

/// Extracts `<title>` text from a pre-parsed document.
fn extract_title_doc(doc: &Html) -> Option<String> {
    let sel = scraper::Selector::parse("title").ok()?;
    doc.select(&sel)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_owned())
        .filter(|t| !t.is_empty())
}

/// Extracts all `<a href>` link URLs from a pre-parsed document.
///
/// When `base` is provided, relative URLs are resolved to absolute.
pub fn extract_links_doc(doc: &Html, base: Option<&url::Url>) -> Vec<String> {
    let Ok(sel) = scraper::Selector::parse("a[href]") else {
        return Vec::new();
    };
    doc.select(&sel)
        .filter_map(|el| el.value().attr("href"))
        .filter(|href| !href.is_empty() && !href.starts_with('#'))
        .map(|href| resolve_href(href, base))
        .collect()
}

/// Extracts the `lang` attribute from the `<html>` element.
fn extract_language_doc(doc: &Html) -> Option<String> {
    let sel = scraper::Selector::parse("html").ok()?;
    doc.select(&sel)
        .next()
        .and_then(|el| el.value().attr("lang"))
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}

/// Extracts the `<meta name="description">` content.
fn extract_description_doc(doc: &Html) -> Option<String> {
    extract_meta_attr(doc, r#"meta[name="description"]"#, "content")
}

/// Extracts the Open Graph image URL (`<meta property="og:image">`).
fn extract_og_image_doc(doc: &Html) -> Option<String> {
    extract_meta_attr(doc, r#"meta[property="og:image"]"#, "content")
}

/// Extracts a single attribute value from the first element matching a CSS
/// selector.
fn extract_meta_attr(doc: &Html, selector: &str, attr: &str) -> Option<String> {
    let sel = scraper::Selector::parse(selector).ok()?;
    doc.select(&sel)
        .next()
        .and_then(|el| el.value().attr(attr))
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}

/// Aggregated page metadata extracted from a parsed HTML document.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct PageMeta {
    /// Page `<title>` text.
    pub title: Option<String>,
    /// `<meta name="description">` content.
    pub description: Option<String>,
    /// `<html lang="…">` attribute.
    pub language: Option<String>,
    /// `<meta property="og:image">` URL.
    pub og_image: Option<String>,
}

impl PageMeta {
    /// Extracts all available metadata from a pre-parsed document.
    pub(crate) fn from_doc(doc: &Html) -> Self {
        Self {
            title: extract_title_doc(doc),
            description: extract_description_doc(doc),
            language: extract_language_doc(doc),
            og_image: extract_og_image_doc(doc),
        }
    }
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
