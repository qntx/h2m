//! Search response types.
//!
//! JSON output uses **camelCase** field names, consistent with the rest of the
//! h2m ecosystem and the Firecrawl API convention.

use serde::Serialize;

use crate::query::SearchSource;

/// A single search result.
///
/// Fields marked `Option` are populated on a best-effort basis; providers
/// that do not expose a piece of data leave it `None` and it is then
/// omitted from the serialised JSON (`#[serde(skip_serializing_if)]`).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SearchHit {
    /// Result title.
    pub title: String,
    /// Canonical URL.
    pub url: String,
    /// Short description or snippet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Publication timestamp as returned by the provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
    /// Upstream engine identifier (`SearXNG` exposes this; others omit it).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,
    /// Relevance score in `[0, 1]` when the provider supplies one
    /// (Tavily's ranking signal; other providers leave this `None`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
}

impl SearchHit {
    /// Creates a bare hit with only the required fields.
    #[must_use]
    pub fn new(title: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            url: url.into(),
            description: None,
            published_at: None,
            engine: None,
            score: None,
        }
    }

    /// Attaches a description snippet.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Attaches a publication timestamp (provider-supplied, untyped).
    #[must_use]
    pub fn with_published_at(mut self, published_at: impl Into<String>) -> Self {
        self.published_at = Some(published_at.into());
        self
    }

    /// Attaches the upstream engine identifier.
    #[must_use]
    pub fn with_engine(mut self, engine: impl Into<String>) -> Self {
        self.engine = Some(engine.into());
        self
    }

    /// Attaches a relevance score (if the provider supplies one).
    #[must_use]
    pub const fn with_score(mut self, score: f64) -> Self {
        self.score = Some(score);
        self
    }
}

/// Complete search response grouped by source.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SearchResponse {
    /// Echoed query string.
    pub query: String,
    /// Provider identifier that served the request.
    pub provider: &'static str,
    /// Optional LLM-generated answer (Tavily `include_answer` feature).
    ///
    /// `None` when not requested or not supplied by the provider. The
    /// field is omitted from the serialised JSON when `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answer: Option<String>,
    /// Web results.
    pub web: Vec<SearchHit>,
    /// News results.
    pub news: Vec<SearchHit>,
    /// Image results.
    pub images: Vec<SearchHit>,
    /// Elapsed wall time in milliseconds.
    pub elapsed_ms: u64,
}

impl SearchResponse {
    /// Constructs an empty response for the given provider/query.
    #[must_use]
    pub fn new(query: impl Into<String>, provider: &'static str) -> Self {
        Self {
            query: query.into(),
            provider,
            answer: None,
            web: Vec::new(),
            news: Vec::new(),
            images: Vec::new(),
            elapsed_ms: 0,
        }
    }

    /// Iterates over every hit across all sources.
    pub fn all_hits(&self) -> impl Iterator<Item = &SearchHit> {
        self.web
            .iter()
            .chain(self.news.iter())
            .chain(self.images.iter())
    }

    /// Total number of results across all sources.
    #[must_use]
    pub const fn total(&self) -> usize {
        self.web.len() + self.news.len() + self.images.len()
    }

    /// Pushes a hit into the slot matching the given source.
    pub fn push(&mut self, source: SearchSource, hit: SearchHit) {
        match source {
            SearchSource::Web => self.web.push(hit),
            SearchSource::News => self.news.push(hit),
            SearchSource::Images => self.images.push(hit),
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

    use super::*;

    #[test]
    fn total_counts_all_sources() {
        let mut r = SearchResponse::new("q", "test");
        r.push(SearchSource::Web, SearchHit::new("a", "https://a"));
        r.push(SearchSource::News, SearchHit::new("b", "https://b"));
        r.push(SearchSource::Images, SearchHit::new("c", "https://c"));
        assert_eq!(r.total(), 3);
        assert_eq!(r.all_hits().count(), 3);
    }

    #[test]
    fn serializes_camel_case_and_skips_none() {
        let mut r = SearchResponse::new("rust", "searxng");
        r.elapsed_ms = 123;
        r.push(SearchSource::Web, SearchHit::new("Rust", "https://r.io"));
        let json = serde_json::to_value(&r).unwrap();
        assert_eq!(json["elapsedMs"], 123);
        assert_eq!(json["web"][0]["title"], "Rust");
        assert!(json["web"][0].get("description").is_none());
    }
}
