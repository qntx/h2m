//! Query-parameter encoding for the `DuckDuckGo` HTML endpoints.
//!
//! `DuckDuckGo` uses compact abbreviations (`kl`, `df`, `safe`) that do not
//! map 1:1 onto the portable [`SearchQuery`] fields. Keeping the encoders
//! here isolates the upstream quirks from the request pipeline.

use crate::query::{SafeSearch, SearchQuery, TimeRange};

/// Returns the static, broadly compatible `Accept` header value.
pub(super) const fn accept_header() -> &'static str {
    "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
}

/// Builds a locale-weighted `Accept-Language` header value.
pub(super) fn accept_language(query: &SearchQuery) -> String {
    match query.language.as_deref() {
        Some(code) if !code.is_empty() => format!("{code},{code};q=0.9,en;q=0.8"),
        _ => "en-US,en;q=0.9".into(),
    }
}

/// Maps [`SafeSearch`] onto the `safe` form-field token.
pub(super) const fn safesearch_token(safe: SafeSearch) -> &'static str {
    match safe {
        SafeSearch::Off => "-2",
        SafeSearch::Moderate => "-1",
        SafeSearch::Strict => "1",
    }
}

/// Maps [`TimeRange`] onto the `df` form-field token.
pub(super) const fn time_range_token(range: TimeRange) -> &'static str {
    match range {
        TimeRange::Day => "d",
        TimeRange::Week => "w",
        TimeRange::Month => "m",
        TimeRange::Year => "y",
    }
}

/// Builds the `kl` (region+language) parameter `DuckDuckGo` expects.
///
/// Returns `wt-wt` (no region) when neither country nor language is set.
pub(super) fn region_code(query: &SearchQuery) -> String {
    let country = query.country.as_deref().unwrap_or("").to_ascii_lowercase();
    let language = query.language.as_deref().unwrap_or("").to_ascii_lowercase();
    if country.is_empty() && language.is_empty() {
        return "wt-wt".into();
    }
    let country = if country.is_empty() { "us" } else { &country };
    let language = if language.is_empty() { "en" } else { &language };
    format!("{country}-{language}")
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn region_code_maps_country_language() {
        let q_full = SearchQuery::new("x").with_country("us").with_language("en");
        assert_eq!(region_code(&q_full), "us-en");
        let q_country = SearchQuery::new("x").with_country("CN");
        assert_eq!(region_code(&q_country), "cn-en");
        let q_lang = SearchQuery::new("x").with_language("ZH");
        assert_eq!(region_code(&q_lang), "us-zh");
        assert_eq!(region_code(&SearchQuery::new("x")), "wt-wt");
    }

    #[test]
    fn safesearch_and_time_range_mapping() {
        assert_eq!(safesearch_token(SafeSearch::Off), "-2");
        assert_eq!(safesearch_token(SafeSearch::Moderate), "-1");
        assert_eq!(safesearch_token(SafeSearch::Strict), "1");
        assert_eq!(time_range_token(TimeRange::Day), "d");
        assert_eq!(time_range_token(TimeRange::Week), "w");
        assert_eq!(time_range_token(TimeRange::Month), "m");
        assert_eq!(time_range_token(TimeRange::Year), "y");
    }

    #[test]
    fn accept_language_falls_back_to_english() {
        assert_eq!(accept_language(&SearchQuery::new("x")), "en-US,en;q=0.9");
        assert_eq!(
            accept_language(&SearchQuery::new("x").with_language("zh")),
            "zh,zh;q=0.9,en;q=0.8"
        );
    }
}
