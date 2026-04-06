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
//! # Save output to a file
//! h2m https://example.com -o output.md
//! ```

#![allow(clippy::print_stderr)]

use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process;

use clap::{Parser, ValueEnum};

/// Convert HTML to Markdown.
///
/// INPUT can be a URL (http/https), a file path, or "-" for stdin.
/// When omitted, reads from stdin.
#[derive(Parser, Debug)]
#[command(name = "h2m", version, about, long_about = None)]
struct Cli {
    /// URL, file path, or "-" for stdin.
    input: Option<String>,

    /// Enable GFM extensions (tables, strikethrough, task lists).
    #[arg(short, long)]
    gfm: bool,

    /// Heading style.
    #[arg(long, value_enum, default_value_t = HeadingStyle::Atx)]
    heading_style: HeadingStyle,

    /// Bullet character for unordered lists.
    #[arg(long, value_enum, default_value_t = BulletStyle::Dash)]
    bullet: BulletStyle,

    /// Fence style for code blocks.
    #[arg(long, value_enum, default_value_t = FenceStyle::Backtick)]
    fence: FenceStyle,

    /// Emphasis delimiter.
    #[arg(long, value_enum, default_value_t = EmStyle::Star)]
    em: EmStyle,

    /// Strong delimiter.
    #[arg(long, value_enum, default_value_t = StrongStyle::Stars)]
    strong: StrongStyle,

    /// Horizontal rule style.
    #[arg(long, value_enum, default_value_t = HrStyle::Dashes)]
    hr: HrStyle,

    /// Link style.
    #[arg(long, value_enum, default_value_t = LinkStyleArg::Inlined)]
    link_style: LinkStyleArg,

    /// Reference link style (only used with --link-style=referenced).
    #[arg(long, value_enum, default_value_t = LinkRefArg::Full)]
    link_ref: LinkRefArg,

    /// Disable markdown character escaping.
    #[arg(long)]
    no_escape: bool,

    /// Base domain for resolving relative URLs (e.g. "example.com").
    /// Auto-detected when input is a URL.
    #[arg(long)]
    domain: Option<String>,

    /// CSS selector to extract before converting (e.g. "article", "main",
    /// "#content").
    #[arg(short, long)]
    selector: Option<String>,

    /// Output file (writes to stdout if omitted).
    #[arg(short, long)]
    output: Option<PathBuf>,
}

/// Heading rendering style.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum HeadingStyle {
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
enum BulletStyle {
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
enum FenceStyle {
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
enum EmStyle {
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
enum HrStyle {
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
enum LinkStyleArg {
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
enum LinkRefArg {
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
enum StrongStyle {
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

fn main() {
    let cli = Cli::parse();

    let (raw_html, auto_domain) = read_input(&cli);

    let html = apply_selector(&cli, &raw_html);

    let options = build_options(&cli);

    let domain = cli.domain.as_deref().or(auto_domain.as_deref());

    let mut builder = h2m::Converter::builder()
        .options(options)
        .use_plugin(h2m::rules::CommonMark);

    if cli.gfm {
        builder = builder.use_plugin(h2m::plugins::Gfm);
    }

    if let Some(d) = domain {
        builder = builder.domain(d);
    }

    let converter = builder.build();

    let md = converter.convert(&html);

    write_output(&cli, &md);
}

/// Parses the input as a URL if it has an http/https scheme.
fn parse_as_url(input: &str) -> Option<url::Url> {
    let parsed = url::Url::parse(input).ok()?;
    if matches!(parsed.scheme(), "http" | "https") {
        Some(parsed)
    } else {
        None
    }
}

/// Reads HTML from URL, file, or stdin. Returns `(html, auto_domain)`.
fn read_input(cli: &Cli) -> (String, Option<String>) {
    let input = match &cli.input {
        Some(s) if s != "-" => s,
        _ => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).unwrap_or_else(|e| {
                eprintln!("error: cannot read stdin: {e}");
                process::exit(1);
            });
            return (buf, None);
        }
    };

    let Some(parsed) = parse_as_url(input) else {
        let path = PathBuf::from(input);
        let html = fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("error: cannot read {}: {e}", path.display());
            process::exit(1);
        });
        return (html, None);
    };

    let auto_domain = parsed.host_str().map(str::to_owned);
    eprintln!("Fetching {input}...");
    let body = reqwest::blocking::get(input)
        .unwrap_or_else(|e| {
            eprintln!("error: failed to fetch {input}: {e}");
            process::exit(1);
        })
        .text()
        .unwrap_or_else(|e| {
            eprintln!("error: failed to read response body: {e}");
            process::exit(1);
        });
    (body, auto_domain)
}

/// If `--selector` is given, extracts matching elements' inner HTML.
fn apply_selector(cli: &Cli, html: &str) -> String {
    let Some(sel) = &cli.selector else {
        return html.to_owned();
    };

    let document = scraper::Html::parse_document(html);
    let selector = scraper::Selector::parse(sel).unwrap_or_else(|e| {
        eprintln!("error: invalid CSS selector {sel:?}: {e}");
        process::exit(1);
    });

    let mut extracted = String::new();
    for element in document.select(&selector) {
        extracted.push_str(&element.inner_html());
    }

    if extracted.is_empty() {
        eprintln!("warning: selector {sel:?} matched no elements, converting full document");
        return html.to_owned();
    }

    extracted
}

/// Builds `h2m::Options` from CLI arguments.
fn build_options(cli: &Cli) -> h2m::Options {
    let mut opts = h2m::Options::default()
        .heading_style(cli.heading_style.into())
        .bullet_marker(cli.bullet.into())
        .fence(cli.fence.into())
        .em_delimiter(cli.em.into())
        .strong_delimiter(cli.strong.into())
        .horizontal_rule(cli.hr.into())
        .link_style(cli.link_style.into())
        .link_reference_style(cli.link_ref.into());

    if cli.no_escape {
        opts = opts.escape_mode(h2m::EscapeMode::Disabled);
    }

    opts
}

/// Writes the markdown output to file or stdout.
fn write_output(cli: &Cli, md: &str) {
    if let Some(path) = &cli.output {
        fs::write(path, md).unwrap_or_else(|e| {
            eprintln!("error: cannot write {}: {e}", path.display());
            process::exit(1);
        });
        eprintln!("Written to {}", path.display());
    } else {
        let stdout = io::stdout();
        let mut out = stdout.lock();
        let _ = out.write_all(md.as_bytes());
        let _ = out.write_all(b"\n");
    }
}
