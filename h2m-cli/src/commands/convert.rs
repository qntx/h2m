//! `h2m convert` — HTML → Markdown from URLs, files, or stdin.

use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::time::Duration;

use clap::Args;
use h2m::Converter;
use h2m::scrape::Scraper;

use crate::error::CliError;
use crate::output;
use crate::shared::{ContentArgs, FormatArgs, HttpArgs, build_options};

/// Arguments for `h2m convert`.
#[derive(Args, Debug)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "CLI flags are naturally boolean"
)]
pub(crate) struct ConvertArgs {
    /// URL(s), file path(s), or "-" for stdin.
    pub input: Vec<String>,

    /// Read URLs from a file (one per line, `#` comments supported).
    #[arg(long, value_name = "FILE")]
    pub urls: Option<PathBuf>,

    /// JSON output: single input → pretty JSON, multiple → NDJSON stream.
    #[arg(long)]
    pub json: bool,

    /// Extract every `<a href>` on the page (included in JSON output).
    #[arg(long)]
    pub extract_links: bool,

    /// Base domain for resolving relative URLs (auto-detected for URL input).
    #[arg(long)]
    pub domain: Option<String>,

    /// Output file path (stdout if omitted, ignored in batch mode).
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    #[command(flatten)]
    pub content: ContentArgs,

    #[command(flatten)]
    pub format: FormatArgs,

    #[command(flatten)]
    pub http: HttpArgs,
}

impl ConvertArgs {
    /// Runs the `convert` subcommand.
    ///
    /// # Errors
    ///
    /// Returns [`CliError`] on scrape, I/O, or file-read failures.
    pub(crate) async fn run(&self) -> Result<(), CliError> {
        let inputs = self.collect_inputs()?;

        if inputs.is_empty() {
            self.run_stdin()?;
            return Ok(());
        }

        let scraper = self.build_scraper()?;

        if let [input] = inputs.as_slice() {
            let result = scraper.scrape(input).await?;
            let mut sink = output::OutputSink::new(self.output.as_deref())?;
            sink.emit_single(self.json, &result);
        } else {
            self.run_batch(&scraper, &inputs).await?;
        }

        Ok(())
    }

    fn collect_inputs(&self) -> Result<Vec<String>, CliError> {
        let mut inputs: Vec<String> = self
            .input
            .iter()
            .filter(|s| s.as_str() != "-")
            .cloned()
            .collect();

        if let Some(path) = &self.urls {
            let content = fs::read_to_string(path).map_err(|e| {
                CliError::bad_input(format!("cannot read URL file {}: {e}", path.display()))
            })?;
            inputs.extend(
                content
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty() && !line.starts_with('#'))
                    .map(str::to_owned),
            );
        }

        if self.input.iter().any(|s| s == "-") && inputs.is_empty() {
            return Ok(Vec::new());
        }

        Ok(inputs)
    }

    fn run_stdin(&self) -> Result<(), CliError> {
        let converter = self.build_converter();

        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;

        let html = if let Some(sel) = &self.content.selector {
            h2m::html::select(&buf, sel)
        } else if self.content.readable {
            h2m::html::readable_content(&buf)
        } else {
            buf
        };
        let md = converter.convert(&html);
        let mut sink = output::OutputSink::new(self.output.as_deref())?;
        sink.emit_single_markdown(self.json, &md);
        Ok(())
    }

    async fn run_batch(&self, scraper: &Scraper, inputs: &[String]) -> Result<(), CliError> {
        let sink = std::sync::Mutex::new(output::OutputSink::new(self.output.as_deref())?);
        let json = self.json;
        scraper
            .scrape_many_streaming(inputs, |result| {
                let Ok(mut sink) = sink.lock() else {
                    return;
                };
                if json {
                    sink.emit_ndjson(&result);
                } else {
                    sink.emit_batch_plain(&result);
                }
            })
            .await;
        Ok(())
    }

    fn build_scraper(&self) -> Result<Scraper, CliError> {
        let mut builder = Scraper::builder()
            .options(build_options(&self.format))
            .gfm(self.format.gfm)
            .extract_links(self.extract_links)
            .concurrency(self.http.concurrency)
            .delay(Duration::from_millis(self.http.delay))
            .timeout(Duration::from_secs(self.http.timeout));

        if let Some(d) = &self.domain {
            builder = builder.domain(d);
        }
        if let Some(s) = &self.content.selector {
            builder = builder.selector(s);
        } else if self.content.readable {
            builder = builder.readable(true);
        }
        if let Some(ua) = &self.http.user_agent {
            builder = builder.user_agent(ua);
        }

        Ok(builder.build()?)
    }

    fn build_converter(&self) -> Converter {
        let mut builder = Converter::builder()
            .options(build_options(&self.format))
            .use_plugin(&h2m::rules::CommonMark);

        if self.format.gfm {
            builder = builder.use_plugin(&h2m::plugins::Gfm);
        }
        if let Some(d) = &self.domain {
            builder = builder.domain(d);
        }

        builder.build()
    }
}
