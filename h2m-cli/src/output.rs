//! Output formatting for JSON, NDJSON, and plain-text modes.
//!
//! A single [`OutputSink`] abstracts over stdout and file destinations.
//! Every subcommand goes through it, so `--output` works uniformly for
//! both `convert` and `search`.

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use h2m::scrape::{ScrapeError, ScrapeResult};

use crate::error::CliError;

/// Write target: either a buffered file or locked stdout.
pub(crate) enum OutputSink {
    /// Writes to stdout (locked per-call). Single-threaded callers should
    /// wrap the sink in a `Mutex` to stream concurrently.
    Stdout,
    /// Buffered writer over an opened file path.
    File(BufWriter<File>, std::path::PathBuf),
}

impl OutputSink {
    /// Opens the sink. `None` targets stdout; `Some(path)` opens/truncates
    /// the file in buffered mode.
    ///
    /// # Errors
    ///
    /// Returns [`CliError::Io`] if the file cannot be created.
    pub(crate) fn new(path: Option<&Path>) -> Result<Self, CliError> {
        match path {
            None => Ok(Self::Stdout),
            Some(p) => {
                let file = File::create(p)?;
                Ok(Self::File(BufWriter::new(file), p.to_path_buf()))
            }
        }
    }

    /// Writes a single `ScrapeResult` (JSON or plain Markdown).
    pub(crate) fn emit_single(&mut self, json: bool, result: &ScrapeResult) {
        if json {
            self.write_json_pretty(result);
        } else {
            self.write_line(&result.markdown);
        }
    }

    /// Writes a stand-alone Markdown string (stdin mode in `convert`).
    pub(crate) fn emit_single_markdown(&mut self, json: bool, md: &str) {
        if json {
            #[derive(serde::Serialize)]
            struct StdinResult<'a> {
                markdown: &'a str,
            }
            self.write_json_pretty(&StdinResult { markdown: md });
        } else {
            self.write_line(md);
        }
    }

    /// Writes a single NDJSON line for a batch scrape result.
    pub(crate) fn emit_ndjson(&mut self, result: &Result<ScrapeResult, ScrapeError>) {
        let serialised = match result {
            Ok(r) => serde_json::to_string(r),
            Err(e) => serde_json::to_string(e),
        };
        if let Ok(line) = serialised {
            self.write_line(&line);
        }
    }

    /// Writes a plain-text batch entry. Errors go to stderr (not the sink)
    /// to keep the primary output clean.
    pub(crate) fn emit_batch_plain(&mut self, result: &Result<ScrapeResult, ScrapeError>) {
        match result {
            Ok(r) => self.write_line(&r.markdown),
            Err(e) => eprintln!("error: {e}"),
        }
    }

    /// Writes any serializable value as pretty-printed JSON.
    #[cfg(feature = "search")]
    pub(crate) fn emit_json_pretty<T: serde::Serialize>(&mut self, value: &T) {
        self.write_json_pretty(value);
    }

    /// Writes any serializable value as a single NDJSON line.
    #[cfg(feature = "search")]
    pub(crate) fn emit_search_ndjson<T: serde::Serialize>(&mut self, value: &T) {
        if let Ok(line) = serde_json::to_string(value) {
            self.write_line(&line);
        }
    }

    fn write_json_pretty(&mut self, value: &impl serde::Serialize) {
        if let Ok(s) = serde_json::to_string_pretty(value) {
            self.write_line(&s);
        }
    }

    fn write_line(&mut self, line: &str) {
        match self {
            Self::Stdout => {
                let stdout = io::stdout();
                let mut out = stdout.lock();
                _ = writeln!(out, "{line}");
            }
            Self::File(writer, _) => {
                _ = writeln!(writer, "{line}");
            }
        }
    }
}

impl Drop for OutputSink {
    fn drop(&mut self) {
        if let Self::File(w, path) = self {
            if let Err(e) = w.flush() {
                eprintln!("error: cannot flush {}: {e}", path.display());
            } else {
                eprintln!("Written to {}", path.display());
            }
        }
    }
}

/// Prints a JSON error object for the `convert` pipeline (always stdout).
pub(crate) fn emit_json_error(msg: &str, url: Option<&str>) {
    let e = ScrapeError::new(msg, url.map(str::to_owned));
    if let Ok(s) = serde_json::to_string_pretty(&e) {
        let stdout = io::stdout();
        let mut out = stdout.lock();
        _ = writeln!(out, "{s}");
    }
}
