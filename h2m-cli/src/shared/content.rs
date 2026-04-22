//! Content-extraction arguments shared by `convert` and `search --scrape`.

use clap::Args;

/// Selector / readable-mode toggles.
#[derive(Args, Debug, Clone)]
pub(crate) struct ContentArgs {
    /// CSS selector to extract before conversion. Mutually exclusive with
    /// `--readable`.
    #[arg(short, long, conflicts_with = "readable")]
    pub selector: Option<String>,

    /// Smart readable extraction (semantic selectors → noise stripping).
    /// Mutually exclusive with `--selector`.
    #[arg(short = 'r', long, conflicts_with = "selector")]
    pub readable: bool,
}
