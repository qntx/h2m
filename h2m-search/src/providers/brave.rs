//! Brave Search API provider.
//!
//! Uses the official Brave Web Search API backed by Brave's independent
//! index. Requires a subscription token supplied via the `BRAVE_API_KEY`
//! environment variable.
//!
//! Enable the `brave` feature to use this provider.

use std::time::{Duration, Instant};

use serde::Deserialize;

use crate::error::SearchError;
use crate::query::{SafeSearch, SearchQuery, SearchSource, TimeRange};
use crate::response::{SearchHit, SearchResponse};

const PROVIDER_ID: &str = "brave";
/// Canonical Brave Search API root URL.
pub const DEFAULT_BASE_URL: &str = "https://api.search.brave.com";
const SEARCH_PATH: &str = "res/v1/web/search";

/// Brave Search API provider.
#[derive(Debug, Clone)]
pub struct Brave {
    client: reqwest::Client,
    base_url: url::Url,
    api_key: String,
}

impl Brave {
    /// Creates a provider pointing at the canonical Brave API endpoint.
    ///
    /// # Errors
    ///
    /// Returns [`SearchError::Config`] if the HTTP client cannot be built.
    pub fn new(api_key: impl Into<String>) -> Result<Self, SearchError> {
        Self::with_base_url(api_key, DEFAULT_BASE_URL)
    }

    /// Creates a provider pointing at a custom `base_url` (useful for tests).
    ///
    /// # Errors
    ///
    /// Returns [`SearchError::Config`] if `base_url` is invalid or the HTTP
    /// client cannot be built.
    pub fn with_base_url(
        api_key: impl Into<String>,
        base_url: impl AsRef<str>,
    ) -> Result<Self, SearchError> {
        let base = url::Url::parse(base_url.as_ref()).map_err(|e| SearchError::Config {
            message: format!("invalid Brave base URL: {e}"),
        })?;
        let client = reqwest::Client::builder()
            .user_agent(concat!("h2m-search/", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| SearchError::Config {
                message: format!("failed to build HTTP client: {e}"),
            })?;
        Ok(Self {
            client,
            base_url: base,
            api_key: api_key.into(),
        })
    }

    /// Executes a search against the Brave API.
    ///
    /// # Errors
    ///
    /// Returns [`SearchError::Http`] on transport errors and
    /// [`SearchError::InvalidResponse`] when the JSON payload does not match
    /// the expected shape.
    pub async fn search(&self, query: &SearchQuery) -> Result<SearchResponse, SearchError> {
        if !query.is_valid() {
            return Err(SearchError::Config {
                message: "search query is empty".into(),
            });
        }

        let start = Instant::now();
        let mut endpoint = self
            .base_url
            .join(SEARCH_PATH)
            .map_err(|e| SearchError::Config {
                message: format!("cannot build Brave endpoint: {e}"),
            })?;

        {
            let mut pairs = endpoint.query_pairs_mut();
            pairs.append_pair("q", &query.query);
            pairs.append_pair("count", &query.limit.min(20).to_string());
            pairs.append_pair("safesearch", safesearch_label(query.safesearch));
            if let Some(country) = &query.country {
                pairs.append_pair("country", country);
            }
            if let Some(lang) = &query.language {
                pairs.append_pair("search_lang", lang);
            }
            if let Some(tr) = query.time_range {
                pairs.append_pair("freshness", freshness_label(tr));
            }
        }

        let response = self
            .client
            .get(endpoint)
            .header("X-Subscription-Token", &self.api_key)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| SearchError::Http {
                provider: PROVIDER_ID,
                message: e.to_string(),
            })?;

        let status = response.status();
        if !status.is_success() {
            return Err(SearchError::Http {
                provider: PROVIDER_ID,
                message: format!("HTTP {status}"),
            });
        }

        let body: BraveResponse =
            response
                .json()
                .await
                .map_err(|e| SearchError::InvalidResponse {
                    provider: PROVIDER_ID,
                    message: e.to_string(),
                })?;

        let mut result = SearchResponse::new(&query.query, PROVIDER_ID);
        if query.sources.contains(&SearchSource::Web) {
            push_section(&mut result, SearchSource::Web, body.web, query.limit);
        }
        if query.sources.contains(&SearchSource::News) {
            push_section(&mut result, SearchSource::News, body.news, query.limit);
        }
        result.elapsed_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        Ok(result)
    }
}

fn push_section(
    response: &mut SearchResponse,
    source: SearchSource,
    section: Option<BraveSection>,
    limit: usize,
) {
    let Some(section) = section else { return };
    for raw in section.results.into_iter().take(limit) {
        response.push(source, raw.into_hit());
    }
}

const fn safesearch_label(level: SafeSearch) -> &'static str {
    match level {
        SafeSearch::Off => "off",
        SafeSearch::Moderate => "moderate",
        SafeSearch::Strict => "strict",
    }
}

const fn freshness_label(range: TimeRange) -> &'static str {
    match range {
        TimeRange::Day => "pd",
        TimeRange::Week => "pw",
        TimeRange::Month => "pm",
        TimeRange::Year => "py",
    }
}

#[derive(Debug, Deserialize)]
struct BraveResponse {
    #[serde(default)]
    web: Option<BraveSection>,
    #[serde(default)]
    news: Option<BraveSection>,
}

#[derive(Debug, Deserialize)]
struct BraveSection {
    #[serde(default)]
    results: Vec<BraveResult>,
}

#[derive(Debug, Deserialize)]
struct BraveResult {
    #[serde(default)]
    title: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    age: Option<String>,
}

impl BraveResult {
    fn into_hit(self) -> SearchHit {
        SearchHit {
            title: self.title,
            url: self.url,
            description: self.description.filter(|s| !s.is_empty()),
            published_at: self.age,
            engine: None,
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    reason = "test assertions should panic on wrong shape"
)]
mod tests {
    use pretty_assertions::assert_eq;
    use wiremock::matchers::{header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    #[tokio::test]
    async fn maps_web_and_news_results() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/res/v1/web/search"))
            .and(query_param("q", "rust"))
            .and(header("X-Subscription-Token", "KEY"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "web": {
                    "results": [
                        { "title": "Rust", "url": "https://r.io", "description": "lang", "age": "2025-09-01" }
                    ]
                },
                "news": {
                    "results": [
                        { "title": "News", "url": "https://n.io", "description": "hi" }
                    ]
                }
            })))
            .mount(&server)
            .await;

        let provider = Brave::with_base_url("KEY", server.uri()).unwrap();
        let query =
            SearchQuery::new("rust").with_sources(vec![SearchSource::Web, SearchSource::News]);
        let response = provider.search(&query).await.unwrap();

        assert_eq!(response.provider, "brave");
        assert_eq!(response.web.len(), 1);
        assert_eq!(response.web[0].url, "https://r.io");
        assert_eq!(response.web[0].published_at.as_deref(), Some("2025-09-01"));
        assert_eq!(response.news.len(), 1);
    }

    #[tokio::test]
    async fn respects_source_filter() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/res/v1/web/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "web": { "results": [{ "title": "W", "url": "https://w" }] },
                "news": { "results": [{ "title": "N", "url": "https://n" }] }
            })))
            .mount(&server)
            .await;

        let provider = Brave::with_base_url("KEY", server.uri()).unwrap();
        let query = SearchQuery::new("q").with_sources(vec![SearchSource::Web]);
        let response = provider.search(&query).await.unwrap();

        assert_eq!(response.web.len(), 1);
        assert_eq!(response.news.len(), 0);
    }

    #[tokio::test]
    async fn http_error_is_reported() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/res/v1/web/search"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;

        let provider = Brave::with_base_url("KEY", server.uri()).unwrap();
        let err = provider.search(&SearchQuery::new("q")).await.unwrap_err();
        assert!(matches!(
            err,
            SearchError::Http {
                provider: "brave",
                ..
            }
        ));
    }

    #[test]
    fn empty_query_is_rejected() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let provider = Brave::new("KEY").unwrap();
        let err = runtime
            .block_on(provider.search(&SearchQuery::new("")))
            .unwrap_err();
        assert!(matches!(err, SearchError::Config { .. }));
    }
}
