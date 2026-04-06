//! Conversion pipeline: raw HTML → `FetchResult`.

use std::time::Instant;

use super::types::{ContentExtraction, ConvertConfig, FetchResult, ResponseMeta};
use crate::converter::{Converter, ConverterBuilder};
use crate::html;
use crate::options::Options;
use crate::plugins::Gfm;
use crate::rules::CommonMark;

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

    let html_to_convert = match &cfg.content {
        ContentExtraction::Full => raw_html.to_owned(),
        ContentExtraction::Selector(sel) => html::select_doc(&doc, raw_html, sel),
        ContentExtraction::Readable => html::readable_content_doc(&doc, raw_html),
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
    let md = convert_raw(&cfg.options, cfg.gfm, &html_to_convert, domain);

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

/// Builds a converter and runs the conversion.
fn convert_raw(options: &Options, gfm: bool, html: &str, domain: Option<&str>) -> String {
    let mut builder: ConverterBuilder = Converter::builder()
        .options(*options)
        .use_plugin(CommonMark);

    if gfm {
        builder = builder.use_plugin(Gfm);
    }

    if let Some(d) = domain {
        builder = builder.domain(d);
    }

    builder.build().convert(html)
}

/// Returns elapsed milliseconds since `start`.
#[allow(clippy::cast_possible_truncation)]
fn elapsed_ms(start: Instant) -> u64 {
    start.elapsed().as_millis() as u64
}
