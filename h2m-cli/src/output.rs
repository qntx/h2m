//! Output formatting for JSON and plain-text modes.

use std::fs;
use std::io::{self, Write};
use std::process;

use h2m::fetch::{FetchError, FetchResult};

use crate::cli::Cli;

/// Emits a single result to stdout (JSON pretty-printed or plain Markdown).
pub fn emit_single(cli: &Cli, result: &Result<FetchResult, FetchError>) {
    if cli.json {
        emit_json(result);
        if result.is_err() {
            process::exit(1);
        }
    } else {
        match result {
            Ok(r) => write_markdown(cli, &r.markdown),
            Err(e) => {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
    }
}

/// Emits a streaming NDJSON line for batch results.
pub fn emit_ndjson(result: &Result<FetchResult, FetchError>) {
    let line = match result {
        Ok(r) => serde_json::to_string(r),
        Err(e) => serde_json::to_string(e),
    };
    if let Ok(json) = line {
        let stdout = io::stdout();
        let mut out = stdout.lock();
        let _ = writeln!(out, "{json}");
    }
}

/// Emits a batch result line (plain text mode).
pub fn emit_batch_plain(result: &Result<FetchResult, FetchError>) {
    match result {
        Ok(r) => {
            let stdout = io::stdout();
            let mut out = stdout.lock();
            let _ = writeln!(out, "{}", r.markdown);
        }
        Err(e) => {
            eprintln!("error: {e}");
        }
    }
}

/// Prints a JSON error to stdout.
pub fn emit_json_error(msg: &str, url: Option<&str>) {
    let e = FetchError::new(msg, url.map(str::to_owned));
    emit_json::<FetchResult>(&Err(e));
}

/// Writes a JSON object (pretty-printed) to stdout.
fn emit_json<T>(result: &Result<T, FetchError>)
where
    T: serde::Serialize,
{
    let rendered = match result {
        Ok(r) => serde_json::to_string_pretty(r),
        Err(e) => serde_json::to_string_pretty(e),
    };
    if let Ok(s) = rendered {
        let stdout = io::stdout();
        let mut out = stdout.lock();
        let _ = writeln!(out, "{s}");
    }
}

/// Writes Markdown to file or stdout.
pub fn write_markdown(cli: &Cli, md: &str) {
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
