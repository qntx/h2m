//! CLI argument definitions and enum mappings.

use std::path::PathBuf;

use clap::{Parser, ValueEnum};

/// Convert HTML to Markdown.
///
/// INPUT can be one or more URLs, file paths, or "-" for stdin.
/// When omitted, reads from stdin. Use --json for structured output.
#[derive(Parser, Debug)]
#[command(name = "h2m", version, about, long_about = None)]
#[allow(clippy::struct_excessive_bools)]
pub struct Cli {
    /// URL(s), file path(s), or "-" for stdin.
    pub input: Vec<String>,

    /// JSON output mode for programmatic/agent consumption.
    /// Single input produces a JSON object; multiple inputs produce NDJSON.
    #[arg(long, global = true)]
    pub json: bool,

    /// Extract all links from the page (included in JSON output).
    #[arg(long)]
    pub extract_links: bool,

    /// Read URLs from a file (one per line).
    #[arg(long, value_name = "FILE")]
    pub urls: Option<PathBuf>,

    /// Maximum concurrent requests for batch mode.
    #[arg(short = 'j', long, default_value_t = 4)]
    pub concurrency: usize,

    /// Delay between requests in milliseconds (rate limiting).
    #[arg(long, default_value_t = 0)]
    pub delay: u64,

    /// Request timeout in seconds.
    #[arg(long, default_value_t = 30)]
    pub timeout: u64,

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

    /// Reference link style (only used with --link-style=referenced).
    #[arg(long, value_enum, default_value_t = LinkRefArg::Full)]
    pub link_ref: LinkRefArg,

    /// Disable markdown character escaping.
    #[arg(long)]
    pub no_escape: bool,

    /// Base domain for resolving relative URLs (e.g. "example.com").
    /// Auto-detected when input is a URL.
    #[arg(long)]
    pub domain: Option<String>,

    /// CSS selector to extract before converting (e.g. "article", "main",
    /// "#content"). Mutually exclusive with --readable.
    #[arg(short, long, conflicts_with = "readable")]
    pub selector: Option<String>,

    /// Smart readable content extraction.
    /// Phase 1: tries semantic selectors (article, main, [role="main"], …).
    /// Phase 2: strips noise elements (nav, footer, aside, …) if no
    /// semantic wrapper is found.
    /// Mutually exclusive with --selector.
    #[arg(short = 'r', long, conflicts_with = "selector")]
    pub readable: bool,

    /// Custom User-Agent header for HTTP requests.
    #[arg(long)]
    pub user_agent: Option<String>,

    /// Output file (writes to stdout if omitted, ignored in batch mode).
    #[arg(short, long)]
    pub output: Option<PathBuf>,
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

/// Builds `h2m::Options` from CLI arguments.
pub fn build_options(cli: &Cli) -> h2m::Options {
    let mut opts = h2m::Options::default()
        .with_heading_style(cli.heading_style.into())
        .with_bullet_marker(cli.bullet.into())
        .with_fence(cli.fence.into())
        .with_em_delimiter(cli.em.into())
        .with_strong_delimiter(cli.strong.into())
        .with_horizontal_rule(cli.hr.into())
        .with_link_style(cli.link_style.into())
        .with_link_reference_style(cli.link_ref.into());

    if cli.no_escape {
        opts = opts.with_escape_mode(h2m::EscapeMode::Disabled);
    }

    opts
}
