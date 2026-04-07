//! h2m — HTML to Markdown converter CLI.
//!
//! Supports URLs, files, and stdin as input sources.
//!
//! # Examples
//!
//! ```sh
//! # Convert a URL directly
//! h2m https://example.com
//!
//! # Convert a local file with GFM extensions
//! h2m --gfm page.html
//!
//! # Pipe from curl, extract only <article>
//! curl -s https://blog.example.com/post | h2m --selector article
//!
//! # JSON output for agent consumption
//! h2m --json https://example.com
//!
//! # Batch convert multiple URLs (NDJSON output)
//! h2m --json url1 url2 url3
//!
//! # Save output to a file
//! h2m https://example.com -o output.md
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

use std::process::ExitCode;

use clap::Parser;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = cli::Cli::parse();
    match convert::run(&cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            if cli.json {
                output::emit_json_error(&e.to_string(), e.url());
            } else {
                eprintln!("error: {e}");
            }
            ExitCode::FAILURE
        }
    }
}
