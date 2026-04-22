//! High-level [`SearchClient`] dispatching to configured providers.
//!
//! The client is a compile-time `enum` over the providers enabled via Cargo
//! features, avoiding dynamic dispatch while still letting downstream users
//! pick a backend at runtime.

use std::env;

use crate::error::SearchError;
use crate::query::SearchQuery;
use crate::response::SearchResponse;

/// Environment variable selecting the default provider (`searxng`, `brave`, …).
pub const ENV_PROVIDER: &str = "H2M_SEARCH_PROVIDER";

/// Environment variable pointing at a `SearXNG` instance base URL.
#[cfg(feature = "searxng")]
pub const ENV_SEARXNG_URL: &str = "H2M_SEARXNG_URL";

/// Environment variable carrying a Brave Search API token.
#[cfg(feature = "brave")]
pub const ENV_BRAVE_API_KEY: &str = "BRAVE_API_KEY";

/// Environment variable carrying a Tavily API token.
#[cfg(feature = "tavily")]
pub const ENV_TAVILY_API_KEY: &str = "TAVILY_API_KEY";

/// Unified, statically-dispatched search client.
///
/// Variants appear only when the matching provider feature is enabled.
/// Construct via [`SearchClient::builder`] or [`SearchClient::from_env`].
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum SearchClient {
    /// `SearXNG` metasearch provider.
    #[cfg(feature = "searxng")]
    SearXNG(crate::providers::searxng::SearXNG),
    /// Brave Search API provider.
    #[cfg(feature = "brave")]
    Brave(crate::providers::brave::Brave),
    /// Tavily Search API provider.
    #[cfg(feature = "tavily")]
    Tavily(crate::providers::tavily::Tavily),
}

impl SearchClient {
    /// Returns a new [`SearchClientBuilder`].
    #[must_use]
    pub fn builder() -> SearchClientBuilder {
        SearchClientBuilder::default()
    }

    /// Builds a client by inspecting environment variables.
    ///
    /// Resolution order:
    /// 1. `H2M_SEARCH_PROVIDER` selects the provider (defaults to `searxng`).
    /// 2. Provider-specific variables supply credentials / endpoints
    ///    (`H2M_SEARXNG_URL`, `BRAVE_API_KEY`, `TAVILY_API_KEY`).
    ///
    /// # Errors
    ///
    /// Returns [`SearchError::ProviderUnavailable`] if the selected provider
    /// is not compiled in, and variant-specific errors (e.g.
    /// [`SearchError::MissingSearxngUrl`]) when configuration is incomplete.
    pub fn from_env() -> Result<Self, SearchError> {
        let name = env::var(ENV_PROVIDER).unwrap_or_else(|_| default_provider().into());
        Self::from_provider_name(&name)
    }

    /// Builds a client for the named provider, reading credentials from the
    /// environment.
    ///
    /// # Errors
    ///
    /// See [`SearchClient::from_env`].
    pub fn from_provider_name(name: &str) -> Result<Self, SearchError> {
        match name.trim().to_ascii_lowercase().as_str() {
            #[cfg(feature = "searxng")]
            "searxng" => {
                let url = env::var(ENV_SEARXNG_URL).map_err(|_| SearchError::MissingSearxngUrl)?;
                Ok(Self::SearXNG(crate::providers::searxng::SearXNG::new(url)?))
            }
            #[cfg(feature = "brave")]
            "brave" => {
                let key = env::var(ENV_BRAVE_API_KEY).map_err(|_| SearchError::MissingApiKey {
                    provider: "brave",
                    env_var: ENV_BRAVE_API_KEY,
                })?;
                Ok(Self::Brave(crate::providers::brave::Brave::new(key)?))
            }
            #[cfg(feature = "tavily")]
            "tavily" => {
                let key = env::var(ENV_TAVILY_API_KEY).map_err(|_| SearchError::MissingApiKey {
                    provider: "tavily",
                    env_var: ENV_TAVILY_API_KEY,
                })?;
                Ok(Self::Tavily(crate::providers::tavily::Tavily::new(key)?))
            }
            other => Err(SearchError::ProviderUnavailable {
                name: other.to_owned(),
            }),
        }
    }

    /// Returns the provider identifier (`"searxng"`, …).
    #[must_use]
    pub const fn id(&self) -> &'static str {
        match self {
            #[cfg(feature = "searxng")]
            Self::SearXNG(_) => "searxng",
            #[cfg(feature = "brave")]
            Self::Brave(_) => "brave",
            #[cfg(feature = "tavily")]
            Self::Tavily(_) => "tavily",
        }
    }

    /// Executes a search.
    ///
    /// # Errors
    ///
    /// Propagates whatever the underlying provider returns.
    pub async fn search(&self, query: &SearchQuery) -> Result<SearchResponse, SearchError> {
        match self {
            #[cfg(feature = "searxng")]
            Self::SearXNG(p) => p.search(query).await,
            #[cfg(feature = "brave")]
            Self::Brave(p) => p.search(query).await,
            #[cfg(feature = "tavily")]
            Self::Tavily(p) => p.search(query).await,
        }
    }
}

/// Builder for [`SearchClient`].
///
/// Provides explicit, per-provider configuration that takes precedence over
/// the environment.
#[derive(Debug, Default)]
pub struct SearchClientBuilder {
    provider: Option<String>,
    searxng_url: Option<String>,
    brave_api_key: Option<String>,
    tavily_api_key: Option<String>,
}

impl SearchClientBuilder {
    /// Selects the provider by name.
    #[must_use]
    pub fn provider(mut self, name: impl Into<String>) -> Self {
        self.provider = Some(name.into());
        self
    }

    /// Overrides the `SearXNG` instance URL.
    #[must_use]
    pub fn searxng_url(mut self, url: impl Into<String>) -> Self {
        self.searxng_url = Some(url.into());
        self
    }

    /// Overrides the Brave API key.
    #[must_use]
    pub fn brave_api_key(mut self, key: impl Into<String>) -> Self {
        self.brave_api_key = Some(key.into());
        self
    }

    /// Overrides the Tavily API key.
    #[must_use]
    pub fn tavily_api_key(mut self, key: impl Into<String>) -> Self {
        self.tavily_api_key = Some(key.into());
        self
    }

    /// Builds the [`SearchClient`].
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`SearchClient::from_env`], but with builder
    /// overrides taking precedence.
    pub fn build(self) -> Result<SearchClient, SearchError> {
        let name = self
            .provider
            .clone()
            .or_else(|| env::var(ENV_PROVIDER).ok())
            .unwrap_or_else(|| default_provider().into());

        match name.trim().to_ascii_lowercase().as_str() {
            #[cfg(feature = "searxng")]
            "searxng" => {
                let url = self
                    .searxng_url
                    .or_else(|| env::var(ENV_SEARXNG_URL).ok())
                    .ok_or(SearchError::MissingSearxngUrl)?;
                Ok(SearchClient::SearXNG(
                    crate::providers::searxng::SearXNG::new(url)?,
                ))
            }
            #[cfg(feature = "brave")]
            "brave" => {
                let key = self
                    .brave_api_key
                    .or_else(|| env::var(ENV_BRAVE_API_KEY).ok())
                    .ok_or(SearchError::MissingApiKey {
                        provider: "brave",
                        env_var: ENV_BRAVE_API_KEY,
                    })?;
                Ok(SearchClient::Brave(crate::providers::brave::Brave::new(
                    key,
                )?))
            }
            #[cfg(feature = "tavily")]
            "tavily" => {
                let key = self
                    .tavily_api_key
                    .or_else(|| env::var(ENV_TAVILY_API_KEY).ok())
                    .ok_or(SearchError::MissingApiKey {
                        provider: "tavily",
                        env_var: ENV_TAVILY_API_KEY,
                    })?;
                Ok(SearchClient::Tavily(crate::providers::tavily::Tavily::new(
                    key,
                )?))
            }
            other => Err(SearchError::ProviderUnavailable {
                name: other.to_owned(),
            }),
        }
    }
}

const fn default_provider() -> &'static str {
    #[cfg(feature = "searxng")]
    {
        "searxng"
    }
    #[cfg(not(feature = "searxng"))]
    {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "searxng")]
    fn builder_accepts_explicit_url() {
        let client = SearchClient::builder()
            .provider("searxng")
            .searxng_url("https://searx.example.org")
            .build()
            .unwrap();
        assert_eq!(client.id(), "searxng");
    }

    #[test]
    #[cfg(feature = "searxng")]
    fn builder_rejects_invalid_searxng_url() {
        let err = SearchClient::builder()
            .provider("searxng")
            .searxng_url("not a url")
            .build()
            .unwrap_err();
        assert!(matches!(err, SearchError::Config { .. }));
    }

    #[test]
    fn unknown_provider_errors() {
        let err = SearchClient::from_provider_name("yahoo").unwrap_err();
        assert!(matches!(err, SearchError::ProviderUnavailable { .. }));
    }
}
