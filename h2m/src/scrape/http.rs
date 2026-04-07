//! HTTP client: fetching HTML with meta-refresh redirect support.

use super::types::{HttpResponse, ScrapeError};

/// Maximum number of `<meta http-equiv="refresh">` hops to follow.
const MAX_META_REDIRECTS: usize = 3;

/// Fetches HTML from a URL, following meta-refresh redirects up to
/// [`MAX_META_REDIRECTS`] times.
///
/// Captures the final URL after both HTTP 3xx and meta-refresh redirects.
pub(super) async fn fetch_html(
    client: &reqwest::Client,
    url: &str,
) -> Result<HttpResponse, ScrapeError> {
    let mut current_url = url.to_owned();

    for _ in 0..=MAX_META_REDIRECTS {
        let resp = client
            .get(&current_url)
            .send()
            .await
            .map_err(|e| ScrapeError::Http {
                url: current_url.clone(),
                message: e.to_string(),
            })?;

        let status_code = resp.status().as_u16();
        let content_type = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        // Capture the final URL after any HTTP 3xx redirects.
        let final_url = resp.url().to_string();

        let body = resp.text().await.map_err(|e| ScrapeError::Http {
            url: current_url.clone(),
            message: format!("failed to read response body: {e}"),
        })?;

        if let Some(target) = extract_meta_refresh(&body, &current_url) {
            current_url = target;
            continue;
        }

        return Ok(HttpResponse {
            body,
            status_code,
            content_type,
            final_url,
        });
    }

    Err(ScrapeError::TooManyRedirects { url: current_url })
}

/// Extracts the redirect URL from a `<meta http-equiv="refresh">` tag.
///
/// Uses DOM parsing first, then falls back to regex on raw HTML to handle
/// `<noscript>`-wrapped meta-refresh tags (which `html5ever` treats as raw
/// text when scripting is enabled).
fn extract_meta_refresh(html: &str, base_url: &str) -> Option<String> {
    extract_meta_refresh_dom(html)
        .or_else(|| extract_meta_refresh_raw(html))
        .and_then(|raw| resolve_redirect_target(&raw, base_url))
}

/// DOM-based extraction: works for normal `<meta http-equiv="refresh">`.
fn extract_meta_refresh_dom(html: &str) -> Option<String> {
    let doc = scraper::Html::parse_document(html);
    let sel = scraper::Selector::parse("meta[http-equiv=\"refresh\" i]").ok()?;
    let content = doc.select(&sel).next()?.value().attr("content")?;
    extract_url_from_content(content)
}

/// Raw HTML fallback: catches `<noscript>`-wrapped meta-refresh that the
/// DOM parser cannot see (html5ever scripting-enabled mode).
fn extract_meta_refresh_raw(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let meta_pos = lower.find("http-equiv")?;
    let after_equiv = &lower[meta_pos..];
    if !after_equiv.contains("refresh") {
        return None;
    }
    let content_pos = lower[meta_pos..].find("content")?;
    let after_content = &html[meta_pos + content_pos..];
    let quote = after_content.find(['\"', '\''])?;
    let delim = *after_content.as_bytes().get(quote)?;
    let value_start = quote + 1;
    let value_end = after_content[value_start..].find(delim as char)? + value_start;
    let content = &after_content[value_start..value_end];
    extract_url_from_content(content)
}

/// Parses the `content` attribute value of a meta-refresh tag to extract
/// the target URL. Format: `"N;url=https://..."` or `"N; url=https://..."`.
fn extract_url_from_content(content: &str) -> Option<String> {
    let lower = content.to_ascii_lowercase();
    let url_start = lower.find("url=")?;
    let raw = content[url_start + 4..].trim().trim_matches(['"', '\'']);
    if raw.is_empty() {
        None
    } else {
        Some(raw.to_owned())
    }
}

/// Resolves a redirect target against the base URL.
fn resolve_redirect_target(raw_target: &str, base_url: &str) -> Option<String> {
    url::Url::parse(base_url).map_or_else(
        |_| Some(raw_target.to_owned()),
        |base| {
            if url::Url::parse(raw_target).is_ok() {
                Some(raw_target.to_owned())
            } else {
                base.join(raw_target).ok().map(|u| u.to_string())
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_refresh_basic() {
        let html = r#"<meta http-equiv="refresh" content="0;url=https://example.com/new">"#;
        let result = extract_meta_refresh(html, "https://example.com/old");
        assert_eq!(result.as_deref(), Some("https://example.com/new"));
    }

    #[test]
    fn meta_refresh_inside_noscript() {
        let html = r#"<!doctype html>
<meta charset="utf-8">
<title>Redirect</title>
<script>window.location.replace("https://example.com/new");</script>
<noscript>
  <meta http-equiv="refresh" content="0; url=https://example.com/new">
</noscript>
<p><a href="https://example.com/new">Click here</a></p>"#;
        let result = extract_meta_refresh(html, "https://example.com/old");
        assert_eq!(result.as_deref(), Some("https://example.com/new"));
    }

    #[test]
    fn meta_refresh_case_insensitive_attr() {
        let html = r#"<meta http-equiv="Refresh" content="0;url=https://example.com/new">"#;
        let result = extract_meta_refresh(html, "https://example.com/old");
        assert_eq!(result.as_deref(), Some("https://example.com/new"));
    }

    #[test]
    fn meta_refresh_relative_url() {
        let html = r#"<meta http-equiv="refresh" content="0;url=/new-page">"#;
        let result = extract_meta_refresh(html, "https://example.com/old");
        assert_eq!(result.as_deref(), Some("https://example.com/new-page"));
    }

    #[test]
    fn meta_refresh_no_url_returns_none() {
        let html = r#"<meta http-equiv="refresh" content="5">"#;
        assert!(extract_meta_refresh(html, "https://example.com").is_none());
    }
}
