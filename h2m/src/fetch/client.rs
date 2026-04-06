//! HTTP client logic: fetching HTML with meta-refresh redirect support.

use super::types::{FetchError, ResponseMeta};

/// Maximum number of `<meta http-equiv="refresh">` hops to follow.
const MAX_META_REDIRECTS: usize = 3;

/// Fetches HTML from a URL using the given client, following meta-refresh
/// redirects up to [`MAX_META_REDIRECTS`] times.
pub async fn fetch_html_inner(
    client: &reqwest::Client,
    url: &str,
) -> Result<(String, ResponseMeta), FetchError> {
    let mut current_url = url.to_owned();

    for _ in 0..=MAX_META_REDIRECTS {
        let resp = client
            .get(&current_url)
            .send()
            .await
            .map_err(|e| FetchError::Http {
                message: e.to_string(),
                url: current_url.clone(),
            })?;

        let meta = ResponseMeta {
            status_code: Some(resp.status().as_u16()),
            content_type: resp
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned),
        };

        let body = resp.text().await.map_err(|e| FetchError::Http {
            message: format!("failed to read response body: {e}"),
            url: current_url.clone(),
        })?;

        if let Some(target) = extract_meta_refresh(&body, &current_url) {
            current_url = target;
            continue;
        }

        return Ok((body, meta));
    }

    Err(FetchError::TooManyRedirects { url: current_url })
}

/// Extracts the redirect URL from a `<meta http-equiv="refresh">` tag, if
/// present. Returns `None` if the page has no such redirect.
fn extract_meta_refresh(html: &str, base_url: &str) -> Option<String> {
    let doc = scraper::Html::parse_document(html);
    let sel = scraper::Selector::parse("meta[http-equiv=\"refresh\" i]").ok()?;
    let meta = doc.select(&sel).next()?;
    let content = meta.value().attr("content")?;

    // Format: "0;url=https://..." or "0; url=https://..."
    let lower = content.to_ascii_lowercase();
    let url_start = lower.find("url=")?;
    let raw_target = content[url_start + 4..].trim().trim_matches(['"', '\'']);

    if raw_target.is_empty() {
        return None;
    }

    // Resolve relative redirect targets against the current URL.
    url::Url::parse(base_url).map_or_else(
        |_| Some(raw_target.to_owned()),
        |base| base.join(raw_target).ok().map(|u| u.to_string()),
    )
}
