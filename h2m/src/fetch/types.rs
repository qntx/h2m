//! Type definitions for the fetch pipeline.

use serde::Serialize;

use crate::converter::Converter;

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
    /// Pre-built converter, cached to avoid per-call rebuilds.
    pub(super) converter: Converter,
    /// Extract links.
    pub(super) extract_links: bool,
    /// Base domain for resolving relative URLs.
    pub(super) domain: Option<String>,
    /// Content extraction strategy.
    pub(super) content: ContentExtraction,
}

/// HTTP response metadata returned alongside the HTML body.
#[derive(Debug, Clone, Default)]
pub struct ResponseMeta {
    /// HTTP status code.
    pub(super) status_code: Option<u16>,
    /// `Content-Type` header value.
    pub(super) content_type: Option<String>,
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
#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
#[allow(clippy::module_name_repetitions)]
pub enum FetchError {
    /// HTTP request or response body decoding failed.
    #[error("HTTP error for {url}: {message}")]
    Http {
        /// Human-readable error description.
        message: String,
        /// URL that caused the error.
        url: String,
    },

    /// Too many `<meta http-equiv="refresh">` hops.
    #[error("too many meta-refresh redirects for {url}")]
    TooManyRedirects {
        /// URL that started the redirect chain.
        url: String,
    },

    /// Failed to construct the underlying HTTP client.
    #[error("failed to build HTTP client: {message}")]
    ClientBuild {
        /// Underlying error description.
        message: String,
    },

    /// A spawned task panicked before producing a result.
    #[error("task panicked: {message}")]
    TaskPanicked {
        /// Panic payload description.
        message: String,
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

impl FetchError {
    /// Creates a new [`FetchError::Other`] with an error message and optional URL.
    #[must_use]
    pub fn new(error: impl Into<String>, url: Option<String>) -> Self {
        Self::Other {
            message: error.into(),
            url,
        }
    }

    /// Returns the URL associated with this error, if any.
    #[must_use]
    pub fn url(&self) -> Option<&str> {
        match self {
            Self::Http { url, .. }
            | Self::TooManyRedirects { url }
            | Self::Other { url: Some(url), .. } => Some(url),
            _ => None,
        }
    }
}

impl Serialize for FetchError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let url = self.url();
        let field_count = if url.is_some() { 2 } else { 1 };
        let mut state = serializer.serialize_struct("FetchError", field_count)?;
        state.serialize_field("error", &self.to_string())?;
        if let Some(u) = url {
            state.serialize_field("url", u)?;
        }
        state.end()
    }
}
