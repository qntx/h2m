//! Tavily Search API provider.
//!
//! AI-oriented search with built-in LLM-friendly snippets. Requires an API
//! token supplied via the `TAVILY_API_KEY` environment variable.
//!
//! Enable the `tavily` feature to use this provider.

use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::error::SearchError;
use crate::query::{SearchQuery, SearchSource, TimeRange};
use crate::response::{SearchHit, SearchResponse};

const PROVIDER_ID: &str = "tavily";
/// Canonical Tavily API root URL.
pub const DEFAULT_BASE_URL: &str = "https://api.tavily.com";
const SEARCH_PATH: &str = "search";

/// Tavily Search API provider.
#[derive(Debug, Clone)]
pub struct Tavily {
    client: reqwest::Client,
    base_url: url::Url,
    api_key: String,
}

impl Tavily {
    /// Creates a provider pointing at the canonical Tavily endpoint.
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
            message: format!("invalid Tavily base URL: {e}"),
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

    /// Executes a search against the Tavily API.
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
        let endpoint = self
            .base_url
            .join(SEARCH_PATH)
            .map_err(|e| SearchError::Config {
                message: format!("cannot build Tavily endpoint: {e}"),
            })?;

        let payload = TavilyRequest {
            api_key: &self.api_key,
            query: &query.query,
            max_results: query.limit,
            search_depth: if query.limit > 10 {
                "advanced"
            } else {
                "basic"
            },
            include_domains: None,
            time_range: query.time_range.map(time_range_label),
            topic: topic_for(&query.sources),
        };

        let response = self
            .client
            .post(endpoint)
            .header("Content-Type", "application/json")
            .json(&payload)
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

        let parsed: TavilyResponse =
            response
                .json()
                .await
                .map_err(|e| SearchError::InvalidResponse {
                    provider: PROVIDER_ID,
                    message: e.to_string(),
                })?;

        let mut result = SearchResponse::new(&query.query, PROVIDER_ID);
        let target_source = if query.sources.contains(&SearchSource::News) {
            SearchSource::News
        } else {
            SearchSource::Web
        };
        for raw in parsed.results {
            result.push(target_source, raw.into_hit());
        }
        result.elapsed_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        Ok(result)
    }
}

const fn time_range_label(range: TimeRange) -> &'static str {
    match range {
        TimeRange::Day => "day",
        TimeRange::Week => "week",
        TimeRange::Month => "month",
        TimeRange::Year => "year",
    }
}

fn topic_for(sources: &[SearchSource]) -> &'static str {
    if sources.contains(&SearchSource::News) {
        "news"
    } else {
        "general"
    }
}

#[derive(Debug, Serialize)]
struct TavilyRequest<'a> {
    api_key: &'a str,
    query: &'a str,
    max_results: usize,
    search_depth: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    time_range: Option<&'static str>,
    topic: &'static str,
}

#[derive(Debug, Deserialize)]
struct TavilyResponse {
    #[serde(default)]
    results: Vec<TavilyResult>,
}

#[derive(Debug, Deserialize)]
struct TavilyResult {
    #[serde(default)]
    title: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    published_date: Option<String>,
}

impl TavilyResult {
    fn into_hit(self) -> SearchHit {
        SearchHit {
            title: self.title,
            url: self.url,
            description: self.content.filter(|s| !s.is_empty()),
            published_at: self.published_date,
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
    use wiremock::matchers::{body_partial_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    #[tokio::test]
    async fn maps_results_to_web() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/search"))
            .and(body_partial_json(serde_json::json!({
                "query": "rust",
                "topic": "general"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [
                    { "title": "Rust", "url": "https://r.io", "content": "lang", "published_date": "2025-10-01" }
                ]
            })))
            .mount(&server)
            .await;

        let provider = Tavily::with_base_url("KEY", server.uri()).unwrap();
        let response = provider.search(&SearchQuery::new("rust")).await.unwrap();

        assert_eq!(response.provider, "tavily");
        assert_eq!(response.web.len(), 1);
        assert_eq!(response.web[0].published_at.as_deref(), Some("2025-10-01"));
    }

    #[tokio::test]
    async fn news_source_routes_to_news_bucket() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/search"))
            .and(body_partial_json(serde_json::json!({ "topic": "news" })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{ "title": "N", "url": "https://n" }]
            })))
            .mount(&server)
            .await;

        let provider = Tavily::with_base_url("KEY", server.uri()).unwrap();
        let query = SearchQuery::new("q").with_sources(vec![SearchSource::News]);
        let response = provider.search(&query).await.unwrap();
        assert_eq!(response.news.len(), 1);
        assert_eq!(response.web.len(), 0);
    }

    #[tokio::test]
    async fn http_error_is_reported() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(403))
            .mount(&server)
            .await;

        let provider = Tavily::with_base_url("KEY", server.uri()).unwrap();
        let err = provider.search(&SearchQuery::new("q")).await.unwrap_err();
        assert!(matches!(
            err,
            SearchError::Http {
                provider: "tavily",
                ..
            }
        ));
    }
}
