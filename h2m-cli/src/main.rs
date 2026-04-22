//! h2m — HTML to Markdown converter CLI.
//!
//! The CLI exposes two subcommands:
//!
//! - `h2m convert <INPUT>…` — convert HTML from URLs, files, or stdin
//! - `h2m search <QUERY>` — run a web search, optionally piping results
//!   through the existing scrape pipeline (requires the `search` feature)
//!
//! # Examples
//!
//! ```sh
//! # Convert a URL
//! h2m convert https://example.com
//!
//! # Convert a local file with GFM
//! h2m convert --gfm page.html
//!
//! # Pipe from curl, extract only <article>
//! curl -s https://blog.example.com/post | h2m convert --selector article
//!
//! # JSON for agent consumption
//! h2m convert --json https://example.com
//!
//! # Batch convert (NDJSON streaming)
//! h2m convert --json url1 url2 url3
//!
//! # Web search (requires H2M_SEARXNG_URL env var for the default provider)
//! h2m search "rust async trait" --limit 5
//!
//! # Search + scrape each hit to Markdown (NDJSON)
//! h2m search "rust async" --scrape --gfm --readable
//! ```

#![allow(
    clippy::print_stderr,
    clippy::shadow_reuse,
    reason = "CLI binary uses stderr for diagnostics and shadow rebinding for option building"
)]

mod cli;
mod convert;
mod error;
mod output;
#[cfg(feature = "search")]
mod search;

use std::process::ExitCode;

use clap::Parser;

use crate::cli::{Cli, Command};

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match &cli.command {
        Command::Convert(args) => convert::run(args).await,
        #[cfg(feature = "search")]
        Command::Search(args) => search::run(args).await,
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            report_error(&cli.command, &e);
            ExitCode::FAILURE
        }
    }
}

fn report_error(command: &Command, err: &error::CliError) {
    let is_json = match command {
        Command::Convert(args) => args.json,
        #[cfg(feature = "search")]
        Command::Search(args) => args.json || args.scrape,
    };
    if is_json {
        output::emit_json_error(&err.to_string(), err.url());
    } else {
        eprintln!("error: {err}");
    }
}
