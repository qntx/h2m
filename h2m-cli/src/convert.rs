//! Core conversion orchestration: single, batch, and stdin modes.

use std::fs;
use std::io::{self, Read};
use std::process;
use std::time::Duration;

use h2m::fetch::{FetchError, FetchResult, Fetcher};

use crate::cli::{self, Cli};
use crate::output;

/// Builds a [`Fetcher`] from CLI arguments.
fn build_fetcher(cli: &Cli) -> Fetcher {
    let mut builder = Fetcher::builder()
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
    }

    builder.build().unwrap_or_else(|e| {
        if cli.json {
            output::emit_json_error(&e.error, None);
        } else {
            eprintln!("error: {e}");
        }
        process::exit(1);
    })
}

/// Main entry point: dispatches to single, batch, or stdin mode.
pub async fn run(cli: &Cli) {
    let inputs = collect_inputs(cli);

    if inputs.is_empty() {
        run_stdin(cli);
        return;
    }

    let fetcher = build_fetcher(cli);

    if inputs.len() == 1 {
        let result = fetcher.fetch(&inputs[0]).await;
        output::emit_single(cli, &result);
    } else {
        run_batch(cli, &fetcher, &inputs).await;
    }
}

/// Collects all input sources from CLI args and `--urls` file.
fn collect_inputs(cli: &Cli) -> Vec<String> {
    let mut inputs: Vec<String> = cli
        .input
        .iter()
        .filter(|s| s.as_str() != "-")
        .cloned()
        .collect();

    if let Some(path) = &cli.urls {
        let content = fs::read_to_string(path).unwrap_or_else(|e| {
            let msg = format!("cannot read URL file {}: {e}", path.display());
            if cli.json {
                output::emit_json_error(&msg, None);
            } else {
                eprintln!("error: {msg}");
            }
            process::exit(1);
        });
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                inputs.push(trimmed.to_owned());
            }
        }
    }

    if cli.input.iter().any(|s| s == "-") && inputs.is_empty() {
        return Vec::new();
    }

    inputs
}

/// Reads from stdin and converts.
fn run_stdin(cli: &Cli) {
    let fetcher = build_fetcher(cli);

    let mut buf = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut buf) {
        let err = FetchError::new(format!("cannot read stdin: {e}"), None);
        output::emit_single(cli, &Err(err));
        return;
    }

    let result: Result<FetchResult, FetchError> = Ok(fetcher.convert_html(&buf));
    output::emit_single(cli, &result);
}

/// Batch-converts multiple URLs with streaming output.
async fn run_batch(cli: &Cli, fetcher: &Fetcher, inputs: &[String]) {
    if cli.json {
        fetcher
            .fetch_many_streaming(inputs, |result| {
                output::emit_ndjson(&result);
            })
            .await;
    } else {
        fetcher
            .fetch_many_streaming(inputs, |result| {
                output::emit_batch_plain(&result);
            })
            .await;
    }
}
