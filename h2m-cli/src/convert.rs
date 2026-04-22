//! Core conversion orchestration: single, batch, and stdin modes.

use std::fs;
use std::io::{self, Read};
use std::time::Duration;

use h2m::Converter;
use h2m::scrape::Scraper;

use crate::cli::{self, ConvertArgs};
use crate::error::CliError;
use crate::output;

/// Builds a [`Scraper`] from [`ConvertArgs`].
pub(crate) fn build_scraper(args: &ConvertArgs) -> Result<Scraper, CliError> {
    let mut builder = Scraper::builder()
        .options(cli::build_options(&args.format))
        .gfm(args.format.gfm)
        .extract_links(args.extract_links)
        .concurrency(args.http.concurrency)
        .delay(Duration::from_millis(args.http.delay))
        .timeout(Duration::from_secs(args.http.timeout));

    if let Some(d) = &args.domain {
        builder = builder.domain(d);
    }
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

/// Builds a [`Converter`] for pure stdin conversion (no HTTP).
fn build_converter(args: &ConvertArgs) -> Converter {
    let mut builder = Converter::builder()
        .options(cli::build_options(&args.format))
        .use_plugin(&h2m::rules::CommonMark);

    if args.format.gfm {
        builder = builder.use_plugin(&h2m::plugins::Gfm);
    }
    if let Some(d) = &args.domain {
        builder = builder.domain(d);
    }

    builder.build()
}

/// Entry point: dispatches to single, batch, or stdin mode.
///
/// # Errors
///
/// Returns `CliError` on scrape failures, I/O errors, or file-read errors.
pub(crate) async fn run(args: &ConvertArgs) -> Result<(), CliError> {
    let inputs = collect_inputs(args)?;

    if inputs.is_empty() {
        run_stdin(args)?;
        return Ok(());
    }

    let scraper = build_scraper(args)?;

    if let [input] = inputs.as_slice() {
        let result = scraper.scrape(input).await?;
        output::emit_single(args.json, args.output.as_deref(), &result);
    } else {
        run_batch(args, &scraper, &inputs).await;
    }

    Ok(())
}

/// Collects all input sources from CLI args and `--urls` file.
fn collect_inputs(args: &ConvertArgs) -> Result<Vec<String>, CliError> {
    let mut inputs: Vec<String> = args
        .input
        .iter()
        .filter(|s| s.as_str() != "-")
        .cloned()
        .collect();

    if let Some(path) = &args.urls {
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

    if args.input.iter().any(|s| s == "-") && inputs.is_empty() {
        return Ok(Vec::new());
    }

    Ok(inputs)
}

/// Reads from stdin and converts without creating an HTTP client.
fn run_stdin(args: &ConvertArgs) -> Result<(), CliError> {
    let converter = build_converter(args);

    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;

    let html = if let Some(sel) = &args.content.selector {
        h2m::html::select(&buf, sel)
    } else if args.content.readable {
        h2m::html::readable_content(&buf)
    } else {
        buf
    };
    let md = converter.convert(&html);
    output::emit_single_markdown(args.json, args.output.as_deref(), &md);
    Ok(())
}

/// Batch-converts multiple URLs with streaming output.
async fn run_batch(args: &ConvertArgs, scraper: &Scraper, inputs: &[String]) {
    if args.json {
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
