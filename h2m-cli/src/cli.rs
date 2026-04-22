//! CLI argument definitions and enum mappings.
//!
//! The CLI is organised as a subcommand tree:
//!
//! - `h2m convert <INPUT>…` — HTML-to-Markdown conversion (URL, file, stdin)
//! - `h2m search <QUERY>` — web search with optional scrape-to-Markdown
//!   (available when compiled with the `search` feature)
//!
//! Formatting, content-extraction, and HTTP flags are grouped into reusable
//! argument structs (`FormatArgs`, `ContentArgs`, `HttpArgs`) that are
//! `#[command(flatten)]`-ed into each subcommand, so both `convert` and
//! `search --scrape` share the exact same options.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

/// HTML-to-Markdown converter with optional web search.
///
/// Use `h2m <COMMAND> --help` for per-subcommand details.
#[derive(Parser, Debug)]
#[command(name = "h2m", version, about, long_about = None, propagate_version = true)]
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

/// Arguments for the `convert` subcommand.
#[derive(Args, Debug)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "CLI flags are naturally boolean"
)]
pub(crate) struct ConvertArgs {
    /// URL(s), file path(s), or "-" for stdin.
    pub input: Vec<String>,

    /// Read URLs from a file (one per line, `#` comments supported).
    #[arg(long, value_name = "FILE")]
    pub urls: Option<PathBuf>,

    /// JSON output: single input → pretty JSON, multiple → NDJSON stream.
    #[arg(long)]
    pub json: bool,

    /// Extract every `<a href>` on the page (included in JSON output).
    #[arg(long)]
    pub extract_links: bool,

    /// Base domain for resolving relative URLs (auto-detected for URL input).
    #[arg(long)]
    pub domain: Option<String>,

    /// Output file path (stdout if omitted, ignored in batch mode).
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Content-extraction selection.
    #[command(flatten)]
    pub content: ContentArgs,

    /// Markdown formatting options.
    #[command(flatten)]
    pub format: FormatArgs,

    /// HTTP client options.
    #[command(flatten)]
    pub http: HttpArgs,
}

/// Arguments for the `search` subcommand.
#[cfg(feature = "search")]
#[derive(Args, Debug)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "CLI flags are naturally boolean"
)]
pub(crate) struct SearchArgs {
    /// The search query.
    pub query: String,

    /// Search provider (`searxng` is the default).
    #[arg(short = 'p', long, default_value = "searxng")]
    pub provider: String,

    /// Maximum number of results per source (1..=100).
    #[arg(long, default_value_t = 10)]
    pub limit: usize,

    /// Result sources to request (comma-separated).
    #[arg(long, value_enum, value_delimiter = ',', default_value = "web")]
    pub sources: Vec<SourceArg>,

    /// Time-range filter.
    #[arg(long, value_enum)]
    pub time_range: Option<TimeRangeArg>,

    /// ISO 3166-1 alpha-2 country code (e.g. `us`, `cn`).
    #[arg(long)]
    pub country: Option<String>,

    /// ISO 639-1 language code (e.g. `en`, `zh`).
    #[arg(long)]
    pub language: Option<String>,

    /// Safe-search filter level.
    #[arg(long, value_enum, default_value_t = SafeSearchArg::Moderate)]
    pub safesearch: SafeSearchArg,

    /// `SearXNG` base URL (overrides `H2M_SEARXNG_URL`).
    #[arg(long)]
    pub searxng_url: Option<String>,

    /// After search, scrape each hit and emit a `ScrapeResult` per line (NDJSON).
    #[arg(long)]
    pub scrape: bool,

    /// JSON output (always on in scrape mode; pretty in non-scrape mode).
    #[arg(long)]
    pub json: bool,

    /// Output file path (stdout if omitted).
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Content-extraction selection (applied when `--scrape` is set).
    #[command(flatten)]
    pub content: ContentArgs,

    /// Markdown formatting options (applied when `--scrape` is set).
    #[command(flatten)]
    pub format: FormatArgs,

    /// HTTP client options for the scraping stage.
    #[command(flatten)]
    pub http: HttpArgs,

    /// Extract links from each scraped page (when `--scrape` is set).
    #[arg(long)]
    pub extract_links: bool,
}

/// Content-extraction options shared by `convert` and `search --scrape`.
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

/// Markdown formatting options shared by `convert` and `search --scrape`.
#[derive(Args, Debug, Clone)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "formatting toggles are naturally boolean"
)]
pub(crate) struct FormatArgs {
    /// Enable GFM extensions (tables, strikethrough, task lists).
    #[arg(short, long)]
    pub gfm: bool,

    /// Heading style.
    #[arg(long, value_enum, default_value_t = HeadingStyle::Atx)]
    pub heading_style: HeadingStyle,

    /// Bullet character for unordered lists.
    #[arg(long, value_enum, default_value_t = BulletStyle::Dash)]
    pub bullet: BulletStyle,

    /// Fence style for code blocks.
    #[arg(long, value_enum, default_value_t = FenceStyle::Backtick)]
    pub fence: FenceStyle,

    /// Emphasis delimiter.
    #[arg(long, value_enum, default_value_t = EmStyle::Star)]
    pub em: EmStyle,

    /// Strong delimiter.
    #[arg(long, value_enum, default_value_t = StrongStyle::Stars)]
    pub strong: StrongStyle,

    /// Horizontal rule style.
    #[arg(long, value_enum, default_value_t = HrStyle::Dashes)]
    pub hr: HrStyle,

    /// Link style.
    #[arg(long, value_enum, default_value_t = LinkStyleArg::Inlined)]
    pub link_style: LinkStyleArg,

    /// Reference link style (only used with `--link-style=referenced`).
    #[arg(long, value_enum, default_value_t = LinkRefArg::Full)]
    pub link_ref: LinkRefArg,

    /// Disable markdown character escaping.
    #[arg(long)]
    pub no_escape: bool,
}

/// HTTP client options shared by `convert` and `search --scrape`.
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

/// Heading rendering style.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum HeadingStyle {
    /// ATX-style (`# Heading`).
    Atx,
    /// Setext-style (underline with `===` or `---`).
    Setext,
}

impl From<HeadingStyle> for h2m::HeadingStyle {
    fn from(s: HeadingStyle) -> Self {
        match s {
            HeadingStyle::Atx => Self::Atx,
            HeadingStyle::Setext => Self::Setext,
        }
    }
}

/// Bullet character for unordered lists.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BulletStyle {
    /// Dash: `-`.
    Dash,
    /// Plus: `+`.
    Plus,
    /// Asterisk: `*`.
    Star,
}

impl From<BulletStyle> for h2m::BulletMarker {
    fn from(s: BulletStyle) -> Self {
        match s {
            BulletStyle::Dash => Self::Dash,
            BulletStyle::Plus => Self::Plus,
            BulletStyle::Star => Self::Asterisk,
        }
    }
}

/// Code fence character style.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum FenceStyle {
    /// Triple backtick fences.
    Backtick,
    /// Triple tilde fences.
    Tilde,
}

impl From<FenceStyle> for h2m::Fence {
    fn from(s: FenceStyle) -> Self {
        match s {
            FenceStyle::Backtick => Self::Backtick,
            FenceStyle::Tilde => Self::Tilde,
        }
    }
}

/// Emphasis delimiter style.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum EmStyle {
    /// Asterisk: `*text*`.
    Star,
    /// Underscore: `_text_`.
    Underscore,
}

impl From<EmStyle> for h2m::EmDelimiter {
    fn from(s: EmStyle) -> Self {
        match s {
            EmStyle::Star => Self::Asterisk,
            EmStyle::Underscore => Self::Underscore,
        }
    }
}

/// Horizontal rule style.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum HrStyle {
    /// Three dashes: `---`.
    Dashes,
    /// Three asterisks: `***`.
    Stars,
    /// Three underscores: `___`.
    Underscores,
}

impl From<HrStyle> for h2m::HorizontalRule {
    fn from(s: HrStyle) -> Self {
        match s {
            HrStyle::Dashes => Self::Dashes,
            HrStyle::Stars => Self::Asterisks,
            HrStyle::Underscores => Self::Underscores,
        }
    }
}

/// Link rendering style.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LinkStyleArg {
    /// Inline links: `[text](url)`.
    Inlined,
    /// Reference-style: `[text][id]` with footer definitions.
    Referenced,
}

impl From<LinkStyleArg> for h2m::LinkStyle {
    fn from(s: LinkStyleArg) -> Self {
        match s {
            LinkStyleArg::Inlined => Self::Inlined,
            LinkStyleArg::Referenced => Self::Referenced,
        }
    }
}

/// Reference link identifier style.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LinkRefArg {
    /// Full reference: `[text][1]`.
    Full,
    /// Collapsed: `[text][]`.
    Collapsed,
    /// Shortcut: `[text]`.
    Shortcut,
}

impl From<LinkRefArg> for h2m::LinkReferenceStyle {
    fn from(s: LinkRefArg) -> Self {
        match s {
            LinkRefArg::Full => Self::Full,
            LinkRefArg::Collapsed => Self::Collapsed,
            LinkRefArg::Shortcut => Self::Shortcut,
        }
    }
}

/// Strong emphasis delimiter style.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum StrongStyle {
    /// Double asterisks: `**bold**`.
    Stars,
    /// Double underscores: `__bold__`.
    Underscores,
}

impl From<StrongStyle> for h2m::StrongDelimiter {
    fn from(s: StrongStyle) -> Self {
        match s {
            StrongStyle::Stars => Self::Asterisks,
            StrongStyle::Underscores => Self::Underscores,
        }
    }
}

/// Search source category.
#[cfg(feature = "search")]
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SourceArg {
    /// General web results.
    Web,
    /// News articles.
    News,
    /// Image results.
    Images,
}

#[cfg(feature = "search")]
impl From<SourceArg> for h2m_search::SearchSource {
    fn from(s: SourceArg) -> Self {
        match s {
            SourceArg::Web => Self::Web,
            SourceArg::News => Self::News,
            SourceArg::Images => Self::Images,
        }
    }
}

/// Time-range filter for search.
#[cfg(feature = "search")]
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TimeRangeArg {
    /// Past 24 hours.
    Day,
    /// Past 7 days.
    Week,
    /// Past 30 days.
    Month,
    /// Past 12 months.
    Year,
}

#[cfg(feature = "search")]
impl From<TimeRangeArg> for h2m_search::TimeRange {
    fn from(t: TimeRangeArg) -> Self {
        match t {
            TimeRangeArg::Day => Self::Day,
            TimeRangeArg::Week => Self::Week,
            TimeRangeArg::Month => Self::Month,
            TimeRangeArg::Year => Self::Year,
        }
    }
}

/// Safe-search level.
#[cfg(feature = "search")]
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SafeSearchArg {
    /// No filtering.
    Off,
    /// Moderate filtering (default).
    Moderate,
    /// Strict filtering.
    Strict,
}

#[cfg(feature = "search")]
impl From<SafeSearchArg> for h2m_search::SafeSearch {
    fn from(s: SafeSearchArg) -> Self {
        match s {
            SafeSearchArg::Off => Self::Off,
            SafeSearchArg::Moderate => Self::Moderate,
            SafeSearchArg::Strict => Self::Strict,
        }
    }
}

/// Builds `h2m::Options` from the shared [`FormatArgs`].
pub(crate) fn build_options(format: &FormatArgs) -> h2m::Options {
    let mut opts = h2m::Options::default()
        .with_heading_style(format.heading_style.into())
        .with_bullet_marker(format.bullet.into())
        .with_fence(format.fence.into())
        .with_em_delimiter(format.em.into())
        .with_strong_delimiter(format.strong.into())
        .with_horizontal_rule(format.hr.into())
        .with_link_style(format.link_style.into())
        .with_link_reference_style(format.link_ref.into());

    if format.no_escape {
        opts = opts.with_escape_mode(h2m::EscapeMode::Disabled);
    }

    opts
}
