//! Error types for search operations.

use serde::Serialize;

/// Errors returned by [`SearchClient`](crate::SearchClient) and its providers.
#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum SearchError {
    /// HTTP request or transport failure.
    #[error("HTTP error from {provider}: {message}")]
    Http {
        /// Provider identifier (`"searxng"`, `"brave"`, `"tavily"`, …).
        provider: &'static str,
        /// Human-readable description.
        message: String,
    },

    /// Required API key is missing from the environment.
    #[error("missing API key for {provider}: set {env_var}")]
    MissingApiKey {
        /// Provider identifier.
        provider: &'static str,
        /// Environment variable the user must set.
        env_var: &'static str,
    },

    /// `SearXNG` base URL was not configured.
    #[error("missing SearXNG base URL: set H2M_SEARXNG_URL or pass --searxng-url")]
    MissingSearxngUrl,

    /// Provider returned a malformed or unexpected payload.
    #[error("invalid response from {provider}: {message}")]
    InvalidResponse {
        /// Provider identifier.
        provider: &'static str,
        /// Human-readable description.
        message: String,
    },

    /// Requested provider is not compiled into this build.
    #[error("provider '{name}' is not available (missing feature flag)")]
    ProviderUnavailable {
        /// Requested provider name.
        name: String,
    },

    /// Invalid configuration or query parameters.
    #[error("configuration error: {message}")]
    Config {
        /// Human-readable description.
        message: String,
    },
}

impl SearchError {
    /// Returns the provider identifier associated with this error, if any.
    #[must_use]
    pub const fn provider(&self) -> Option<&'static str> {
        match self {
            Self::Http { provider, .. }
            | Self::MissingApiKey { provider, .. }
            | Self::InvalidResponse { provider, .. } => Some(provider),
            Self::MissingSearxngUrl | Self::ProviderUnavailable { .. } | Self::Config { .. } => {
                None
            }
        }
    }
}

impl Serialize for SearchError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("error", &self.to_string())?;
        if let Some(p) = self.provider() {
            map.serialize_entry("provider", p)?;
        }
        map.end()
    }
}
