//! High-level [`SearchClient`] dispatching to configured providers.
//!
//! The client is a compile-time `enum` over the providers enabled via Cargo
//! features, avoiding dynamic dispatch while still letting downstream users
//! pick a backend at runtime.

use std::env;

use crate::error::SearchError;
use crate::query::SearchQuery;
use crate::response::SearchResponse;
#[cfg(any(feature = "brave", feature = "tavily"))]
use crate::secret::SecretString;

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
    /// `DuckDuckGo` zero-config HTML-scraping provider (default).
    #[cfg(feature = "duckduckgo")]
    DuckDuckGo(crate::providers::duckduckgo::DuckDuckGo),
    /// Wikipedia `MediaWiki` Search API provider.
    #[cfg(feature = "wikipedia")]
    Wikipedia(crate::providers::wikipedia::Wikipedia),
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
    /// 1. `H2M_SEARCH_PROVIDER` selects the provider (defaults to `duckduckgo`).
    /// 2. Provider-specific variables supply credentials / endpoints
    ///    (`H2M_SEARXNG_URL`, `BRAVE_API_KEY`, `TAVILY_API_KEY`).
    ///    `duckduckgo` and `wikipedia` require nothing — zero-config.
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
        let builder = Self::builder().provider(name);
        #[cfg(feature = "searxng")]
        let builder = if name.trim().eq_ignore_ascii_case("searxng")
            && let Ok(url) = env::var(ENV_SEARXNG_URL)
        {
            builder.searxng_url(url)
        } else {
            builder
        };
        builder.build()
    }

    /// Returns the provider identifier (`"duckduckgo"`, `"wikipedia"`, …).
    #[must_use]
    pub const fn id(&self) -> &'static str {
        match self {
            #[cfg(feature = "duckduckgo")]
            Self::DuckDuckGo(_) => "duckduckgo",
            #[cfg(feature = "wikipedia")]
            Self::Wikipedia(_) => "wikipedia",
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
            #[cfg(feature = "duckduckgo")]
            Self::DuckDuckGo(p) => p.search(query).await,
            #[cfg(feature = "wikipedia")]
            Self::Wikipedia(p) => p.search(query).await,
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
    #[cfg(feature = "searxng")]
    searxng_url: Option<String>,
    #[cfg(feature = "brave")]
    brave_api_key: Option<SecretString>,
    #[cfg(feature = "tavily")]
    tavily_api_key: Option<SecretString>,
    #[cfg(feature = "tavily")]
    tavily_include_answer: bool,
    #[cfg(feature = "wikipedia")]
    wikipedia_language: Option<String>,
    http: Option<crate::http::HttpConfig>,
    retry: Option<crate::retry::RetryPolicy>,
}

impl SearchClientBuilder {
    /// Selects the provider by name.
    #[must_use]
    pub fn provider(mut self, name: impl Into<String>) -> Self {
        self.provider = Some(name.into());
        self
    }

    /// Overrides the `SearXNG` instance URL.
    #[cfg(feature = "searxng")]
    #[must_use]
    pub fn searxng_url(mut self, url: impl Into<String>) -> Self {
        self.searxng_url = Some(url.into());
        self
    }

    /// Overrides the Brave API key.
    #[cfg(feature = "brave")]
    #[must_use]
    pub fn brave_api_key(mut self, key: impl Into<SecretString>) -> Self {
        self.brave_api_key = Some(key.into());
        self
    }

    /// Overrides the Tavily API key.
    #[cfg(feature = "tavily")]
    #[must_use]
    pub fn tavily_api_key(mut self, key: impl Into<SecretString>) -> Self {
        self.tavily_api_key = Some(key.into());
        self
    }

    /// Enables Tavily's LLM-generated answer field.
    ///
    /// Ignored by other providers. Defaults to `false` because the answer
    /// costs extra Tavily credits per request.
    #[cfg(feature = "tavily")]
    #[must_use]
    pub const fn tavily_include_answer(mut self, include: bool) -> Self {
        self.tavily_include_answer = include;
        self
    }

    /// Sets the Wikipedia language edition (e.g. `"en"`, `"zh"`).
    ///
    /// Per-query [`SearchQuery::language`](crate::SearchQuery::language)
    /// still wins when both are set. Defaults to `en`.
    #[cfg(feature = "wikipedia")]
    #[must_use]
    pub fn wikipedia_language(mut self, lang: impl Into<String>) -> Self {
        self.wikipedia_language = Some(lang.into());
        self
    }

    /// Supplies a shared [`HttpConfig`](crate::HttpConfig) used by the
    /// selected provider. When omitted, each provider constructs its own.
    #[must_use]
    pub fn http(mut self, http: crate::http::HttpConfig) -> Self {
        self.http = Some(http);
        self
    }

    /// Overrides the retry policy applied by the chosen provider.
    #[must_use]
    pub const fn retry(mut self, policy: crate::retry::RetryPolicy) -> Self {
        self.retry = Some(policy);
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
            #[cfg(feature = "duckduckgo")]
            "duckduckgo" | "ddg" => {
                let mut b = crate::providers::duckduckgo::DuckDuckGo::builder();
                if let Some(http) = self.http {
                    b = b.http(http);
                }
                if let Some(retry) = self.retry {
                    b = b.retry(retry);
                }
                Ok(SearchClient::DuckDuckGo(b.build()?))
            }
            #[cfg(feature = "wikipedia")]
            "wikipedia" | "wiki" => {
                let mut b = crate::providers::wikipedia::Wikipedia::builder();
                if let Some(lang) = self.wikipedia_language {
                    b = b.language(lang);
                }
                if let Some(http) = self.http {
                    b = b.http(http);
                }
                if let Some(retry) = self.retry {
                    b = b.retry(retry);
                }
                Ok(SearchClient::Wikipedia(b.build()?))
            }
            #[cfg(feature = "searxng")]
            "searxng" => {
                let url = self
                    .searxng_url
                    .or_else(|| env::var(ENV_SEARXNG_URL).ok())
                    .ok_or(SearchError::MissingSearxngUrl)?;
                let mut b = crate::providers::searxng::SearXNG::builder(url);
                if let Some(http) = self.http {
                    b = b.http(http);
                }
                if let Some(retry) = self.retry {
                    b = b.retry(retry);
                }
                Ok(SearchClient::SearXNG(b.build()?))
            }
            #[cfg(feature = "brave")]
            "brave" => {
                let key = self
                    .brave_api_key
                    .or_else(|| env::var(ENV_BRAVE_API_KEY).ok().map(SecretString::new))
                    .ok_or(SearchError::MissingApiKey {
                        provider: "brave",
                        env_var: ENV_BRAVE_API_KEY,
                    })?;
                let mut b = crate::providers::brave::Brave::builder(key);
                if let Some(http) = self.http {
                    b = b.http(http);
                }
                if let Some(retry) = self.retry {
                    b = b.retry(retry);
                }
                Ok(SearchClient::Brave(b.build()?))
            }
            #[cfg(feature = "tavily")]
            "tavily" => {
                let key = self
                    .tavily_api_key
                    .or_else(|| env::var(ENV_TAVILY_API_KEY).ok().map(SecretString::new))
                    .ok_or(SearchError::MissingApiKey {
                        provider: "tavily",
                        env_var: ENV_TAVILY_API_KEY,
                    })?;
                let mut b = crate::providers::tavily::Tavily::builder(key)
                    .include_answer(self.tavily_include_answer);
                if let Some(http) = self.http {
                    b = b.http(http);
                }
                if let Some(retry) = self.retry {
                    b = b.retry(retry);
                }
                Ok(SearchClient::Tavily(b.build()?))
            }
            other => Err(SearchError::ProviderUnavailable {
                name: other.to_owned(),
            }),
        }
    }
}

const fn default_provider() -> &'static str {
    #[cfg(feature = "duckduckgo")]
    {
        "duckduckgo"
    }
    #[cfg(all(not(feature = "duckduckgo"), feature = "wikipedia"))]
    {
        "wikipedia"
    }
    #[cfg(all(
        not(feature = "duckduckgo"),
        not(feature = "wikipedia"),
        feature = "searxng"
    ))]
    {
        "searxng"
    }
    #[cfg(all(
        not(feature = "duckduckgo"),
        not(feature = "wikipedia"),
        not(feature = "searxng")
    ))]
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

    #[test]
    #[cfg(feature = "duckduckgo")]
    fn duckduckgo_is_the_default_provider() {
        // SAFETY: tests run sequentially; resetting env vars is fine.
        // We only assert the default string, not that env is pristine.
        assert_eq!(default_provider(), "duckduckgo");
    }

    #[test]
    #[cfg(feature = "duckduckgo")]
    fn builder_creates_duckduckgo_with_no_config() {
        let client = SearchClient::builder()
            .provider("duckduckgo")
            .build()
            .unwrap();
        assert_eq!(client.id(), "duckduckgo");
    }

    #[test]
    #[cfg(feature = "wikipedia")]
    fn builder_creates_wikipedia_with_custom_language() {
        let client = SearchClient::builder()
            .provider("wikipedia")
            .wikipedia_language("zh")
            .build()
            .unwrap();
        assert_eq!(client.id(), "wikipedia");
    }

    #[test]
    #[cfg(feature = "duckduckgo")]
    fn ddg_alias_resolves_to_duckduckgo() {
        let client = SearchClient::builder().provider("ddg").build().unwrap();
        assert_eq!(client.id(), "duckduckgo");
    }
}
