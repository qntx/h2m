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
//! # Logging
//!
//! The CLI honours `RUST_LOG` (e.g. `RUST_LOG=h2m=debug,h2m_search=debug`)
//! and writes structured logs to stderr. By default it is silent.
//!
//! # Graceful shutdown
//!
//! Both subcommands race their main work against `SIGINT` (Ctrl-C). A second
//! signal terminates immediately; the first signal returns a
//! [`CliError::Interrupted`] so in-flight output can be flushed cleanly.
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
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;
#[cfg(test)]
use {insta as _, pretty_assertions as _, wiremock as _};

use crate::cli::Cli;
use crate::error::CliError;

#[tokio::main]
async fn main() -> ExitCode {
    init_tracing();
    let cli = Cli::parse();
    match run_with_shutdown(&cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            cli.command.report_error(&e);
            ExitCode::FAILURE
        }
    }
}

/// Installs a stderr subscriber that honours the `RUST_LOG` env var.
///
/// Missing / invalid filters default to `off` so non-interactive pipelines
/// stay quiet unless logging is explicitly enabled.
fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("off"));
    fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_target(false)
        .compact()
        .try_init()
        .ok();
}

/// Races the subcommand against Ctrl-C.
async fn run_with_shutdown(cli: &Cli) -> Result<(), CliError> {
    tokio::select! {
        result = cli.command.run() => result,
        _ = tokio::signal::ctrl_c() => {
            tracing::warn!("received interrupt signal, shutting down");
            Err(CliError::Interrupted)
        }
    }
}
