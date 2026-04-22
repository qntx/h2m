//! Output formatting for JSON and plain-text modes.

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use h2m::scrape::{ScrapeError, ScrapeResult};

/// Emits a single `ScrapeResult` to stdout or a file (JSON or plain).
pub(crate) fn emit_single(json: bool, output: Option<&Path>, result: &ScrapeResult) {
    if json {
        write_json_pretty(result);
    } else {
        write_markdown(output, &result.markdown);
    }
}

/// Emits a plain markdown string (stdin mode).
pub(crate) fn emit_single_markdown(json: bool, output: Option<&Path>, md: &str) {
    if json {
        #[derive(serde::Serialize)]
        struct StdinResult<'a> {
            markdown: &'a str,
        }
        write_json_pretty(&StdinResult { markdown: md });
    } else {
        write_markdown(output, md);
    }
}

/// Emits a streaming NDJSON line for batch results.
pub(crate) fn emit_ndjson(result: &Result<ScrapeResult, ScrapeError>) {
    let line = match result {
        Ok(r) => serde_json::to_string(r),
        Err(e) => serde_json::to_string(e),
    };
    if let Ok(json) = line {
        write_stdout_line(&json);
    }
}

/// Emits a batch result line (plain-text mode).
pub(crate) fn emit_batch_plain(result: &Result<ScrapeResult, ScrapeError>) {
    match result {
        Ok(r) => write_stdout_line(&r.markdown),
        Err(e) => eprintln!("error: {e}"),
    }
}

/// Prints a JSON error object for the `convert` pipeline.
pub(crate) fn emit_json_error(msg: &str, url: Option<&str>) {
    let e = ScrapeError::new(msg, url.map(str::to_owned));
    write_json_pretty(&e);
}

/// Serializes any value to pretty JSON and writes it to stdout.
#[cfg(feature = "search")]
pub(crate) fn emit_json_pretty<T: serde::Serialize>(value: &T) {
    write_json_pretty(value);
}

/// Writes a single NDJSON line for search results.
#[cfg(feature = "search")]
pub(crate) fn emit_search_ndjson<T: serde::Serialize>(value: &T) {
    if let Ok(line) = serde_json::to_string(value) {
        write_stdout_line(&line);
    }
}

/// Writes Markdown to file or stdout.
fn write_markdown(output: Option<&Path>, md: &str) {
    if let Some(path) = output {
        if let Err(e) = fs::write(path, md) {
            eprintln!("error: cannot write {}: {e}", path.display());
        } else {
            eprintln!("Written to {}", path.display());
        }
    } else {
        let stdout = io::stdout();
        let mut out = stdout.lock();
        _ = out.write_all(md.as_bytes());
        _ = out.write_all(b"\n");
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
    _ = writeln!(out, "{line}");
}
