//! `SearXNG` provider.
//!
//! `SearXNG` is a free, open-source metasearch engine that aggregates results
//! from Google, Bing, `DuckDuckGo`, and dozens of other backends. It requires
//! no API key, but the caller must supply an instance URL — self-hosted
//! deployments are recommended for reliability.
//!
//! Enable the `searxng` feature (on by default) to use this provider.

use std::time::{Duration, Instant};

use serde::Deserialize;

use crate::error::SearchError;
use crate::query::{SearchQuery, SearchSource};
use crate::response::{SearchHit, SearchResponse};

const PROVIDER_ID: &str = "searxng";

/// `SearXNG` search provider.
#[derive(Debug, Clone)]
pub struct SearXNG {
    client: reqwest::Client,
    base_url: url::Url,
}

impl SearXNG {
    /// Creates a new provider using the given `base_url`.
    ///
    /// # Errors
    ///
    /// Returns [`SearchError::Config`] if `base_url` is not a valid URL or the
    /// default HTTP client cannot be built.
    pub fn new(base_url: impl AsRef<str>) -> Result<Self, SearchError> {
        let base = url::Url::parse(base_url.as_ref()).map_err(|e| SearchError::Config {
            message: format!("invalid SearXNG base URL: {e}"),
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
        })
    }

    /// Wraps an existing [`reqwest::Client`] with the given `base_url`.
    ///
    /// Use this when you want to share a pool of connections across multiple
    /// providers or with the [`h2m::scrape::Scraper`] pipeline.
    ///
    /// # Errors
    ///
    /// Returns [`SearchError::Config`] if `base_url` cannot be parsed.
    pub fn with_client(
        client: reqwest::Client,
        base_url: impl AsRef<str>,
    ) -> Result<Self, SearchError> {
        let base = url::Url::parse(base_url.as_ref()).map_err(|e| SearchError::Config {
            message: format!("invalid SearXNG base URL: {e}"),
        })?;
        Ok(Self {
            client,
            base_url: base,
        })
    }

    /// Executes a search against the `SearXNG` instance.
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
            .join("search")
            .map_err(|e| SearchError::Config {
                message: format!("cannot build /search endpoint: {e}"),
            })?;

        let categories = build_categories(&query.sources);
        let safesearch = query.safesearch.as_u8().to_string();
        let pageno = "1";

        {
            let mut pairs = endpoint.query_pairs_mut();
            pairs.append_pair("q", &query.query);
            pairs.append_pair("format", "json");
            pairs.append_pair("pageno", pageno);
            pairs.append_pair("safesearch", &safesearch);
            if !categories.is_empty() {
                pairs.append_pair("categories", &categories);
            }
            if let Some(tr) = query.time_range {
                pairs.append_pair("time_range", tr.as_searxng());
            }
            if let Some(lang) = &query.language {
                pairs.append_pair("language", lang);
            }
        }

        let response = self
            .client
            .get(endpoint)
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

        let body: SearxngResponse =
            response
                .json()
                .await
                .map_err(|e| SearchError::InvalidResponse {
                    provider: PROVIDER_ID,
                    message: e.to_string(),
                })?;

        let mut result = SearchResponse::new(&query.query, PROVIDER_ID);
        for (i, raw) in body.results.into_iter().enumerate() {
            if i >= query.limit {
                break;
            }
            let source = classify_source(raw.category.as_deref());
            result.push(source, raw.into_hit());
        }
        result.elapsed_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        Ok(result)
    }
}

fn build_categories(sources: &[SearchSource]) -> String {
    let mut out = Vec::with_capacity(sources.len());
    for src in sources {
        let cat = match src {
            SearchSource::Web => "general",
            SearchSource::News => "news",
            SearchSource::Images => "images",
        };
        if !out.contains(&cat) {
            out.push(cat);
        }
    }
    out.join(",")
}

fn classify_source(category: Option<&str>) -> SearchSource {
    match category {
        Some("news") => SearchSource::News,
        Some("images") => SearchSource::Images,
        _ => SearchSource::Web,
    }
}

#[derive(Debug, Deserialize)]
struct SearxngResponse {
    #[serde(default)]
    results: Vec<SearxngResult>,
}

#[derive(Debug, Deserialize)]
struct SearxngResult {
    #[serde(default)]
    url: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    engine: Option<String>,
    #[serde(default)]
    category: Option<String>,
    #[serde(default, rename = "publishedDate")]
    published_date: Option<String>,
}

impl SearxngResult {
    fn into_hit(self) -> SearchHit {
        SearchHit {
            title: self.title,
            url: self.url,
            description: self.content.filter(|s| !s.is_empty()),
            published_at: self.published_date,
            engine: self.engine,
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
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    #[tokio::test]
    async fn happy_path_maps_fields() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/search"))
            .and(query_param("q", "rust"))
            .and(query_param("format", "json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [
                    {
                        "url": "https://rust-lang.org",
                        "title": "Rust",
                        "content": "A language empowering everyone.",
                        "engine": "google",
                        "category": "general"
                    },
                    {
                        "url": "https://news.example.com/rust",
                        "title": "Rust news",
                        "content": "Latest",
                        "engine": "bing",
                        "category": "news"
                    }
                ]
            })))
            .mount(&server)
            .await;

        let provider = SearXNG::new(server.uri()).unwrap();
        let response = provider.search(&SearchQuery::new("rust")).await.unwrap();

        assert_eq!(response.provider, "searxng");
        assert_eq!(response.web.len(), 1);
        assert_eq!(response.news.len(), 1);
        assert_eq!(response.web[0].title, "Rust");
        assert_eq!(
            response.web[0].description.as_deref(),
            Some("A language empowering everyone.")
        );
        assert_eq!(response.web[0].engine.as_deref(), Some("google"));
    }

    #[tokio::test]
    async fn limit_truncates_results() {
        let server = MockServer::start().await;
        let results: Vec<_> = (0..20)
            .map(|i| {
                serde_json::json!({
                    "url": format!("https://example.com/{i}"),
                    "title": format!("Result {i}"),
                    "content": "snippet",
                    "category": "general"
                })
            })
            .collect();
        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({ "results": results })),
            )
            .mount(&server)
            .await;

        let provider = SearXNG::new(server.uri()).unwrap();
        let response = provider
            .search(&SearchQuery::new("x").with_limit(5))
            .await
            .unwrap();
        assert_eq!(response.total(), 5);
    }

    #[tokio::test]
    async fn http_error_is_reported() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&server)
            .await;

        let provider = SearXNG::new(server.uri()).unwrap();
        let err = provider.search(&SearchQuery::new("x")).await.unwrap_err();
        assert!(matches!(
            err,
            SearchError::Http {
                provider: "searxng",
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
        let provider = SearXNG::new("https://searx.example.org").unwrap();
        let err = runtime
            .block_on(provider.search(&SearchQuery::new("")))
            .unwrap_err();
        assert!(matches!(err, SearchError::Config { .. }));
    }

    #[test]
    fn bad_base_url_is_config_error() {
        let err = SearXNG::new("not a url").unwrap_err();
        assert!(matches!(err, SearchError::Config { .. }));
    }
}
