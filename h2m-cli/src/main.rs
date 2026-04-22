//! `h2m` — HTML-to-Markdown converter CLI with optional web search.
//!
//! The CLI is a subcommand tree:
//!
//! - `h2m convert <INPUT>…` — convert HTML from URLs, files, or stdin
//! - `h2m search <QUERY>` — run a web search, optionally scrape each hit
//!
//! See [`cli`] for the root parser, [`commands`] for subcommand modules,
//! and [`shared`] for argument groups shared between subcommands.
//!
//! # Examples
//!
//! ```sh
//! h2m convert https://example.com
//! h2m convert --gfm page.html
//! curl -s https://blog.example.com/post | h2m convert --selector article
//! h2m convert --json url1 url2 url3
//! h2m search "rust async trait" --limit 5
//! h2m search "rust async" --scrape --gfm --readable
//! ```

#![allow(
    clippy::print_stderr,
    reason = "CLI binary uses stderr for diagnostics"
)]

mod cli;
mod commands;
mod error;
mod output;
mod shared;

use std::process::ExitCode;

use clap::Parser;

use crate::cli::Cli;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command.run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            cli.command.report_error(&e);
            ExitCode::FAILURE
        }
    }
}
