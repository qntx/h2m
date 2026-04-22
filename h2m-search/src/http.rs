//! Shared HTTP primitives for every provider.
//!
//! Every provider gets the same [`reqwest::Client`] by default, ensuring a
//! single connection pool across the whole crate. Providers may still bring
//! their own client via [`HttpConfig::with_client`] when callers need
//! custom middleware, proxies, or shared pools with the host application.

use std::sync::Arc;
use std::time::Duration;

use crate::error::SearchError;

/// Default per-request timeout applied to every provider.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// User agent sent with every request.
pub const USER_AGENT: &str = concat!("h2m-search/", env!("CARGO_PKG_VERSION"));

/// Shared HTTP configuration used to build provider-specific clients.
///
/// The [`Arc<reqwest::Client>`] is reference-counted so multiple providers
/// in the same process share a single TLS / connection pool.
#[derive(Debug, Clone)]
pub struct HttpConfig {
    client: Arc<reqwest::Client>,
}

impl HttpConfig {
    /// Builds the default client (30s timeout, h2m user-agent, system proxy).
    ///
    /// # Errors
    ///
    /// Returns [`SearchError::Config`] if the underlying `reqwest` builder
    /// fails (e.g. missing TLS backend).
    pub fn new() -> Result<Self, SearchError> {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(DEFAULT_TIMEOUT)
            .build()
            .map_err(|e| SearchError::Config {
                message: format!("failed to build HTTP client: {e}"),
            })?;
        Ok(Self {
            client: Arc::new(client),
        })
    }

    /// Wraps an existing client; the caller owns connection-pool lifecycle.
    #[must_use]
    pub fn with_client(client: reqwest::Client) -> Self {
        Self {
            client: Arc::new(client),
        }
    }

    /// Shared client handle.
    #[must_use]
    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }
}

impl Default for HttpConfig {
    /// Panics on build failure. Prefer [`HttpConfig::new`] in library code.
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            client: Arc::new(reqwest::Client::new()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_client_preserves_identity_across_clones() {
        let cfg = HttpConfig::new().unwrap();
        let cloned = cfg.clone();
        assert!(
            Arc::ptr_eq(&cfg.client, &cloned.client),
            "cloning HttpConfig must share the inner Arc"
        );
    }
}
