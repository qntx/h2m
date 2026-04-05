//! h2m — HTML to Markdown converter CLI.

#![allow(clippy::print_stderr)]

use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process;

use clap::Parser;

/// Convert HTML to Markdown.
#[derive(Parser, Debug)]
#[command(name = "h2m", version, about)]
struct Cli {
    /// HTML file to convert (reads stdin if omitted or "-").
    input: Option<PathBuf>,

    /// Enable GFM extensions (tables, strikethrough, task lists).
    #[arg(long)]
    gfm: bool,

    /// Heading style: "atx" or "setext".
    #[arg(long, default_value = "atx")]
    heading_style: String,

    /// Bullet character for unordered lists.
    #[arg(long, default_value = "-")]
    bullet: char,

    /// Fence style: "backtick" or "tilde".
    #[arg(long, default_value = "backtick")]
    fence: String,

    /// Link style: "inlined" or "referenced".
    #[arg(long, default_value = "inlined")]
    link_style: String,

    /// Disable markdown character escaping.
    #[arg(long)]
    no_escape: bool,

    /// Output file (writes to stdout if omitted).
    #[arg(short, long)]
    output: Option<PathBuf>,
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

    if cli.heading_style == "setext" {
        options.heading_style = h2m::options::HeadingStyle::Setext;
    }
    options.bullet_marker = cli.bullet;
    if cli.fence == "tilde" {
        options.fence = h2m::options::Fence::Tilde;
    }
    if cli.link_style == "referenced" {
        options.link_style = h2m::options::LinkStyle::Referenced;
    }
    if cli.no_escape {
        options.escape_mode = h2m::options::EscapeMode::Disabled;
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
