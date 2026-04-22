//! `h2m search` — web search with optional scrape-to-Markdown pipeline.

use std::path::PathBuf;
use std::time::Duration;

use clap::{Args, ValueEnum};
use h2m::scrape::Scraper;
use h2m_search::{SearchClient, SearchQuery, SearchResponse, SearchSource};

use crate::error::CliError;
use crate::output;
use crate::shared::{ContentArgs, FormatArgs, HttpArgs, build_options};

/// Arguments for `h2m search`.
#[derive(Args, Debug)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "CLI flags are naturally boolean"
)]
pub(crate) struct SearchArgs {
    /// The search query.
    pub query: String,

    /// Search provider (`searxng` is the default).
    #[arg(short = 'p', long, default_value = "searxng")]
    pub provider: String,

    /// Maximum number of results per source (1..=100).
    #[arg(long, default_value_t = 10)]
    pub limit: usize,

    /// Result sources to request (comma-separated).
    #[arg(long, value_enum, value_delimiter = ',', default_value = "web")]
    pub sources: Vec<SourceArg>,

    /// Time-range filter.
    #[arg(long, value_enum)]
    pub time_range: Option<TimeRangeArg>,

    /// ISO 3166-1 alpha-2 country code (e.g. `us`, `cn`).
    #[arg(long)]
    pub country: Option<String>,

    /// ISO 639-1 language code (e.g. `en`, `zh`).
    #[arg(long)]
    pub language: Option<String>,

    /// Safe-search filter level.
    #[arg(long, value_enum, default_value_t = SafeSearchArg::Moderate)]
    pub safesearch: SafeSearchArg,

    /// `SearXNG` base URL (overrides `H2M_SEARXNG_URL`).
    #[arg(long)]
    pub searxng_url: Option<String>,

    /// After search, scrape each hit and emit a `ScrapeResult` per line (NDJSON).
    #[arg(long)]
    pub scrape: bool,

    /// JSON output (NDJSON for hits in pure-search mode).
    #[arg(long)]
    pub json: bool,

    /// Output file path (stdout if omitted).
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Extract links from each scraped page (when `--scrape` is set).
    #[arg(long)]
    pub extract_links: bool,

    #[command(flatten)]
    pub content: ContentArgs,

    #[command(flatten)]
    pub format: FormatArgs,

    #[command(flatten)]
    pub http: HttpArgs,
}

impl SearchArgs {
    /// Runs the `search` subcommand.
    ///
    /// # Errors
    ///
    /// Returns [`CliError::Search`] on provider failures and
    /// [`CliError::Scrape`] on downstream HTTP errors when `--scrape` is set.
    pub(crate) async fn run(&self) -> Result<(), CliError> {
        let client = self.build_client()?;
        let response = client.search(&self.build_query()).await?;

        if self.scrape {
            self.run_scrape(&response).await?;
        } else {
            self.emit(&response);
        }
        Ok(())
    }

    fn build_client(&self) -> Result<SearchClient, CliError> {
        let mut builder = SearchClient::builder().provider(&self.provider);
        if let Some(url) = &self.searxng_url {
            builder = builder.searxng_url(url);
        }
        builder.build().map_err(CliError::from)
    }

    fn build_query(&self) -> SearchQuery {
        let sources: Vec<SearchSource> = self.sources.iter().copied().map(Into::into).collect();
        let mut q = SearchQuery::new(&self.query)
            .with_limit(self.limit)
            .with_sources(sources)
            .with_safesearch(self.safesearch.into());
        if let Some(tr) = self.time_range {
            q = q.with_time_range(tr.into());
        }
        if let Some(lang) = &self.language {
            q = q.with_language(lang);
        }
        if let Some(country) = &self.country {
            q = q.with_country(country);
        }
        q
    }

    fn emit(&self, response: &SearchResponse) {
        if self.json {
            for hit in response.all_hits() {
                output::emit_search_ndjson(hit);
            }
        } else {
            output::emit_json_pretty(response);
        }
    }

    async fn run_scrape(&self, response: &SearchResponse) -> Result<(), CliError> {
        let urls: Vec<String> = response.all_hits().map(|h| h.url.clone()).collect();
        if urls.is_empty() {
            return Ok(());
        }

        let scraper = self.build_scraper()?;
        scraper
            .scrape_many_streaming(&urls, |result| {
                output::emit_ndjson(&result);
            })
            .await;
        Ok(())
    }

    fn build_scraper(&self) -> Result<Scraper, CliError> {
        let mut builder = Scraper::builder()
            .options(build_options(&self.format))
            .gfm(self.format.gfm)
            .extract_links(self.extract_links)
            .concurrency(self.http.concurrency)
            .delay(Duration::from_millis(self.http.delay))
            .timeout(Duration::from_secs(self.http.timeout));

        if let Some(s) = &self.content.selector {
            builder = builder.selector(s);
        } else if self.content.readable {
            builder = builder.readable(true);
        }
        if let Some(ua) = &self.http.user_agent {
            builder = builder.user_agent(ua);
        }

        Ok(builder.build()?)
    }
}

/// Search source category (CLI-side value-enum).
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SourceArg {
    /// General web results.
    Web,
    /// News articles.
    News,
    /// Image results.
    Images,
}

impl From<SourceArg> for SearchSource {
    fn from(s: SourceArg) -> Self {
        match s {
            SourceArg::Web => Self::Web,
            SourceArg::News => Self::News,
            SourceArg::Images => Self::Images,
        }
    }
}

/// Time-range filter for search (CLI-side value-enum).
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TimeRangeArg {
    /// Past 24 hours.
    Day,
    /// Past 7 days.
    Week,
    /// Past 30 days.
    Month,
    /// Past 12 months.
    Year,
}

impl From<TimeRangeArg> for h2m_search::TimeRange {
    fn from(t: TimeRangeArg) -> Self {
        match t {
            TimeRangeArg::Day => Self::Day,
            TimeRangeArg::Week => Self::Week,
            TimeRangeArg::Month => Self::Month,
            TimeRangeArg::Year => Self::Year,
        }
    }
}

/// Safe-search level (CLI-side value-enum).
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SafeSearchArg {
    /// No filtering.
    Off,
    /// Moderate filtering (default).
    Moderate,
    /// Strict filtering.
    Strict,
}

impl From<SafeSearchArg> for h2m_search::SafeSearch {
    fn from(s: SafeSearchArg) -> Self {
        match s {
            SafeSearchArg::Off => Self::Off,
            SafeSearchArg::Moderate => Self::Moderate,
            SafeSearchArg::Strict => Self::Strict,
        }
    }
}
