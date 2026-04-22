//! Top-level CLI definition.
//!
//! Only the root [`Cli`] parser and the [`Command`] enum live here — each
//! subcommand's argument struct and execution logic are colocated under
//! [`crate::commands`].

use clap::{Parser, Subcommand};

use crate::commands::ConvertArgs;
#[cfg(feature = "search")]
use crate::commands::SearchArgs;

/// HTML-to-Markdown converter with optional web search.
///
/// Use `h2m <COMMAND> --help` for per-subcommand details.
#[derive(Parser, Debug)]
#[command(
    name = "h2m",
    version,
    about,
    long_about = None,
    propagate_version = true,
    arg_required_else_help = true,
)]
pub(crate) struct Cli {
    /// Subcommand to execute.
    #[command(subcommand)]
    pub command: Command,
}

/// Top-level subcommand.
#[derive(Subcommand, Debug)]
pub(crate) enum Command {
    /// Convert HTML to Markdown from URLs, files, or stdin.
    Convert(ConvertArgs),

    /// Search the web and optionally scrape each result to Markdown.
    #[cfg(feature = "search")]
    Search(SearchArgs),
}
