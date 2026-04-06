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

/// Code fence character style.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum FenceStyle {
    /// Triple backtick fences.
    Backtick,
    /// Triple tilde fences.
    Tilde,
}

/// Emphasis delimiter style.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum EmStyle {
    /// Asterisk: `*text*`.
    Star,
    /// Underscore: `_text_`.
    Underscore,
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

/// Link rendering style.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum LinkStyleArg {
    /// Inline links: `[text](url)`.
    Inlined,
    /// Reference-style: `[text][id]` with footer definitions.
    Referenced,
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

/// Strong emphasis delimiter style.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum StrongStyle {
    /// Double asterisks: `**bold**`.
    Stars,
    /// Double underscores: `__bold__`.
    Underscores,
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
    match &cli.input {
        Some(input) if parse_as_url(input).is_some() => {
            let Some(parsed) = parse_as_url(input) else {
                unreachable!("guard already checked");
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
        Some(input) if input != "-" => {
            let path = PathBuf::from(input);
            let html = fs::read_to_string(&path).unwrap_or_else(|e| {
                eprintln!("error: cannot read {}: {e}", path.display());
                process::exit(1);
            });
            (html, None)
        }
        _ => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).unwrap_or_else(|e| {
                eprintln!("error: cannot read stdin: {e}");
                process::exit(1);
            });
            (buf, None)
        }
    }
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
    let mut options = h2m::Options::default();

    options.heading_style = match cli.heading_style {
        HeadingStyle::Atx => h2m::HeadingStyle::Atx,
        HeadingStyle::Setext => h2m::HeadingStyle::Setext,
    };

    options.bullet_marker = match cli.bullet {
        BulletStyle::Dash => h2m::BulletMarker::Dash,
        BulletStyle::Plus => h2m::BulletMarker::Plus,
        BulletStyle::Star => h2m::BulletMarker::Asterisk,
    };

    options.fence = match cli.fence {
        FenceStyle::Backtick => h2m::Fence::Backtick,
        FenceStyle::Tilde => h2m::Fence::Tilde,
    };

    options.em_delimiter = match cli.em {
        EmStyle::Star => h2m::EmDelimiter::Asterisk,
        EmStyle::Underscore => h2m::EmDelimiter::Underscore,
    };

    options.strong_delimiter = match cli.strong {
        StrongStyle::Stars => h2m::StrongDelimiter::Asterisks,
        StrongStyle::Underscores => h2m::StrongDelimiter::Underscores,
    };

    options.horizontal_rule = match cli.hr {
        HrStyle::Dashes => h2m::HorizontalRule::Dashes,
        HrStyle::Stars => h2m::HorizontalRule::Asterisks,
        HrStyle::Underscores => h2m::HorizontalRule::Underscores,
    };

    if cli.no_escape {
        options.escape_mode = h2m::EscapeMode::Disabled;
    }

    options.link_style = match cli.link_style {
        LinkStyleArg::Inlined => h2m::LinkStyle::Inlined,
        LinkStyleArg::Referenced => h2m::LinkStyle::Referenced,
    };

    options.link_reference_style = match cli.link_ref {
        LinkRefArg::Full => h2m::LinkReferenceStyle::Full,
        LinkRefArg::Collapsed => h2m::LinkReferenceStyle::Collapsed,
        LinkRefArg::Shortcut => h2m::LinkReferenceStyle::Shortcut,
    };

    options
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
