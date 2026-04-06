//! Core conversion orchestration: single, batch, and stdin modes.

use std::fs;
use std::io::{self, Read};
use std::time::Duration;

use h2m::Converter;
use h2m::scrape::Scraper;

use crate::cli::{self, Cli};
use crate::error::CliError;
use crate::output;

/// Builds a [`Scraper`] from CLI arguments.
fn build_scraper(cli: &Cli) -> Result<Scraper, CliError> {
    let mut builder = Scraper::builder()
        .options(cli::build_options(cli))
        .gfm(cli.gfm)
        .extract_links(cli.extract_links)
        .concurrency(cli.concurrency)
        .delay(Duration::from_millis(cli.delay))
        .timeout(Duration::from_secs(cli.timeout));

    if let Some(d) = &cli.domain {
        builder = builder.domain(d);
    }
    if let Some(s) = &cli.selector {
        builder = builder.selector(s);
    } else if cli.readable {
        builder = builder.readable(true);
    }
    if let Some(ua) = &cli.user_agent {
        builder = builder.user_agent(ua);
    }

    Ok(builder.build()?)
}

/// Builds a [`Converter`] from CLI arguments (no HTTP client needed).
fn build_converter(cli: &Cli) -> Converter {
    let mut builder = Converter::builder()
        .options(cli::build_options(cli))
        .use_plugin(h2m::rules::CommonMark);

    if cli.gfm {
        builder = builder.use_plugin(h2m::plugins::Gfm);
    }
    if let Some(d) = &cli.domain {
        builder = builder.domain(d);
    }

    builder.build()
}

/// Main entry point: dispatches to single, batch, or stdin mode.
///
/// # Errors
///
/// Returns `CliError` on scrape failures, I/O errors, or file read errors.
pub async fn run(cli: &Cli) -> Result<(), CliError> {
    let inputs = collect_inputs(cli)?;

    if inputs.is_empty() {
        run_stdin(cli)?;
        return Ok(());
    }

    let scraper = build_scraper(cli)?;

    if inputs.len() == 1 {
        let result = scraper.scrape(&inputs[0]).await?;
        output::emit_single(cli, &result);
    } else {
        run_batch(cli, &scraper, &inputs).await;
    }

    Ok(())
}

/// Collects all input sources from CLI args and `--urls` file.
fn collect_inputs(cli: &Cli) -> Result<Vec<String>, CliError> {
    let mut inputs: Vec<String> = cli
        .input
        .iter()
        .filter(|s| s.as_str() != "-")
        .cloned()
        .collect();

    if let Some(path) = &cli.urls {
        let content = fs::read_to_string(path).map_err(|e| CliError::Other {
            message: format!("cannot read URL file {}: {e}", path.display()),
            url: None,
        })?;
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                inputs.push(trimmed.to_owned());
            }
        }
    }

    if cli.input.iter().any(|s| s == "-") && inputs.is_empty() {
        return Ok(Vec::new());
    }

    Ok(inputs)
}

/// Reads from stdin and converts without creating an HTTP client.
fn run_stdin(cli: &Cli) -> Result<(), CliError> {
    let converter = build_converter(cli);

    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;

    let html = if let Some(sel) = &cli.selector {
        h2m::html::select(&buf, sel)
    } else if cli.readable {
        h2m::html::readable_content(&buf)
    } else {
        buf
    };
    let md = converter.convert(&html);
    output::emit_single_markdown(cli, &md);
    Ok(())
}

/// Batch-converts multiple URLs with streaming output.
async fn run_batch(cli: &Cli, scraper: &Scraper, inputs: &[String]) {
    if cli.json {
        scraper
            .scrape_many_streaming(inputs, |result| {
                output::emit_ndjson(&result);
            })
            .await;
    } else {
        scraper
            .scrape_many_streaming(inputs, |result| {
                output::emit_batch_plain(&result);
            })
            .await;
    }
}
