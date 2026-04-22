//! HTTP-client arguments shared by `convert` and `search --scrape`.

use clap::Args;

/// Concurrency, rate-limiting, timeout, user-agent.
#[derive(Args, Debug, Clone)]
pub(crate) struct HttpArgs {
    /// Maximum concurrent HTTP requests.
    #[arg(short = 'j', long, default_value_t = 4)]
    pub concurrency: usize,

    /// Delay between requests in milliseconds (rate limiting).
    #[arg(long, default_value_t = 0)]
    pub delay: u64,

    /// Request timeout in seconds.
    #[arg(long, default_value_t = 30)]
    pub timeout: u64,

    /// Custom `User-Agent` header.
    #[arg(long)]
    pub user_agent: Option<String>,
}
