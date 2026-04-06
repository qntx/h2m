//! Async HTTP fetching and batch conversion pipeline.
//!
//! Enabled with the `fetch` [Cargo feature](https://doc.rust-lang.org/cargo/reference/features.html).
//!
//! # Examples
//!
//! ```no_run
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use h2m::fetch::Fetcher;
//!
//! let fetcher = Fetcher::builder().concurrency(4).build()?;
//! let result = fetcher.fetch("https://example.com").await?;
//! println!("{}", result.markdown);
//! # Ok(())
//! # }
//! ```

use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Serialize;
use tokio::sync::Semaphore;

use crate::converter::{Converter, ConverterBuilder};
use crate::html;
use crate::options::Options;
use crate::plugins::Gfm;
use crate::rules::CommonMark;

/// Bundled conversion parameters passed to spawned tasks.
#[derive(Debug, Clone, Copy)]
struct ConvertConfig {
    /// Converter options.
    options: Options,
    /// Enable GFM.
    gfm: bool,
    /// Extract links.
    extract_links: bool,
}

/// Successful conversion result with metadata.
#[derive(Debug, Serialize)]
#[non_exhaustive]
#[allow(clippy::module_name_repetitions)]
pub struct FetchResult {
    /// Source URL (if input was a URL).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Resolved domain name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    /// Page `<title>` text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Converted Markdown content.
    pub markdown: String,
    /// Extracted links (when enabled).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<String>>,
    /// Total elapsed time in milliseconds.
    pub elapsed_ms: u64,
    /// Original HTML byte length.
    pub content_length: usize,
}

/// Error returned by fetch operations.
#[derive(Debug, Serialize)]
#[non_exhaustive]
#[allow(clippy::module_name_repetitions)]
pub struct FetchError {
    /// Error message.
    pub error: String,
    /// URL that caused the error, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl FetchError {
    /// Creates a new `FetchError` with an error message and optional URL.
    #[must_use]
    pub fn new(error: impl Into<String>, url: Option<String>) -> Self {
        Self {
            error: error.into(),
            url,
        }
    }
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl std::error::Error for FetchError {}

/// Builder for configuring a [`Fetcher`].
#[derive(Debug)]
pub struct FetcherBuilder {
    /// Converter options.
    options: Options,
    /// Enable GFM extensions.
    gfm: bool,
    /// Base domain for resolving relative URLs.
    domain: Option<String>,
    /// CSS selector to extract before converting.
    selector: Option<String>,
    /// Extract links from pages.
    extract_links: bool,
    /// Max concurrent requests.
    concurrency: usize,
    /// Delay between requests.
    delay: Duration,
    /// Request timeout.
    timeout: Duration,
}

impl Default for FetcherBuilder {
    fn default() -> Self {
        Self {
            options: Options::default(),
            gfm: false,
            domain: None,
            selector: None,
            extract_links: false,
            concurrency: 4,
            delay: Duration::ZERO,
            timeout: Duration::from_secs(30),
        }
    }
}

impl FetcherBuilder {
    /// Sets the converter options.
    #[must_use]
    pub const fn options(mut self, options: Options) -> Self {
        self.options = options;
        self
    }

    /// Enables GFM extensions (tables, strikethrough, task lists).
    #[must_use]
    pub const fn gfm(mut self, enable: bool) -> Self {
        self.gfm = enable;
        self
    }

    /// Sets the base domain for resolving relative URLs.
    #[must_use]
    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    /// Sets a CSS selector to extract before converting.
    #[must_use]
    pub fn selector(mut self, selector: impl Into<String>) -> Self {
        self.selector = Some(selector.into());
        self
    }

    /// Enables link extraction in results.
    #[must_use]
    pub const fn extract_links(mut self, enable: bool) -> Self {
        self.extract_links = enable;
        self
    }

    /// Sets the maximum number of concurrent HTTP requests.
    #[must_use]
    pub const fn concurrency(mut self, n: usize) -> Self {
        self.concurrency = n;
        self
    }

    /// Sets the delay between starting each request.
    #[must_use]
    pub const fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Sets the HTTP request timeout.
    #[must_use]
    pub const fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Builds the [`Fetcher`].
    ///
    /// # Errors
    ///
    /// Returns `FetchError` if the HTTP client cannot be constructed.
    pub fn build(self) -> Result<Fetcher, FetchError> {
        let client = reqwest::Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(|e| FetchError {
                error: format!("failed to build HTTP client: {e}"),
                url: None,
            })?;

        Ok(Fetcher {
            client,
            options: self.options,
            gfm: self.gfm,
            domain: self.domain,
            selector: self.selector,
            extract_links: self.extract_links,
            concurrency: self.concurrency.max(1),
            delay: self.delay,
        })
    }
}

/// Async HTTP fetcher with integrated HTML-to-Markdown conversion.
///
/// Created via [`Fetcher::builder()`].
#[derive(Debug)]
pub struct Fetcher {
    /// HTTP client.
    client: reqwest::Client,
    /// Converter options.
    options: Options,
    /// Enable GFM.
    gfm: bool,
    /// Base domain override.
    domain: Option<String>,
    /// CSS selector.
    selector: Option<String>,
    /// Extract links.
    extract_links: bool,
    /// Max concurrency.
    concurrency: usize,
    /// Inter-request delay.
    delay: Duration,
}

impl Fetcher {
    /// Creates a new [`FetcherBuilder`] with default settings.
    #[must_use]
    pub fn builder() -> FetcherBuilder {
        FetcherBuilder::default()
    }

    /// Fetches a single URL and converts it to Markdown.
    ///
    /// # Errors
    ///
    /// Returns `FetchError` if the HTTP request fails or the response body
    /// cannot be decoded.
    pub async fn fetch(&self, url: &str) -> Result<FetchResult, FetchError> {
        let start = Instant::now();
        let raw_html = self.fetch_html(url).await?;
        Ok(self.convert_fetched(url, &raw_html, start))
    }

    /// Fetches and converts multiple URLs concurrently.
    ///
    /// Results are returned as they complete (unordered). Each result is
    /// independent — a failure for one URL does not affect others.
    pub async fn fetch_many(&self, urls: &[String]) -> Vec<Result<FetchResult, FetchError>> {
        let sem = Arc::new(Semaphore::new(self.concurrency));
        let mut handles = Vec::with_capacity(urls.len());

        for (i, url) in urls.iter().enumerate() {
            if i > 0 && !self.delay.is_zero() {
                tokio::time::sleep(self.delay).await;
            }

            let permit = Arc::clone(&sem).acquire_owned().await;
            let owned_url = url.clone();
            let cli = self.client.clone();
            let cfg = ConvertConfig {
                options: self.options,
                gfm: self.gfm,
                extract_links: self.extract_links,
            };
            let dom = self.domain.clone();
            let sel = self.selector.clone();

            handles.push(tokio::spawn(async move {
                let _permit = permit;
                let start = Instant::now();

                let raw_html = fetch_html_inner(&cli, &owned_url).await?;
                Ok(convert_fetched_inner(
                    &owned_url,
                    &raw_html,
                    start,
                    &cfg,
                    dom.as_deref(),
                    sel.as_deref(),
                ))
            }));
        }

        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(Err(FetchError {
                    error: format!("task panicked: {e}"),
                    url: None,
                })),
            }
        }
        results
    }

    /// Fetches and converts multiple URLs, calling `on_result` for each
    /// completed item. This enables streaming/NDJSON output.
    pub async fn fetch_many_streaming<F>(&self, urls: &[String], mut on_result: F)
    where
        F: FnMut(Result<FetchResult, FetchError>),
    {
        let sem = Arc::new(Semaphore::new(self.concurrency));
        let (tx, mut rx) =
            tokio::sync::mpsc::channel::<Result<FetchResult, FetchError>>(self.concurrency * 2);

        let urls_owned: Vec<String> = urls.to_vec();
        let client = self.client.clone();
        let cfg = ConvertConfig {
            options: self.options,
            gfm: self.gfm,
            extract_links: self.extract_links,
        };
        let domain = self.domain.clone();
        let selector = self.selector.clone();
        let delay = self.delay;

        let producer = tokio::spawn(async move {
            for (i, url) in urls_owned.iter().enumerate() {
                if i > 0 && !delay.is_zero() {
                    tokio::time::sleep(delay).await;
                }

                let permit = Arc::clone(&sem).acquire_owned().await;
                let tx_c = tx.clone();
                let owned_url = url.clone();
                let cli = client.clone();
                let dom = domain.clone();
                let sel = selector.clone();

                tokio::spawn(async move {
                    let _permit = permit;
                    let start = Instant::now();

                    let result = fetch_html_inner(&cli, &owned_url).await.map(|raw_html| {
                        convert_fetched_inner(
                            &owned_url,
                            &raw_html,
                            start,
                            &cfg,
                            dom.as_deref(),
                            sel.as_deref(),
                        )
                    });

                    let _ = tx_c.send(result).await;
                });
            }
        });

        while let Some(result) = rx.recv().await {
            on_result(result);
        }

        let _ = producer.await;
    }

    /// Converts already-fetched HTML into a `FetchResult`.
    ///
    /// Useful when you have HTML from a non-HTTP source (file, stdin).
    #[must_use]
    pub fn convert_html(&self, raw_html: &str) -> FetchResult {
        let start = Instant::now();
        let content_length = raw_html.len();

        let html_to_convert = self
            .selector
            .as_deref()
            .map_or_else(|| raw_html.to_owned(), |sel| html::select(raw_html, sel));

        let title = html::extract_title(raw_html);
        let links = if self.extract_links {
            Some(html::extract_links(raw_html))
        } else {
            None
        };

        let md = convert_raw(
            &self.options,
            self.gfm,
            &html_to_convert,
            self.domain.as_deref(),
        );

        FetchResult {
            url: None,
            domain: self.domain.clone(),
            title,
            markdown: md,
            links,
            elapsed_ms: elapsed_ms(start),
            content_length,
        }
    }

    /// Fetches raw HTML from a URL.
    async fn fetch_html(&self, url: &str) -> Result<String, FetchError> {
        fetch_html_inner(&self.client, url).await
    }

    /// Converts fetched HTML to a `FetchResult` with URL metadata.
    fn convert_fetched(&self, url: &str, raw_html: &str, start: Instant) -> FetchResult {
        let cfg = ConvertConfig {
            options: self.options,
            gfm: self.gfm,
            extract_links: self.extract_links,
        };
        convert_fetched_inner(
            url,
            raw_html,
            start,
            &cfg,
            self.domain.as_deref(),
            self.selector.as_deref(),
        )
    }
}

/// Fetches HTML from a URL using the given client.
async fn fetch_html_inner(client: &reqwest::Client, url: &str) -> Result<String, FetchError> {
    let resp = client.get(url).send().await.map_err(|e| FetchError {
        error: format!("failed to fetch {url}: {e}"),
        url: Some(url.to_owned()),
    })?;

    resp.text().await.map_err(|e| FetchError {
        error: format!("failed to read response body: {e}"),
        url: Some(url.to_owned()),
    })
}

/// Pure conversion from fetched HTML to `FetchResult`.
fn convert_fetched_inner(
    url: &str,
    raw_html: &str,
    start: Instant,
    cfg: &ConvertConfig,
    domain_override: Option<&str>,
    selector: Option<&str>,
) -> FetchResult {
    let content_length = raw_html.len();

    let html_to_convert =
        selector.map_or_else(|| raw_html.to_owned(), |sel| html::select(raw_html, sel));

    let title = html::extract_title(raw_html);
    let links = if cfg.extract_links {
        Some(html::extract_links(raw_html))
    } else {
        None
    };

    let parsed = url::Url::parse(url).ok();
    let auto_domain = parsed
        .as_ref()
        .and_then(|u| u.host_str())
        .map(str::to_owned);

    let domain = domain_override.or(auto_domain.as_deref());
    let md = convert_raw(&cfg.options, cfg.gfm, &html_to_convert, domain);

    FetchResult {
        url: Some(url.to_owned()),
        domain: domain.map(str::to_owned),
        title,
        markdown: md,
        links,
        elapsed_ms: elapsed_ms(start),
        content_length,
    }
}

/// Builds a converter and runs the conversion.
fn convert_raw(options: &Options, gfm: bool, html: &str, domain: Option<&str>) -> String {
    let mut builder: ConverterBuilder = Converter::builder()
        .options(*options)
        .use_plugin(CommonMark);

    if gfm {
        builder = builder.use_plugin(Gfm);
    }

    if let Some(d) = domain {
        builder = builder.domain(d);
    }

    builder.build().convert(html)
}

/// Returns elapsed milliseconds since `start`.
#[allow(clippy::cast_possible_truncation)]
fn elapsed_ms(start: Instant) -> u64 {
    start.elapsed().as_millis() as u64
}
