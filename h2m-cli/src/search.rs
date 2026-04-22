//! `h2m search` subcommand: run a web search, optionally scraping each hit.
//!
//! Pure search mode emits either one NDJSON line per hit (`--json`) or a
//! pretty-printed [`SearchResponse`] (default). `--scrape` pipes the URLs
//! through the existing [`h2m::scrape::Scraper`] pipeline and streams
//! `ScrapeResult` NDJSON, matching the shape produced by
//! `h2m convert --json` in batch mode.

use std::time::Duration;

use h2m::scrape::Scraper;
use h2m_search::{SearchClient, SearchQuery, SearchResponse, SearchSource};

use crate::cli::{self, SearchArgs};
use crate::error::CliError;
use crate::output;

/// Entry point for the `search` subcommand.
///
/// # Errors
///
/// Returns `CliError::Search` on provider failures and `CliError::Scrape` on
/// downstream HTTP errors when `--scrape` is set.
pub(crate) async fn run(args: &SearchArgs) -> Result<(), CliError> {
    let client = build_client(args)?;
    let query = build_query(args);
    let response = client.search(&query).await?;

    if args.scrape {
        run_scrape(args, &response).await?;
    } else {
        emit_search_results(args, &response);
    }
    Ok(())
}

fn build_client(args: &SearchArgs) -> Result<SearchClient, CliError> {
    let mut builder = SearchClient::builder().provider(&args.provider);
    if let Some(url) = &args.searxng_url {
        builder = builder.searxng_url(url);
    }
    builder.build().map_err(CliError::from)
}

fn build_query(args: &SearchArgs) -> SearchQuery {
    let sources: Vec<SearchSource> = args.sources.iter().copied().map(Into::into).collect();
    let mut q = SearchQuery::new(&args.query)
        .with_limit(args.limit)
        .with_sources(sources)
        .with_safesearch(args.safesearch.into());
    if let Some(tr) = args.time_range {
        q = q.with_time_range(tr.into());
    }
    if let Some(lang) = &args.language {
        q = q.with_language(lang);
    }
    if let Some(country) = &args.country {
        q = q.with_country(country);
    }
    q
}

fn emit_search_results(args: &SearchArgs, response: &SearchResponse) {
    if args.json {
        // NDJSON: one line per hit for streaming consumers.
        for hit in response.all_hits() {
            output::emit_search_ndjson(hit);
        }
    } else {
        // Default: pretty-printed response object (includes metadata).
        output::emit_json_pretty(response);
    }
}

async fn run_scrape(args: &SearchArgs, response: &SearchResponse) -> Result<(), CliError> {
    let urls: Vec<String> = response.all_hits().map(|h| h.url.clone()).collect();
    if urls.is_empty() {
        return Ok(());
    }

    let scraper = build_search_scraper(args)?;
    scraper
        .scrape_many_streaming(&urls, |result| {
            output::emit_ndjson(&result);
        })
        .await;
    Ok(())
}

fn build_search_scraper(args: &SearchArgs) -> Result<Scraper, CliError> {
    let mut builder = Scraper::builder()
        .options(cli::build_options(&args.format))
        .gfm(args.format.gfm)
        .extract_links(args.extract_links)
        .concurrency(args.http.concurrency)
        .delay(Duration::from_millis(args.http.delay))
        .timeout(Duration::from_secs(args.http.timeout));

    if let Some(s) = &args.content.selector {
        builder = builder.selector(s);
    } else if args.content.readable {
        builder = builder.readable(true);
    }
    if let Some(ua) = &args.http.user_agent {
        builder = builder.user_agent(ua);
    }

    Ok(builder.build()?)
}
