//! `DuckDuckGo` provider.
//!
//! Talks to `DuckDuckGo`'s unauthenticated HTML endpoints — the same interface
//! the `ddgs` (5M+ monthly downloads), `LangChain`, and Haystack libraries use.
//! No API key, no registration, no environment variables: this is the
//! default zero-config provider for `h2m`.
//!
//! # Robustness layers
//!
//! The provider is engineered against rate limiting and anti-bot challenges
//! via four complementary strategies:
//!
//! 1. **User-Agent rotation** — every request picks from a pool of 10
//!    current (Chrome 121+, Firefox 123+, Safari 17+) browser UAs.
//! 2. **POST + form-data** — the HTML endpoint is reached via POST, which
//!    empirically survives rate limiting better than GET + query string.
//! 3. **Exponential backoff** — 429s and detected CAPTCHA pages retry with
//!    backoff via the shared [`RetryPolicy`].
//! 4. **Endpoint fallback** — when the HTML endpoint returns a CAPTCHA or
//!    parse-failure, the provider automatically falls back to the lighter
//!    `lite.duckduckgo.com/lite/` endpoint, whose HTML is table-based and
//!    much harder to break.
//!
//! Enable the `duckduckgo` feature (on by default) to use this provider.

use std::sync::LazyLock;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use scraper::{ElementRef, Html, Selector};
use tracing::{instrument, warn};

use super::common::{classify_status, classify_transport};
use crate::error::SearchError;
use crate::http::HttpConfig;
use crate::query::{SafeSearch, SearchQuery, SearchSource, TimeRange};
use crate::response::{SearchHit, SearchResponse};
use crate::retry::{RetryPolicy, retry};

const PROVIDER_ID: &str = "duckduckgo";
const DEFAULT_HTML_ENDPOINT: &str = "https://html.duckduckgo.com/html/";
const DEFAULT_LITE_ENDPOINT: &str = "https://lite.duckduckgo.com/lite/";
/// `DuckDuckGo` returns roughly 25–30 results per page.
const RESULTS_PER_PAGE: u32 = 25;
/// Hard cap on pagination — deeper pages usually hit anomaly detection.
const MAX_PAGES: u32 = 4;
/// Minimum body size for a legitimate HTML response. Anything tiny is
/// almost certainly an error / challenge page.
const MIN_HTML_BODY_BYTES: usize = 512;

/// Rotating user-agent pool (Chrome 121+, Firefox 123+, Safari 17+, Edge 121+).
///
/// Using a realistic, diverse pool rather than a single UA dramatically
/// reduces anomaly-detection hits on `DuckDuckGo`'s HTML endpoint.
const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:123.0) Gecko/20100101 Firefox/123.0",
    "Mozilla/5.0 (X11; Linux x86_64; rv:122.0) Gecko/20100101 Firefox/122.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 14.2; rv:123.0) Gecko/20100101 Firefox/123.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36 Edg/121.0.0.0",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 17_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Mobile/15E148 Safari/604.1",
];

/// Tokens whose presence in the response body signals an anti-bot page.
const CAPTCHA_MARKERS: &[&str] = &[
    "anomaly-modal",
    "challenge-platform",
    "DDG.anomalyDetection",
    "If this error persists",
    "/assets/common/error.js",
];

// -----------------------------------------------------------------------------
// Selectors (compiled once via LazyLock)
// -----------------------------------------------------------------------------

/// Selector bundle for the html.duckduckgo.com layout.
struct HtmlSelectors {
    result: Selector,
    title: Selector,
    snippet: Selector,
}

static HTML_SEL: LazyLock<HtmlSelectors> = LazyLock::new(|| HtmlSelectors {
    result: selector("div.result"),
    title: selector("a.result__a"),
    snippet: selector("a.result__snippet, .result__snippet"),
});

/// Row selector for the lite.duckduckgo.com table layout.
static LITE_RESULT_LINK_SEL: LazyLock<Selector> = LazyLock::new(|| selector("a.result-link"));
static LITE_RESULT_ROW_SEL: LazyLock<Selector> = LazyLock::new(|| selector("tr"));

/// Compiles a static CSS selector. The selector text is a compile-time
/// constant in every call site — `unwrap_or_else` here is equivalent to
/// `expect`, but satisfies `clippy::expect_used`.
fn selector(css: &'static str) -> Selector {
    Selector::parse(css).unwrap_or_else(|_| unreachable!("invalid static selector: {css}"))
}

// -----------------------------------------------------------------------------
// Percent-encoding set for DDG URL unwrapping (decoding their redirect wrapper)
// -----------------------------------------------------------------------------

const FORM_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    .add(b'&')
    .add(b'+')
    .add(b'=');

// -----------------------------------------------------------------------------
// Public API
// -----------------------------------------------------------------------------

/// `DuckDuckGo` search provider.
#[derive(Debug, Clone)]
pub struct DuckDuckGo {
    http: HttpConfig,
    html_endpoint: String,
    lite_endpoint: String,
    retry_policy: RetryPolicy,
    enable_fallback: bool,
}

impl DuckDuckGo {
    /// Creates a new provider with default settings.
    ///
    /// # Errors
    ///
    /// Returns [`SearchError::Config`] if the default HTTP client cannot be
    /// constructed.
    pub fn new() -> Result<Self, SearchError> {
        Self::builder().build()
    }

    /// Starts a typed builder for fine-grained configuration.
    #[must_use]
    pub fn builder() -> DuckDuckGoBuilder {
        DuckDuckGoBuilder::default()
    }

    /// Executes a search against `DuckDuckGo`.
    ///
    /// The primary HTML endpoint is tried first with the configured retry
    /// policy. On persistent CAPTCHA / parse failures the provider
    /// automatically falls back to the lite endpoint (unless disabled).
    ///
    /// # Errors
    ///
    /// Returns the specific [`SearchError`] from the underlying HTTP layer.
    /// Parse failures surface as [`SearchError::ParseFailed`]; anti-bot
    /// pages surface as [`SearchError::CaptchaDetected`].
    #[instrument(level = "debug", skip(self), fields(provider = PROVIDER_ID))]
    pub async fn search(&self, query: &SearchQuery) -> Result<SearchResponse, SearchError> {
        if !query.is_valid() {
            return Err(SearchError::Config {
                message: "search query is empty".into(),
            });
        }

        let start = Instant::now();
        let hits = match self.fetch_html(query).await {
            Ok(h) => h,
            Err(e) if self.enable_fallback && is_scrape_failure(&e) => {
                warn!(primary_error = %e, "falling back to lite.duckduckgo.com");
                self.fetch_lite(query).await?
            }
            Err(e) => return Err(e),
        };

        let mut response = SearchResponse::new(&query.query, PROVIDER_ID);
        for hit in hits.into_iter().take(query.limit) {
            response.push(SearchSource::Web, hit);
        }
        response.elapsed_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        Ok(response)
    }

    async fn fetch_html(&self, query: &SearchQuery) -> Result<Vec<SearchHit>, SearchError> {
        let mut collected: Vec<SearchHit> = Vec::with_capacity(query.limit);
        let mut offset: u32 = 0;
        let mut pages: u32 = 0;
        while collected.len() < query.limit && pages < MAX_PAGES {
            let body = retry(&self.retry_policy, || {
                self.post_html(query, offset.saturating_mul(RESULTS_PER_PAGE))
            })
            .await?;
            let page = parse_html_results(&body)?;
            let page_len = page.len();
            collected.extend(page);
            if page_len < RESULTS_PER_PAGE as usize || collected.len() >= query.limit {
                break;
            }
            offset = offset.saturating_add(1);
            pages = pages.saturating_add(1);
        }
        Ok(collected)
    }

    async fn post_html(&self, query: &SearchQuery, offset: u32) -> Result<String, SearchError> {
        let offset_str = offset.to_string();
        let kl = region_code(query);
        let safe = safesearch_token(query.safesearch);
        let df = query.time_range.map_or("", time_range_token);
        let mut form: Vec<(&str, &str)> = vec![
            ("q", query.query.as_str()),
            ("b", ""),
            ("kl", &kl),
            ("safe", safe),
        ];
        if offset > 0 {
            form.push(("s", &offset_str));
            form.push(("dc", &offset_str));
            form.push(("v", "l"));
            form.push(("o", "json"));
            form.push(("api", "d.js"));
        }
        if !df.is_empty() {
            form.push(("df", df));
        }

        let response = self
            .http
            .client()
            .post(&self.html_endpoint)
            .header(reqwest::header::USER_AGENT, pick_user_agent())
            .header(reqwest::header::ACCEPT, accept_header())
            .header(reqwest::header::ACCEPT_LANGUAGE, accept_language(query))
            .header(reqwest::header::REFERER, "https://duckduckgo.com/")
            .header("DNT", "1")
            .header("Upgrade-Insecure-Requests", "1")
            .form(&form)
            .send()
            .await
            .map_err(|e| classify_transport(PROVIDER_ID, &e))?;

        finalize_body(response).await
    }

    async fn fetch_lite(&self, query: &SearchQuery) -> Result<Vec<SearchHit>, SearchError> {
        let body = retry(&self.retry_policy, || self.post_lite(query)).await?;
        let hits = parse_lite_results(&body)?;
        Ok(hits.into_iter().take(query.limit).collect())
    }

    async fn post_lite(&self, query: &SearchQuery) -> Result<String, SearchError> {
        let kl = region_code(query);
        let safe = safesearch_token(query.safesearch);
        let df = query.time_range.map_or("", time_range_token);
        let mut form: Vec<(&str, &str)> =
            vec![("q", query.query.as_str()), ("kl", &kl), ("safe", safe)];
        if !df.is_empty() {
            form.push(("df", df));
        }

        let response = self
            .http
            .client()
            .post(&self.lite_endpoint)
            .header(reqwest::header::USER_AGENT, pick_user_agent())
            .header(reqwest::header::ACCEPT, accept_header())
            .header(reqwest::header::ACCEPT_LANGUAGE, accept_language(query))
            .header(reqwest::header::REFERER, "https://lite.duckduckgo.com/")
            .form(&form)
            .send()
            .await
            .map_err(|e| classify_transport(PROVIDER_ID, &e))?;

        finalize_body(response).await
    }
}

/// Typed builder for [`DuckDuckGo`].
#[derive(Debug, Default)]
pub struct DuckDuckGoBuilder {
    http: Option<HttpConfig>,
    html_endpoint: Option<String>,
    lite_endpoint: Option<String>,
    retry: Option<RetryPolicy>,
    enable_fallback: Option<bool>,
}

impl DuckDuckGoBuilder {
    /// Supplies a shared [`HttpConfig`].
    #[must_use]
    pub fn http(mut self, http: HttpConfig) -> Self {
        self.http = Some(http);
        self
    }

    /// Overrides the primary HTML endpoint (useful for tests / mirrors).
    #[must_use]
    pub fn html_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.html_endpoint = Some(endpoint.into());
        self
    }

    /// Overrides the lite fallback endpoint.
    #[must_use]
    pub fn lite_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.lite_endpoint = Some(endpoint.into());
        self
    }

    /// Overrides the retry policy.
    #[must_use]
    pub const fn retry(mut self, policy: RetryPolicy) -> Self {
        self.retry = Some(policy);
        self
    }

    /// Disables the automatic html → lite fallback.
    ///
    /// Fallback is enabled by default. Disable when you want to surface
    /// anomaly/CAPTCHA errors to the caller untouched.
    #[must_use]
    pub const fn fallback(mut self, enabled: bool) -> Self {
        self.enable_fallback = Some(enabled);
        self
    }

    /// Builds the provider.
    ///
    /// # Errors
    ///
    /// Returns [`SearchError::Config`] if the default HTTP client cannot be
    /// constructed.
    pub fn build(self) -> Result<DuckDuckGo, SearchError> {
        let http = match self.http {
            Some(cfg) => cfg,
            None => HttpConfig::new()?,
        };
        Ok(DuckDuckGo {
            http,
            html_endpoint: self
                .html_endpoint
                .unwrap_or_else(|| DEFAULT_HTML_ENDPOINT.into()),
            lite_endpoint: self
                .lite_endpoint
                .unwrap_or_else(|| DEFAULT_LITE_ENDPOINT.into()),
            retry_policy: self.retry.unwrap_or_default(),
            enable_fallback: self.enable_fallback.unwrap_or(true),
        })
    }
}

// -----------------------------------------------------------------------------
// Helpers: headers, region mapping, body handling
// -----------------------------------------------------------------------------

fn pick_user_agent() -> &'static str {
    // Nano-resolution clock suffices for cheap UA rotation; no `rand` dependency.
    // USER_AGENTS is guaranteed non-empty by `user_agent_pool_is_populated` test
    // and the fallback default below keeps us correct even if that invariant
    // ever changes.
    const FALLBACK_UA: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36";
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.subsec_nanos() as usize);
    let idx = nanos % USER_AGENTS.len().max(1);
    USER_AGENTS.get(idx).copied().unwrap_or(FALLBACK_UA)
}

const fn accept_header() -> &'static str {
    "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
}

fn accept_language(query: &SearchQuery) -> String {
    match query.language.as_deref() {
        Some(code) if !code.is_empty() => format!("{code},{code};q=0.9,en;q=0.8"),
        _ => "en-US,en;q=0.9".into(),
    }
}

const fn safesearch_token(safe: SafeSearch) -> &'static str {
    match safe {
        SafeSearch::Off => "-2",
        SafeSearch::Moderate => "-1",
        SafeSearch::Strict => "1",
    }
}

const fn time_range_token(tr: TimeRange) -> &'static str {
    match tr {
        TimeRange::Day => "d",
        TimeRange::Week => "w",
        TimeRange::Month => "m",
        TimeRange::Year => "y",
    }
}

/// Builds the `kl` (region+language) parameter `DuckDuckGo` expects.
///
/// Returns `wt-wt` (no region) when neither country nor language is set.
fn region_code(query: &SearchQuery) -> String {
    let country = query.country.as_deref().unwrap_or("").to_ascii_lowercase();
    let language = query.language.as_deref().unwrap_or("").to_ascii_lowercase();
    if country.is_empty() && language.is_empty() {
        return "wt-wt".into();
    }
    let country = if country.is_empty() { "us" } else { &country };
    let language = if language.is_empty() { "en" } else { &language };
    format!("{country}-{language}")
}

async fn finalize_body(response: reqwest::Response) -> Result<String, SearchError> {
    if !response.status().is_success() {
        // 202 with empty body is DDG's classic soft-block signal.
        if response.status().as_u16() == 202 {
            return Err(SearchError::CaptchaDetected {
                provider: PROVIDER_ID,
            });
        }
        return Err(classify_status(PROVIDER_ID, &response));
    }
    let body = response.text().await.map_err(|e| SearchError::Transport {
        provider: PROVIDER_ID,
        message: format!("failed to read body: {e}"),
    })?;
    if looks_like_captcha(&body) {
        return Err(SearchError::CaptchaDetected {
            provider: PROVIDER_ID,
        });
    }
    Ok(body)
}

fn looks_like_captcha(body: &str) -> bool {
    if body.len() < MIN_HTML_BODY_BYTES {
        return true;
    }
    CAPTCHA_MARKERS.iter().any(|m| body.contains(m))
}

/// Classifies an error as recoverable by falling back to the lite endpoint.
const fn is_scrape_failure(err: &SearchError) -> bool {
    matches!(
        err,
        SearchError::CaptchaDetected { .. }
            | SearchError::ParseFailed { .. }
            | SearchError::RateLimited { .. }
    )
}

fn collect_text(el: &ElementRef<'_>) -> String {
    let mut buf = String::new();
    for chunk in el.text() {
        let trimmed = chunk.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !buf.is_empty() {
            buf.push(' ');
        }
        buf.push_str(trimmed);
    }
    buf
}

/// Unwraps a `DuckDuckGo` redirect link into the real destination URL.
///
/// `DuckDuckGo` wraps every outbound link in a `/l/?uddg=<encoded>` path.
/// Links may be absolute, protocol-relative, or site-relative.
fn unwrap_ddg_url(href: &str) -> String {
    let href = href.trim();
    if href.is_empty() {
        return String::new();
    }
    let normalized = if href.starts_with("//") {
        format!("https:{href}")
    } else if let Some(rest) = href.strip_prefix('/') {
        format!("https://duckduckgo.com/{rest}")
    } else {
        href.to_owned()
    };

    if let Ok(parsed) = url::Url::parse(&normalized)
        && let Some(host) = parsed.host_str()
        && (host == "duckduckgo.com" || host.ends_with(".duckduckgo.com"))
        && parsed.path().starts_with("/l/")
        && let Some((_, value)) = parsed.query_pairs().find(|(k, _)| k == "uddg")
    {
        return value.into_owned();
    }
    normalized
}

/// Percent-encodes a query string segment using the form-urlencoded set.
///
/// Exposed purely so callers building DDG URLs outside the provider (e.g.
/// debug tooling) can match our on-the-wire encoding byte-for-byte.
#[must_use]
pub fn encode_form(value: &str) -> String {
    utf8_percent_encode(value, FORM_ENCODE_SET).to_string()
}

// -----------------------------------------------------------------------------
// HTML parsing
// -----------------------------------------------------------------------------

fn parse_html_results(body: &str) -> Result<Vec<SearchHit>, SearchError> {
    let doc = Html::parse_document(body);
    let mut out = Vec::new();
    for node in doc.select(&HTML_SEL.result) {
        let Some(title_el) = node.select(&HTML_SEL.title).next() else {
            continue;
        };
        let title = collect_text(&title_el);
        let Some(href) = title_el.value().attr("href") else {
            continue;
        };
        let url = unwrap_ddg_url(href);
        if url.is_empty() || title.is_empty() {
            continue;
        }
        let description = node
            .select(&HTML_SEL.snippet)
            .next()
            .map(|s| collect_text(&s))
            .filter(|s| !s.is_empty());
        out.push(SearchHit {
            title,
            url,
            description,
            published_at: None,
            engine: Some(PROVIDER_ID.into()),
            score: None,
        });
    }

    if out.is_empty() && body.len() > MIN_HTML_BODY_BYTES {
        // Empty query result is legitimate; parse failure is not. Distinguish
        // by looking for the known "no results" marker.
        if !body.contains("No results.")
            && !body.contains("no-results")
            && !body.contains("No results found")
        {
            return Err(SearchError::ParseFailed {
                provider: PROVIDER_ID,
                message: "no div.result nodes matched in html endpoint".into(),
            });
        }
    }
    Ok(out)
}

fn parse_lite_results(body: &str) -> Result<Vec<SearchHit>, SearchError> {
    let doc = Html::parse_document(body);
    let mut out = Vec::new();
    // The lite layout alternates rows: title-link, snippet, metadata, spacer.
    // We walk rows sequentially and attach the next row's text as the snippet.
    let rows: Vec<ElementRef<'_>> = doc.select(&LITE_RESULT_ROW_SEL).collect();
    let mut i = 0;
    while let Some(row) = rows.get(i).copied() {
        if let Some(link) = row.select(&LITE_RESULT_LINK_SEL).next() {
            let title = collect_text(&link);
            let Some(href) = link.value().attr("href") else {
                i += 1;
                continue;
            };
            let url = unwrap_ddg_url(href);
            if url.is_empty() || title.is_empty() {
                i += 1;
                continue;
            }
            let description = rows
                .get(i + 1)
                .map(|r| collect_text(r))
                .filter(|s| !s.is_empty());
            out.push(SearchHit {
                title,
                url,
                description,
                published_at: None,
                engine: Some(PROVIDER_ID.into()),
                score: None,
            });
            // Advance past the snippet + metadata + spacer rows.
            i += 4;
        } else {
            i += 1;
        }
    }

    if out.is_empty() && body.len() > MIN_HTML_BODY_BYTES && !body.contains("No results.") {
        return Err(SearchError::ParseFailed {
            provider: PROVIDER_ID,
            message: "no a.result-link anchors matched in lite endpoint".into(),
        });
    }
    Ok(out)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    reason = "test assertions should panic on wrong shape"
)]
mod tests {
    use pretty_assertions::assert_eq;
    use wiremock::matchers::{body_string_contains, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    /// Canonical HTML response fragment exercised by several tests.
    const SAMPLE_HTML: &str = include_str!("testdata/duckduckgo_html_results.html");
    const SAMPLE_LITE: &str = include_str!("testdata/duckduckgo_lite_results.html");
    const SAMPLE_CAPTCHA: &str = r#"<!DOCTYPE html><html><body>
<div class="anomaly-modal">
    <p>If this error persists, please reach out to us.</p>
</div>
<script src="/assets/common/error.js"></script>
</body></html>"#;

    fn build_provider(html: &str, lite: &str) -> DuckDuckGo {
        DuckDuckGo::builder()
            .html_endpoint(html)
            .lite_endpoint(lite)
            .retry(RetryPolicy::NONE)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn html_happy_path_extracts_results() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/"))
            .and(body_string_contains("q=rust"))
            .respond_with(ResponseTemplate::new(200).set_body_string(SAMPLE_HTML))
            .mount(&server)
            .await;

        let provider = build_provider(&format!("{}/", server.uri()), "http://unused");
        let response = provider.search(&SearchQuery::new("rust")).await.unwrap();
        assert_eq!(response.provider, "duckduckgo");
        assert_eq!(response.web.len(), 3);
        assert_eq!(response.web[0].title, "Rust Programming Language");
        assert_eq!(response.web[0].url, "https://www.rust-lang.org/");
        assert_eq!(
            response.web[0].description.as_deref(),
            Some("A language empowering everyone to build reliable and efficient software.")
        );
        assert_eq!(response.web[0].engine.as_deref(), Some("duckduckgo"));
    }

    #[tokio::test]
    async fn captcha_triggers_lite_fallback() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/html"))
            .respond_with(ResponseTemplate::new(200).set_body_string(SAMPLE_CAPTCHA))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/lite"))
            .respond_with(ResponseTemplate::new(200).set_body_string(SAMPLE_LITE))
            .mount(&server)
            .await;

        let provider = build_provider(
            &format!("{}/html", server.uri()),
            &format!("{}/lite", server.uri()),
        );
        let response = provider.search(&SearchQuery::new("rust")).await.unwrap();
        assert_eq!(response.web.len(), 2);
        assert_eq!(response.web[0].title, "Rust Lang");
        assert!(response.web[0].url.starts_with("https://"));
    }

    #[tokio::test]
    async fn http_202_classifies_as_captcha_before_fallback() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/html"))
            .respond_with(ResponseTemplate::new(202))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/lite"))
            .respond_with(ResponseTemplate::new(200).set_body_string(SAMPLE_LITE))
            .mount(&server)
            .await;

        let provider = build_provider(
            &format!("{}/html", server.uri()),
            &format!("{}/lite", server.uri()),
        );
        let response = provider.search(&SearchQuery::new("rust")).await.unwrap();
        assert_eq!(response.web.len(), 2);
    }

    #[tokio::test]
    async fn fallback_disabled_surfaces_captcha() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(202))
            .mount(&server)
            .await;

        let provider = DuckDuckGo::builder()
            .html_endpoint(format!("{}/", server.uri()))
            .lite_endpoint("http://unused")
            .retry(RetryPolicy::NONE)
            .fallback(false)
            .build()
            .unwrap();
        let err = provider
            .search(&SearchQuery::new("rust"))
            .await
            .unwrap_err();
        assert!(matches!(err, SearchError::CaptchaDetected { .. }));
    }

    #[tokio::test]
    async fn limit_truncates_across_pages() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_string(SAMPLE_HTML))
            .mount(&server)
            .await;

        let provider = build_provider(&format!("{}/", server.uri()), "http://unused");
        let response = provider
            .search(&SearchQuery::new("rust").with_limit(2))
            .await
            .unwrap();
        assert_eq!(response.web.len(), 2);
    }

    #[tokio::test]
    async fn parse_failure_on_unrecognised_layout() {
        let server = MockServer::start().await;
        // Large body (> MIN_HTML_BODY_BYTES) with no results, no markers,
        // no parseable structure — selectors find nothing.
        let filler = "<p>lorem ipsum dolor sit amet</p>".repeat(60);
        let body = format!("<html><body>{filler}</body></html>");
        Mock::given(method("POST"))
            .and(path("/html"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body.clone()))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/lite"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(&server)
            .await;

        let provider = build_provider(
            &format!("{}/html", server.uri()),
            &format!("{}/lite", server.uri()),
        );
        let err = provider
            .search(&SearchQuery::new("rust"))
            .await
            .unwrap_err();
        assert!(matches!(err, SearchError::ParseFailed { .. }));
    }

    #[tokio::test]
    async fn rate_limited_propagates() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/html"))
            .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "3"))
            .mount(&server)
            .await;
        // Lite also rate-limits — no recovery path.
        Mock::given(method("POST"))
            .and(path("/lite"))
            .respond_with(ResponseTemplate::new(429))
            .mount(&server)
            .await;

        let provider = build_provider(
            &format!("{}/html", server.uri()),
            &format!("{}/lite", server.uri()),
        );
        let err = provider
            .search(&SearchQuery::new("rust"))
            .await
            .unwrap_err();
        assert!(matches!(err, SearchError::RateLimited { .. }));
    }

    #[tokio::test]
    async fn empty_query_rejected() {
        let provider = DuckDuckGo::new().unwrap();
        let err = provider.search(&SearchQuery::new("")).await.unwrap_err();
        assert!(matches!(err, SearchError::Config { .. }));
    }

    // ------ pure unit tests of helpers --------------------------------------

    #[test]
    fn unwrap_handles_protocol_relative_and_absolute() {
        assert_eq!(
            unwrap_ddg_url("//duckduckgo.com/l/?uddg=https%3A%2F%2Frust-lang.org%2F&rut=xyz"),
            "https://rust-lang.org/"
        );
        assert_eq!(
            unwrap_ddg_url("/l/?uddg=https%3A%2F%2Fdocs.rs%2Fasync-trait&rut=abc"),
            "https://docs.rs/async-trait"
        );
        assert_eq!(
            unwrap_ddg_url("https://html.duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com%2F"),
            "https://example.com/"
        );
        assert_eq!(
            unwrap_ddg_url("https://rust-lang.org/"),
            "https://rust-lang.org/",
            "plain URLs pass through"
        );
        assert_eq!(unwrap_ddg_url(""), "");
    }

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
    fn captcha_markers_detected() {
        assert!(looks_like_captcha("short"));
        assert!(looks_like_captcha(&format!(
            "{pad}anomaly-modal{pad}",
            pad = "x".repeat(400)
        )));
        assert!(!looks_like_captcha(
            &"<html><body><div class=\"result\">ok</div></body></html>".repeat(20)
        ));
    }

    #[test]
    fn user_agent_pool_is_populated() {
        assert!(!USER_AGENTS.is_empty());
        for ua in USER_AGENTS {
            assert!(ua.starts_with("Mozilla/5.0"));
        }
    }
}
