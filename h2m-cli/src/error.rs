//! Unified CLI error type.

/// Errors that can occur during CLI execution.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub(crate) enum CliError {
    /// HTTP scrape or conversion error from the library.
    #[error("{0}")]
    Scrape(#[from] h2m::scrape::ScrapeError),

    /// Web search error (only compiled in with the `search` feature).
    #[cfg(feature = "search")]
    #[error("{0}")]
    Search(#[from] h2m_search::SearchError),

    /// I/O error (stdin read, file read/write).
    #[error("{0}")]
    Io(#[from] std::io::Error),

    /// User-supplied input is malformed (e.g. missing URL file, bad flag combo).
    #[error("{message}")]
    BadInput {
        /// Human-readable description.
        message: String,
        /// Associated URL, if any.
        url: Option<String>,
    },

    /// The process received `SIGINT` / Ctrl-C.
    #[error("operation cancelled by user")]
    Interrupted,
}

impl CliError {
    /// Returns the URL associated with this error, if any.
    pub(crate) fn url(&self) -> Option<&str> {
        match self {
            Self::Scrape(e) => e.url(),
            #[cfg(feature = "search")]
            Self::Search(_) => None,
            Self::Io(_) | Self::Interrupted => None,
            Self::BadInput { url, .. } => url.as_deref(),
        }
    }

    /// Convenience helper: constructs a [`CliError::BadInput`].
    pub(crate) fn bad_input(message: impl Into<String>) -> Self {
        Self::BadInput {
            message: message.into(),
            url: None,
        }
    }
}
