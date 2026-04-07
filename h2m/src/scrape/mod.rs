//! Async HTTP scraping and conversion pipeline.
//!
//! Enabled with the `scrape` [Cargo feature](https://doc.rust-lang.org/cargo/reference/features.html).
//!
//! # Examples
//!
//! ```no_run
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use h2m::scrape::Scraper;
//!
//! let scraper = Scraper::builder().concurrency(4).build()?;
//! let result = scraper.scrape("https://example.com").await?;
//! println!("{}", result.markdown);
//! # Ok(())
//! # }
//! ```

mod http;
mod pipeline;
mod types;

use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Semaphore;
pub(crate) use types::ContentExtraction;
use types::ConvertConfig;
pub use types::{Metadata, ScrapeError, ScrapeResult};

use crate::converter::Converter;
use crate::options::Options;
use crate::plugins::Gfm;
use crate::rules::CommonMark;

/// Default User-Agent header value.
const DEFAULT_USER_AGENT: &str = concat!("h2m/", env!("CARGO_PKG_VERSION"));

/// Builder for configuring a [`Scraper`].
#[derive(Debug)]
pub struct ScraperBuilder {
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

impl Default for ScraperBuilder {
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

impl ScraperBuilder {
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

    /// Builds the [`Scraper`].
    ///
    /// # Errors
    ///
    /// Returns [`ScrapeError`] if the HTTP client cannot be constructed.
    pub fn build(self) -> Result<Scraper, ScrapeError> {
        let client = reqwest::Client::builder()
            .user_agent(&self.user_agent)
            .timeout(self.timeout)
            .build()
            .map_err(|e| ScrapeError::new(format!("failed to build HTTP client: {e}"), None))?;

        let mut builder = Converter::builder()
            .options(self.options)
            .use_plugin(&CommonMark);
        if self.gfm {
            builder = builder.use_plugin(&Gfm);
        }
        if let Some(d) = &self.domain {
            builder = builder.domain(d);
        }

        let config = ConvertConfig {
            converter: builder.build(),
            extract_links: self.extract_links,
            content: self.content,
        };

        Ok(Scraper {
            client,
            config,
            concurrency: self.concurrency.max(1),
            delay: self.delay,
        })
    }
}

/// Async HTTP scraper with integrated HTML-to-Markdown conversion.
///
/// Created via [`Scraper::builder()`].
#[derive(Debug)]
pub struct Scraper {
    /// HTTP client.
    client: reqwest::Client,
    /// Pre-built conversion config.
    config: ConvertConfig,
    /// Max concurrency.
    concurrency: usize,
    /// Inter-request delay.
    delay: Duration,
}

impl Scraper {
    /// Creates a new [`ScraperBuilder`] with default settings.
    #[must_use]
    pub fn builder() -> ScraperBuilder {
        ScraperBuilder::default()
    }

    /// Scrapes a single URL and converts it to Markdown.
    ///
    /// # Errors
    ///
    /// Returns [`ScrapeError`] if the HTTP request fails or the response body
    /// cannot be decoded.
    pub async fn scrape(&self, url: &str) -> Result<ScrapeResult, ScrapeError> {
        let start = Instant::now();
        let response = http::fetch_html(&self.client, url).await?;
        Ok(pipeline::build_result(url, &response, start, &self.config))
    }

    /// Scrapes and converts multiple URLs concurrently.
    ///
    /// Results are returned in completion order (unordered). Each result is
    /// independent — a failure for one URL does not affect others.
    pub async fn scrape_many<S: AsRef<str> + Sync>(
        &self,
        urls: &[S],
    ) -> Vec<Result<ScrapeResult, ScrapeError>> {
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
                let response = http::fetch_html(&cli, &owned_url).await?;
                Ok(pipeline::build_result(
                    &owned_url, &response, start, &cfg_task,
                ))
            }));
        }

        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(Err(ScrapeError::new(format!("task panicked: {e}"), None))),
            }
        }
        results
    }

    /// Scrapes multiple URLs, calling `on_result` for each completed item.
    ///
    /// This enables streaming/NDJSON output without buffering all results.
    pub async fn scrape_many_streaming<S, F>(&self, urls: &[S], mut on_result: F)
    where
        S: AsRef<str> + Sync,
        F: FnMut(Result<ScrapeResult, ScrapeError>),
    {
        let sem = Arc::new(Semaphore::new(self.concurrency));
        let (tx, mut rx) =
            tokio::sync::mpsc::channel::<Result<ScrapeResult, ScrapeError>>(self.concurrency * 2);

        let urls_owned: Vec<String> = urls.iter().map(|s| s.as_ref().to_owned()).collect();
        let client = self.client.clone();
        let cfg = Arc::new(self.config.clone());
        let delay = self.delay;

        let producer = tokio::spawn(produce_tasks(urls_owned, client, cfg, sem, tx, delay));

        while let Some(result) = rx.recv().await {
            on_result(result);
        }

        _ = producer.await;
    }
}

async fn produce_tasks(
    urls: Vec<String>,
    client: reqwest::Client,
    cfg: Arc<ConvertConfig>,
    sem: Arc<Semaphore>,
    tx: tokio::sync::mpsc::Sender<Result<ScrapeResult, ScrapeError>>,
    delay: Duration,
) {
    for (i, url) in urls.iter().enumerate() {
        if i > 0 && !delay.is_zero() {
            tokio::time::sleep(delay).await;
        }
        let Ok(permit) = Arc::clone(&sem).acquire_owned().await else {
            break;
        };
        tokio::spawn(scrape_task(
            client.clone(),
            url.clone(),
            Arc::clone(&cfg),
            tx.clone(),
            permit,
        ));
    }
}

async fn scrape_task(
    client: reqwest::Client,
    url: String,
    cfg: Arc<ConvertConfig>,
    tx: tokio::sync::mpsc::Sender<Result<ScrapeResult, ScrapeError>>,
    _permit: tokio::sync::OwnedSemaphorePermit,
) {
    let start = Instant::now();
    let result = http::fetch_html(&client, &url)
        .await
        .map(|response| pipeline::build_result(&url, &response, start, &cfg));
    _ = tx.send(result).await;
}
