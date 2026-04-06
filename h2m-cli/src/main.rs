//! h2m — HTML to Markdown converter CLI.

#![allow(clippy::print_stderr)]

use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process;

use clap::{Parser, ValueEnum};

/// Convert HTML to Markdown.
#[derive(Parser, Debug)]
#[command(name = "h2m", version, about)]
struct Cli {
    /// HTML file to convert (reads stdin if omitted or "-").
    input: Option<PathBuf>,

    /// Enable GFM extensions (tables, strikethrough, task lists).
    #[arg(long)]
    gfm: bool,

    /// Heading style.
    #[arg(long, value_enum, default_value_t = HeadingStyle::Atx)]
    heading_style: HeadingStyle,

    /// Bullet character for unordered lists.
    #[arg(long, default_value = "-")]
    bullet: char,

    /// Fence style for code blocks.
    #[arg(long, value_enum, default_value_t = FenceStyle::Backtick)]
    fence: FenceStyle,

    /// Disable markdown character escaping.
    #[arg(long)]
    no_escape: bool,

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

/// Code fence character style.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum FenceStyle {
    /// Triple backtick fences.
    Backtick,
    /// Triple tilde fences.
    Tilde,
}

fn main() {
    let cli = Cli::parse();

    let html = match &cli.input {
        Some(path) if path.to_str() != Some("-") => fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("error: cannot read {}: {e}", path.display());
            process::exit(1);
        }),
        _ => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).unwrap_or_else(|e| {
                eprintln!("error: cannot read stdin: {e}");
                process::exit(1);
            });
            buf
        }
    };

    let mut options = h2m::Options::default();

    if matches!(cli.heading_style, HeadingStyle::Setext) {
        options.heading_style = h2m::HeadingStyle::Setext;
    }
    options.bullet_marker = cli.bullet;
    if matches!(cli.fence, FenceStyle::Tilde) {
        options.fence = h2m::Fence::Tilde;
    }
    if cli.no_escape {
        options.escape_mode = h2m::EscapeMode::Disabled;
    }

    let mut builder = h2m::Converter::builder()
        .options(options)
        .use_plugin(h2m::rules::CommonMark);

    if cli.gfm {
        builder = builder.use_plugin(h2m::plugins::Gfm);
    }

    let converter = builder.build();

    let md = converter.convert(&html).unwrap_or_else(|e| {
        eprintln!("error: conversion failed: {e}");
        process::exit(1);
    });

    if let Some(path) = &cli.output {
        fs::write(path, &md).unwrap_or_else(|e| {
            eprintln!("error: cannot write {}: {e}", path.display());
            process::exit(1);
        });
    } else {
        let stdout = io::stdout();
        let mut out = stdout.lock();
        let _ = out.write_all(md.as_bytes());
        let _ = out.write_all(b"\n");
    }
}
