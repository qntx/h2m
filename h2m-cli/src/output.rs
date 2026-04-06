//! Output formatting for JSON and plain-text modes.

use std::fs;
use std::io::{self, Write};

use h2m::scrape::{ScrapeError, ScrapeResult};

use crate::cli::Cli;

/// Emits a single `ScrapeResult` to stdout (JSON pretty-printed or plain
/// Markdown).
pub fn emit_single(cli: &Cli, result: &ScrapeResult) {
    if cli.json {
        write_json_pretty(result);
    } else {
        write_markdown(cli, &result.markdown);
    }
}

/// Emits a plain markdown string to stdout (for stdin mode).
pub fn emit_single_markdown(cli: &Cli, md: &str) {
    if cli.json {
        #[derive(serde::Serialize)]
        struct StdinResult<'a> {
            markdown: &'a str,
        }
        write_json_pretty(&StdinResult { markdown: md });
    } else {
        write_markdown(cli, md);
    }
}

/// Emits a streaming NDJSON line for batch results.
pub fn emit_ndjson(result: &Result<ScrapeResult, ScrapeError>) {
    let line = match result {
        Ok(r) => serde_json::to_string(r),
        Err(e) => serde_json::to_string(e),
    };
    if let Ok(json) = line {
        write_stdout_line(&json);
    }
}

/// Emits a batch result line (plain text mode).
pub fn emit_batch_plain(result: &Result<ScrapeResult, ScrapeError>) {
    match result {
        Ok(r) => write_stdout_line(&r.markdown),
        Err(e) => eprintln!("error: {e}"),
    }
}

/// Prints a JSON error object to stdout.
pub fn emit_json_error(msg: &str, url: Option<&str>) {
    let e = ScrapeError::new(msg, url.map(str::to_owned));
    write_json_pretty(&e);
}

/// Writes Markdown to file or stdout.
fn write_markdown(cli: &Cli, md: &str) {
    if let Some(path) = &cli.output {
        if let Err(e) = fs::write(path, md) {
            eprintln!("error: cannot write {}: {e}", path.display());
        } else {
            eprintln!("Written to {}", path.display());
        }
    } else {
        let stdout = io::stdout();
        let mut out = stdout.lock();
        let _ = out.write_all(md.as_bytes());
        let _ = out.write_all(b"\n");
    }
}

/// Writes a pretty-printed JSON value to stdout.
fn write_json_pretty(value: &impl serde::Serialize) {
    if let Ok(s) = serde_json::to_string_pretty(value) {
        write_stdout_line(&s);
    }
}

/// Writes a single line to stdout, silently ignoring broken-pipe errors.
fn write_stdout_line(line: &str) {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let _ = writeln!(out, "{line}");
}
