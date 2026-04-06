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
fn extract_meta_refresh(html: &str, base_url: &str) -> Option<String> {
    let doc = scraper::Html::parse_document(html);
    let sel = scraper::Selector::parse("meta[http-equiv=\"refresh\" i]").ok()?;
    let meta = doc.select(&sel).next()?;
    let content = meta.value().attr("content")?;

    let lower = content.to_ascii_lowercase();
    let url_start = lower.find("url=")?;
    let raw_target = content[url_start + 4..].trim().trim_matches(['"', '\'']);

    if raw_target.is_empty() {
        return None;
    }

    url::Url::parse(base_url).map_or_else(
        |_| Some(raw_target.to_owned()),
        |base| base.join(raw_target).ok().map(|u| u.to_string()),
    )
}
