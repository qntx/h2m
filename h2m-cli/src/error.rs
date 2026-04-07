//! Unified CLI error type.

/// Errors that can occur during CLI execution.
#[derive(Debug, thiserror::Error)]
pub(crate) enum CliError {
    /// HTTP scrape or conversion error from the library.
    #[error("{0}")]
    Scrape(#[from] h2m::scrape::ScrapeError),

    /// I/O error (stdin read, file read/write).
    #[error("{0}")]
    Io(#[from] std::io::Error),

    /// Context-rich error with an optional URL.
    #[error("{message}")]
    Other {
        /// Human-readable error description.
        message: String,
        /// URL that caused the error, if applicable.
        url: Option<String>,
    },
}

impl CliError {
    /// Returns the URL associated with this error, if any.
    pub(crate) fn url(&self) -> Option<&str> {
        match self {
            Self::Scrape(e) => e.url(),
            Self::Io(_) => None,
            Self::Other { url, .. } => url.as_deref(),
        }
    }
}
