//! Error types for h2m.

/// Errors that can occur during HTML-to-Markdown conversion.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// An I/O error occurred while reading input.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// A specialized [`Result`] type for h2m operations.
pub type Result<T> = std::result::Result<T, Error>;
