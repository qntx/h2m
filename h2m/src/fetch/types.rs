//! Type definitions for the fetch pipeline.

use serde::Serialize;

/// How to extract content from the HTML document before conversion.
#[derive(Debug, Clone, Default)]
pub enum ContentExtraction {
    /// Use the full document.
    #[default]
    Full,
    /// Apply an explicit CSS selector.
    Selector(String),
    /// Smart readable extraction: semantic selectors → noise stripping.
    Readable,
}

/// Bundled conversion parameters passed to spawned tasks.
#[derive(Debug, Clone)]
pub struct ConvertConfig {
    /// Converter options.
    pub options: crate::options::Options,
    /// Enable GFM.
    pub gfm: bool,
    /// Extract links.
    pub extract_links: bool,
    /// Base domain for resolving relative URLs.
    pub domain: Option<String>,
    /// Content extraction strategy.
    pub content: ContentExtraction,
}

/// HTTP response metadata returned alongside the HTML body.
#[derive(Debug, Clone, Default)]
pub struct ResponseMeta {
    /// HTTP status code.
    pub status_code: Option<u16>,
    /// `Content-Type` header value.
    pub content_type: Option<String>,
}

/// Successful conversion result with metadata.
///
/// Fields are grouped: source → HTTP → document → content → metrics.
#[derive(Debug, Serialize)]
#[non_exhaustive]
#[allow(clippy::module_name_repetitions)]
pub struct FetchResult {
    /// Source URL (if input was a URL).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Resolved domain name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    /// HTTP status code of the final response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<u16>,
    /// `Content-Type` header of the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    /// Page `<title>` text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Document language from `<html lang="…">`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Page description from `<meta name="description">`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Open Graph image URL from `<meta property="og:image">`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub og_image: Option<String>,
    /// Converted Markdown content.
    pub markdown: String,
    /// Extracted links (when enabled).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<String>>,
    /// Total elapsed time in milliseconds.
    pub elapsed_ms: u64,
    /// Original HTML byte length.
    pub content_length: usize,
}

/// Error returned by fetch operations.
#[derive(Debug, Serialize, thiserror::Error)]
#[error("{error}")]
#[non_exhaustive]
#[allow(clippy::module_name_repetitions)]
pub struct FetchError {
    /// Error message.
    pub error: String,
    /// URL that caused the error, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl FetchError {
    /// Creates a new `FetchError` with an error message and optional URL.
    #[must_use]
    pub fn new(error: impl Into<String>, url: Option<String>) -> Self {
        Self {
            error: error.into(),
            url,
        }
    }
}
