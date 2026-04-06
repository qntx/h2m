//! Conversion pipeline: raw HTML → `FetchResult`.

use std::borrow::Cow;
use std::time::Instant;

use super::types::{ContentExtraction, ConvertConfig, FetchResult, ResponseMeta};
use crate::html;

/// Single unified conversion path: raw HTML → `FetchResult`.
///
/// Parses the HTML once, then reuses the parsed document for title extraction,
/// link extraction, and CSS selection.
pub fn convert_to_result(
    url: Option<&str>,
    raw_html: &str,
    start: Instant,
    cfg: &ConvertConfig,
    resp: &ResponseMeta,
) -> FetchResult {
    let content_length = raw_html.len();
    let doc = scraper::Html::parse_document(raw_html);

    let html_to_convert: Cow<'_, str> = match &cfg.content {
        ContentExtraction::Full => Cow::Borrowed(raw_html),
        ContentExtraction::Selector(sel) => Cow::Owned(html::select_doc(&doc, raw_html, sel)),
        ContentExtraction::Readable => Cow::Owned(html::readable_content_doc(&doc, raw_html)),
    };

    let title = html::extract_title_doc(&doc);
    let language = html::extract_language_doc(&doc);
    let description = html::extract_description_doc(&doc);
    let og_image = html::extract_og_image_doc(&doc);

    let parsed_url = url.and_then(|u| url::Url::parse(u).ok());
    let auto_domain = parsed_url
        .as_ref()
        .and_then(|u| u.host_str().map(str::to_owned));
    let domain = cfg.domain.as_deref().or(auto_domain.as_deref());

    let links = if cfg.extract_links {
        Some(html::extract_links_doc(&doc, parsed_url.as_ref()))
    } else {
        None
    };
    let md = cfg.converter.convert(&html_to_convert);

    FetchResult {
        url: url.map(str::to_owned),
        domain: domain.map(str::to_owned),
        status_code: resp.status_code,
        content_type: resp.content_type.clone(),
        title,
        language,
        description,
        og_image,
        markdown: md,
        links,
        elapsed_ms: elapsed_ms(start),
        content_length,
    }
}

/// Returns elapsed milliseconds since `start`.
#[allow(clippy::cast_possible_truncation)]
fn elapsed_ms(start: Instant) -> u64 {
    start.elapsed().as_millis() as u64
}
