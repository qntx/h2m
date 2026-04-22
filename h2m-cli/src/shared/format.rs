//! Markdown-formatting arguments shared by `convert` and `search --scrape`,
//! together with the `ValueEnum` wrappers that bridge the CLI to
//! [`h2m::Options`].

use clap::{Args, ValueEnum};

/// All Markdown-formatting toggles.
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

/// Builds [`h2m::Options`] from a [`FormatArgs`].
pub(crate) fn build_options(args: &FormatArgs) -> h2m::Options {
    let mut opts = h2m::Options::default()
        .with_heading_style(args.heading_style.into())
        .with_bullet_marker(args.bullet.into())
        .with_fence(args.fence.into())
        .with_em_delimiter(args.em.into())
        .with_strong_delimiter(args.strong.into())
        .with_horizontal_rule(args.hr.into())
        .with_link_style(args.link_style.into())
        .with_link_reference_style(args.link_ref.into());

    if args.no_escape {
        opts = opts.with_escape_mode(h2m::EscapeMode::Disabled);
    }

    opts
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
