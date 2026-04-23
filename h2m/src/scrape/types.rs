//! Type definitions for the scrape pipeline.

use serde::Serialize;

use crate::converter::Converter;

/// How to extract content from the HTML document before conversion.
#[derive(Debug, Clone, Default)]
pub(crate) enum ContentExtraction {
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
pub(super) struct ConvertConfig {
    /// Pre-built converter, cached to avoid per-call rebuilds.
    pub converter: Converter,
    /// Whether to extract links from pages.
    pub extract_links: bool,
    /// Content extraction strategy.
    pub content: ContentExtraction,
}

/// HTTP response data returned by the client layer.
#[derive(Debug, Clone)]
pub(super) struct HttpResponse {
    /// HTML response body.
    pub body: String,
    /// HTTP status code.
    pub status_code: u16,
    /// `Content-Type` header value.
    pub content_type: Option<String>,
    /// Final URL after HTTP 3xx redirects.
    pub final_url: String,
}

/// Successful scrape result with nested metadata.
///
/// JSON output uses **camelCase** field names to align with web-ecosystem
/// conventions (consistent with Firecrawl, Jina Reader, etc.).
///
/// ```text
/// {
///   "markdown": "…",
///   "metadata": { "title": "…", "sourceUrl": "…", "url": "…", … },
///   "links": ["…"]
/// }
/// ```
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ScrapeResult {
    /// Converted Markdown content.
    pub markdown: String,
    /// Page and HTTP metadata.
    pub metadata: Metadata,
    /// Extracted links (present only when link extraction is enabled).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<String>>,
}

/// Page and HTTP metadata extracted during scraping.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Metadata {
    /// Page `<title>` text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// `<meta name="description">` content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// `<html lang="…">` attribute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// `<meta property="og:image">` URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub og_image: Option<String>,
    /// Original requested URL.
    pub source_url: String,
    /// Final URL after all redirects (HTTP 3xx + meta-refresh).
    pub url: String,
    /// HTTP status code of the final response.
    pub status_code: u16,
    /// `Content-Type` header value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    /// Total elapsed time in milliseconds.
    pub elapsed_ms: u64,
}

/// Error returned by scrape operations.
#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum ScrapeError {
    /// HTTP request or response decoding failed.
    #[error("HTTP error for {url}: {message}")]
    Http {
        /// URL that caused the error.
        url: String,
        /// Human-readable error description.
        message: String,
    },

    /// Too many `<meta http-equiv="refresh">` hops.
    #[error("too many redirects for {url}")]
    TooManyRedirects {
        /// URL that started the redirect chain.
        url: String,
    },

    /// Catch-all for errors that don't fit other variants.
    #[error("{message}")]
    Other {
        /// Human-readable error description.
        message: String,
        /// URL that caused the error, if applicable.
        url: Option<String>,
    },
}

impl ScrapeError {
    /// Creates a [`ScrapeError::Other`] with a message and optional URL.
    ///
    /// Renamed from `new` in 0.8 because it only constructs a single
    /// variant — the old name suggested it was a primary constructor.
    #[must_use]
    pub fn other(message: impl Into<String>, url: Option<String>) -> Self {
        Self::Other {
            message: message.into(),
            url,
        }
    }

    /// Returns the URL associated with this error, if any.
    #[must_use]
    pub fn url(&self) -> Option<&str> {
        match self {
            Self::Http { url, .. } | Self::TooManyRedirects { url } => Some(url),
            Self::Other { url, .. } => url.as_deref(),
        }
    }
}

impl Serialize for ScrapeError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("error", &self.to_string())?;
        if let Some(u) = self.url() {
            map.serialize_entry("url", u)?;
        }
        map.end()
    }
}
