//! Unified CLI error type.

/// Errors that can occur during CLI execution.
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    /// HTTP fetch or conversion error from the library.
    #[error("{0}")]
    Fetch(#[from] h2m::fetch::FetchError),

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
    pub fn url(&self) -> Option<&str> {
        match self {
            Self::Fetch(e) => e.url(),
            Self::Io(_) => None,
            Self::Other { url, .. } => url.as_deref(),
        }
    }
}
