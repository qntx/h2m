//! HTML parsing for the `DuckDuckGo` provider.
//!
//! Isolated from the provider module so the network orchestration and
//! HTML-level extraction can evolve independently. The entry points are
//! [`parse_html_results`] (html.duckduckgo.com layout) and
//! [`parse_lite_results`] (lite.duckduckgo.com table layout).

use std::sync::LazyLock;

use scraper::{ElementRef, Html, Selector};

use super::PROVIDER_ID;
use super::captcha::MIN_HTML_BODY_BYTES;
use crate::error::SearchError;
use crate::response::SearchHit;

/// Selector bundle for the html.duckduckgo.com layout.
struct HtmlSelectors {
    result: Selector,
    title: Selector,
    snippet: Selector,
}

static HTML_SEL: LazyLock<HtmlSelectors> = LazyLock::new(|| HtmlSelectors {
    result: selector("div.result"),
    title: selector("a.result__a"),
    snippet: selector("a.result__snippet, .result__snippet"),
});

/// Row selector for the lite.duckduckgo.com table layout.
static LITE_RESULT_LINK_SEL: LazyLock<Selector> = LazyLock::new(|| selector("a.result-link"));
static LITE_RESULT_ROW_SEL: LazyLock<Selector> = LazyLock::new(|| selector("tr"));

/// Compiles a static CSS selector. The selector text is a compile-time
/// constant in every call site — `unwrap_or_else` here is equivalent to
/// `expect`, but satisfies `clippy::expect_used`.
fn selector(css: &'static str) -> Selector {
    Selector::parse(css).unwrap_or_else(|_| unreachable!("invalid static selector: {css}"))
}

/// Extracts results from the html.duckduckgo.com layout.
pub(super) fn parse_html_results(body: &str) -> Result<Vec<SearchHit>, SearchError> {
    let doc = Html::parse_document(body);
    let mut out = Vec::new();
    for node in doc.select(&HTML_SEL.result) {
        let Some(title_el) = node.select(&HTML_SEL.title).next() else {
            continue;
        };
        let title = collect_text(&title_el);
        let Some(href) = title_el.value().attr("href") else {
            continue;
        };
        let url = unwrap_ddg_url(href);
        if url.is_empty() || title.is_empty() {
            continue;
        }
        let description = node
            .select(&HTML_SEL.snippet)
            .next()
            .map(|s| collect_text(&s))
            .filter(|s| !s.is_empty());
        out.push(SearchHit {
            title,
            url,
            description,
            published_at: None,
            engine: Some(PROVIDER_ID.into()),
            score: None,
        });
    }

    if out.is_empty() && body.len() > MIN_HTML_BODY_BYTES {
        // Empty query result is legitimate; parse failure is not. Distinguish
        // by looking for the known "no results" marker.
        if !body.contains("No results.")
            && !body.contains("no-results")
            && !body.contains("No results found")
        {
            return Err(SearchError::ParseFailed {
                provider: PROVIDER_ID,
                message: "no div.result nodes matched in html endpoint".into(),
            });
        }
    }
    Ok(out)
}

/// Extracts results from the lite.duckduckgo.com layout.
pub(super) fn parse_lite_results(body: &str) -> Result<Vec<SearchHit>, SearchError> {
    let doc = Html::parse_document(body);
    let mut out = Vec::new();
    // The lite layout alternates rows: title-link, snippet, metadata, spacer.
    // We walk rows sequentially and attach the next row's text as the snippet.
    let rows: Vec<ElementRef<'_>> = doc.select(&LITE_RESULT_ROW_SEL).collect();
    let mut i = 0;
    while let Some(row) = rows.get(i).copied() {
        if let Some(link) = row.select(&LITE_RESULT_LINK_SEL).next() {
            let title = collect_text(&link);
            let Some(href) = link.value().attr("href") else {
                i += 1;
                continue;
            };
            let url = unwrap_ddg_url(href);
            if url.is_empty() || title.is_empty() {
                i += 1;
                continue;
            }
            let description = rows
                .get(i + 1)
                .map(|r| collect_text(r))
                .filter(|s| !s.is_empty());
            out.push(SearchHit {
                title,
                url,
                description,
                published_at: None,
                engine: Some(PROVIDER_ID.into()),
                score: None,
            });
            // Advance past the snippet + metadata + spacer rows.
            i += 4;
        } else {
            i += 1;
        }
    }

    if out.is_empty() && body.len() > MIN_HTML_BODY_BYTES && !body.contains("No results.") {
        return Err(SearchError::ParseFailed {
            provider: PROVIDER_ID,
            message: "no a.result-link anchors matched in lite endpoint".into(),
        });
    }
    Ok(out)
}

fn collect_text(el: &ElementRef<'_>) -> String {
    let mut buf = String::new();
    for chunk in el.text() {
        let trimmed = chunk.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !buf.is_empty() {
            buf.push(' ');
        }
        buf.push_str(trimmed);
    }
    buf
}

/// Unwraps a `DuckDuckGo` redirect link into the real destination URL.
///
/// `DuckDuckGo` wraps every outbound link in a `/l/?uddg=<encoded>` path.
/// Links may be absolute, protocol-relative, or site-relative.
pub(super) fn unwrap_ddg_url(href: &str) -> String {
    let href = href.trim();
    if href.is_empty() {
        return String::new();
    }
    let normalized = if href.starts_with("//") {
        format!("https:{href}")
    } else if let Some(rest) = href.strip_prefix('/') {
        format!("https://duckduckgo.com/{rest}")
    } else {
        href.to_owned()
    };

    if let Ok(parsed) = url::Url::parse(&normalized)
        && let Some(host) = parsed.host_str()
        && (host == "duckduckgo.com" || host.ends_with(".duckduckgo.com"))
        && parsed.path().starts_with("/l/")
        && let Some((_, value)) = parsed.query_pairs().find(|(k, _)| k == "uddg")
    {
        return value.into_owned();
    }
    normalized
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::unwrap_ddg_url;

    #[test]
    fn unwrap_handles_protocol_relative_and_absolute() {
        assert_eq!(
            unwrap_ddg_url("//duckduckgo.com/l/?uddg=https%3A%2F%2Frust-lang.org%2F&rut=xyz"),
            "https://rust-lang.org/"
        );
        assert_eq!(
            unwrap_ddg_url("/l/?uddg=https%3A%2F%2Fdocs.rs%2Fasync-trait&rut=abc"),
            "https://docs.rs/async-trait"
        );
        assert_eq!(
            unwrap_ddg_url("https://html.duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com%2F"),
            "https://example.com/"
        );
        assert_eq!(
            unwrap_ddg_url("https://rust-lang.org/"),
            "https://rust-lang.org/",
            "plain URLs pass through"
        );
        assert_eq!(unwrap_ddg_url(""), "");
    }
}
