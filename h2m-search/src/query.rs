//! Search request parameters.

/// Search query with provider-agnostic parameters.
///
/// Construct with [`SearchQuery::new`] and refine using the builder-style
/// setters. Unset fields fall back to provider defaults.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SearchQuery {
    /// The search query string.
    pub query: String,
    /// Maximum number of results per source (default `10`, range `1..=100`).
    pub limit: usize,
    /// Result categories to request (default `[Web]`).
    pub sources: Vec<SearchSource>,
    /// Time-range filter.
    pub time_range: Option<TimeRange>,
    /// ISO 639-1 language code (e.g. `"en"`, `"zh"`).
    pub language: Option<String>,
    /// ISO 3166-1 alpha-2 country code (e.g. `"us"`, `"cn"`).
    pub country: Option<String>,
    /// Safe-search filter level.
    pub safesearch: SafeSearch,
}

impl SearchQuery {
    /// Creates a new query with only the query text set and defaults for
    /// everything else.
    #[must_use]
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            limit: 10,
            sources: vec![SearchSource::Web],
            time_range: None,
            language: None,
            country: None,
            safesearch: SafeSearch::Moderate,
        }
    }

    /// Sets the maximum result count. Values are clamped to `1..=100`.
    #[must_use]
    pub const fn with_limit(mut self, limit: usize) -> Self {
        self.limit = if limit == 0 {
            1
        } else if limit > 100 {
            100
        } else {
            limit
        };
        self
    }

    /// Replaces the source list.
    #[must_use]
    pub fn with_sources(mut self, sources: Vec<SearchSource>) -> Self {
        if sources.is_empty() {
            self.sources = vec![SearchSource::Web];
        } else {
            self.sources = sources;
        }
        self
    }

    /// Sets the time-range filter.
    #[must_use]
    pub const fn with_time_range(mut self, time_range: TimeRange) -> Self {
        self.time_range = Some(time_range);
        self
    }

    /// Sets the language filter.
    #[must_use]
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Sets the country filter.
    #[must_use]
    pub fn with_country(mut self, country: impl Into<String>) -> Self {
        self.country = Some(country.into());
        self
    }

    /// Sets the safe-search level.
    #[must_use]
    pub const fn with_safesearch(mut self, safesearch: SafeSearch) -> Self {
        self.safesearch = safesearch;
        self
    }

    /// Returns `true` if the query has a non-empty search string.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.query.trim().is_empty()
    }
}

/// Search result category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchSource {
    /// General web results.
    Web,
    /// News articles.
    News,
    /// Image results.
    Images,
}

impl SearchSource {
    /// Returns the canonical lowercase identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::News => "news",
            Self::Images => "images",
        }
    }
}

/// Time-range filter applied by providers that support it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeRange {
    /// Past 24 hours.
    Day,
    /// Past 7 days.
    Week,
    /// Past 30 days.
    Month,
    /// Past 12 months.
    Year,
}

impl TimeRange {
    /// Returns the SearXNG-native identifier.
    #[must_use]
    pub const fn as_searxng(self) -> &'static str {
        match self {
            Self::Day => "day",
            Self::Week | Self::Month => "month",
            Self::Year => "year",
        }
    }

    /// Returns the Firecrawl-style `tbs` identifier.
    #[must_use]
    pub const fn as_tbs(self) -> &'static str {
        match self {
            Self::Day => "qdr:d",
            Self::Week => "qdr:w",
            Self::Month => "qdr:m",
            Self::Year => "qdr:y",
        }
    }
}

/// Safe-search filter level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SafeSearch {
    /// No filtering.
    Off,
    /// Moderate filtering (default).
    #[default]
    Moderate,
    /// Strict filtering.
    Strict,
}

impl SafeSearch {
    /// Returns the numeric value used by `SearXNG` and most other engines.
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        match self {
            Self::Off => 0,
            Self::Moderate => 1,
            Self::Strict => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_defaults() {
        let q = SearchQuery::new("rust");
        assert_eq!(q.query, "rust");
        assert_eq!(q.limit, 10);
        assert_eq!(q.sources, vec![SearchSource::Web]);
        assert_eq!(q.safesearch, SafeSearch::Moderate);
    }

    #[test]
    fn limit_clamp() {
        assert_eq!(SearchQuery::new("x").with_limit(0).limit, 1);
        assert_eq!(SearchQuery::new("x").with_limit(500).limit, 100);
        assert_eq!(SearchQuery::new("x").with_limit(25).limit, 25);
    }

    #[test]
    fn empty_sources_fallback_to_web() {
        let q = SearchQuery::new("x").with_sources(vec![]);
        assert_eq!(q.sources, vec![SearchSource::Web]);
    }

    #[test]
    fn is_valid_rejects_whitespace() {
        assert!(!SearchQuery::new("   ").is_valid());
        assert!(SearchQuery::new("x").is_valid());
    }
}
