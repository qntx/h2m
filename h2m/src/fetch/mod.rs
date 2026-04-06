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

mod client;
mod pipeline;
mod types;

use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Semaphore;
pub(crate) use types::{ContentExtraction, ConvertConfig, ResponseMeta};
#[allow(clippy::module_name_repetitions)]
pub use types::{FetchError, FetchResult};

use crate::converter::Converter;
use crate::options::Options;
use crate::plugins::Gfm;
use crate::rules::CommonMark;

/// Default User-Agent header value.
const DEFAULT_USER_AGENT: &str = concat!("h2m/", env!("CARGO_PKG_VERSION"));

/// Builder for configuring a [`Fetcher`].
#[derive(Debug)]
pub struct FetcherBuilder {
    /// Converter options.
    options: Options,
    /// Enable GFM extensions.
    gfm: bool,
    /// Base domain for resolving relative URLs.
    domain: Option<String>,
    /// Content extraction strategy.
    content: ContentExtraction,
    /// Extract links from pages.
    extract_links: bool,
    /// Max concurrent requests.
    concurrency: usize,
    /// Delay between requests.
    delay: Duration,
    /// Request timeout.
    timeout: Duration,
    /// HTTP User-Agent header.
    user_agent: String,
}

impl Default for FetcherBuilder {
    fn default() -> Self {
        Self {
            options: Options::default(),
            gfm: false,
            domain: None,
            content: ContentExtraction::default(),
            extract_links: false,
            concurrency: 4,
            delay: Duration::ZERO,
            timeout: Duration::from_secs(30),
            user_agent: DEFAULT_USER_AGENT.to_owned(),
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

    /// Sets an explicit CSS selector to extract before converting.
    ///
    /// Mutually exclusive with [`readable`](Self::readable).
    #[must_use]
    pub fn selector(mut self, selector: impl Into<String>) -> Self {
        self.content = ContentExtraction::Selector(selector.into());
        self
    }

    /// Enables smart readable content extraction.
    ///
    /// Phase 1: tries semantic selectors (`article`, `main`, `[role="main"]`, …).
    /// Phase 2: strips noise elements (`nav`, `footer`, `aside`, …) if no
    /// semantic wrapper is found.
    /// Mutually exclusive with [`selector`](Self::selector).
    #[must_use]
    pub fn readable(mut self, enable: bool) -> Self {
        self.content = if enable {
            ContentExtraction::Readable
        } else {
            ContentExtraction::Full
        };
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

    /// Sets the HTTP `User-Agent` header.
    #[must_use]
    pub fn user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = ua.into();
        self
    }

    /// Builds the [`Fetcher`].
    ///
    /// # Errors
    ///
    /// Returns `FetchError` if the HTTP client cannot be constructed.
    pub fn build(self) -> Result<Fetcher, FetchError> {
        let client = reqwest::Client::builder()
            .user_agent(&self.user_agent)
            .timeout(self.timeout)
            .build()
            .map_err(|e| FetchError {
                error: format!("failed to build HTTP client: {e}"),
                url: None,
            })?;

        let mut builder = Converter::builder()
            .options(self.options)
            .use_plugin(CommonMark);
        if self.gfm {
            builder = builder.use_plugin(Gfm);
        }
        if let Some(d) = &self.domain {
            builder = builder.domain(d);
        }

        let config = ConvertConfig {
            converter: builder.build(),
            extract_links: self.extract_links,
            domain: self.domain,
            content: self.content,
        };

        Ok(Fetcher {
            client,
            config,
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
    /// Pre-built conversion config (contains cached `Converter`).
    config: ConvertConfig,
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
        let (raw_html, meta) = client::fetch_html_inner(&self.client, url).await?;
        Ok(pipeline::convert_to_result(
            Some(url),
            &raw_html,
            start,
            &self.config,
            &meta,
        ))
    }

    /// Fetches and converts multiple URLs concurrently.
    ///
    /// Results are returned as they complete (unordered). Each result is
    /// independent — a failure for one URL does not affect others.
    pub async fn fetch_many<S: AsRef<str> + Sync>(
        &self,
        urls: &[S],
    ) -> Vec<Result<FetchResult, FetchError>> {
        let sem = Arc::new(Semaphore::new(self.concurrency));
        let cfg = Arc::new(self.config.clone());
        let mut handles = Vec::with_capacity(urls.len());

        for (i, url) in urls.iter().enumerate() {
            if i > 0 && !self.delay.is_zero() {
                tokio::time::sleep(self.delay).await;
            }

            let Ok(permit) = Arc::clone(&sem).acquire_owned().await else {
                break;
            };
            let owned_url = url.as_ref().to_owned();
            let cli = self.client.clone();
            let cfg_task = Arc::clone(&cfg);

            handles.push(tokio::spawn(async move {
                let _permit = permit;
                let start = Instant::now();

                let (raw_html, meta) = client::fetch_html_inner(&cli, &owned_url).await?;
                Ok(pipeline::convert_to_result(
                    Some(&owned_url),
                    &raw_html,
                    start,
                    &cfg_task,
                    &meta,
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
    pub async fn fetch_many_streaming<S, F>(&self, urls: &[S], mut on_result: F)
    where
        S: AsRef<str> + Sync,
        F: FnMut(Result<FetchResult, FetchError>) + Send,
    {
        let sem = Arc::new(Semaphore::new(self.concurrency));
        let (tx, mut rx) =
            tokio::sync::mpsc::channel::<Result<FetchResult, FetchError>>(self.concurrency * 2);

        let urls_owned: Vec<String> = urls.iter().map(|s| s.as_ref().to_owned()).collect();
        let client = self.client.clone();
        let cfg = Arc::new(self.config.clone());
        let delay = self.delay;

        let producer = tokio::spawn(async move {
            for (i, url) in urls_owned.iter().enumerate() {
                if i > 0 && !delay.is_zero() {
                    tokio::time::sleep(delay).await;
                }

                let Ok(permit) = Arc::clone(&sem).acquire_owned().await else {
                    break;
                };
                let tx_c = tx.clone();
                let owned_url = url.clone();
                let cli = client.clone();
                let cfg_task = Arc::clone(&cfg);

                tokio::spawn(async move {
                    let _permit = permit;
                    let start = Instant::now();

                    let result =
                        client::fetch_html_inner(&cli, &owned_url)
                            .await
                            .map(|(raw_html, meta)| {
                                pipeline::convert_to_result(
                                    Some(&owned_url),
                                    &raw_html,
                                    start,
                                    &cfg_task,
                                    &meta,
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
        pipeline::convert_to_result(
            None,
            raw_html,
            start,
            &self.config,
            &ResponseMeta::default(),
        )
    }
}
