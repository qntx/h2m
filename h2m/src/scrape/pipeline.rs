//! Conversion pipeline: raw HTML → [`ScrapeResult`].

use std::borrow::Cow;
use std::time::Instant;

use super::types::{ContentExtraction, ConvertConfig, HttpResponse, Metadata, ScrapeResult};
use crate::html;

/// Converts fetched HTML into a [`ScrapeResult`].
///
/// Parses the HTML once, reusing the parsed document for metadata extraction,
/// content selection, and link extraction.
pub(super) fn build_result(
    source_url: &str,
    response: &HttpResponse,
    start: Instant,
    cfg: &ConvertConfig,
) -> ScrapeResult {
    let doc = scraper::Html::parse_document(&response.body);

    let html_to_convert: Cow<'_, str> = match &cfg.content {
        ContentExtraction::Full => Cow::Borrowed(&response.body),
        ContentExtraction::Selector(sel) => Cow::Owned(html::select_doc(&doc, &response.body, sel)),
        ContentExtraction::Readable => Cow::Owned(html::readable_content_doc(&doc, &response.body)),
    };

    let page = html::PageMeta::from_doc(&doc);
    let parsed_url = url::Url::parse(source_url).ok();

    let links = if cfg.extract_links {
        Some(html::extract_links_doc(&doc, parsed_url.as_ref()))
    } else {
        None
    };

    let markdown = cfg.converter.convert(&html_to_convert);

    let elapsed_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

    ScrapeResult {
        markdown,
        metadata: Metadata {
            title: page.title,
            description: page.description,
            language: page.language,
            og_image: page.og_image,
            source_url: source_url.to_owned(),
            url: response.final_url.clone(),
            status_code: response.status_code,
            content_type: response.content_type.clone(),
            elapsed_ms,
        },
        links,
    }
}
