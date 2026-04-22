//! Helpers shared by every provider.
//!
//! Every provider classifies HTTP failures into the same fine-grained
//! [`SearchError`] variants and parses `Retry-After` identically. Keeping
//! that logic in one place avoids drift between providers.

use std::time::Duration;

use reqwest::Response;
use reqwest::StatusCode;

use crate::error::SearchError;

/// Classifies a non-success HTTP response into the appropriate
/// [`SearchError`] variant.
pub(crate) fn classify_status(provider: &'static str, response: &Response) -> SearchError {
    let status = response.status();
    let code = status.as_u16();
    if status == StatusCode::TOO_MANY_REQUESTS {
        SearchError::RateLimited {
            provider,
            retry_after: parse_retry_after(response),
        }
    } else if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
        SearchError::AuthFailed {
            provider,
            status: code,
        }
    } else if status.is_server_error() {
        SearchError::ServerError {
            provider,
            status: code,
        }
    } else {
        SearchError::Transport {
            provider,
            message: format!("unexpected HTTP {code}"),
        }
    }
}

/// Maps a transport-layer [`reqwest::Error`] to a [`SearchError`].
pub(crate) fn classify_transport(provider: &'static str, err: &reqwest::Error) -> SearchError {
    SearchError::Transport {
        provider,
        message: err.to_string(),
    }
}

/// Maps a deserialisation failure to [`SearchError::InvalidResponse`].
pub(crate) fn classify_parse(provider: &'static str, err: &reqwest::Error) -> SearchError {
    SearchError::InvalidResponse {
        provider,
        message: err.to_string(),
    }
}

/// Parses the `Retry-After` header into a [`Duration`].
///
/// The header can be either a number of seconds or an HTTP-date. We only
/// handle the seconds form — it is the format every provider in scope
/// actually uses.
fn parse_retry_after(response: &Response) -> Option<Duration> {
    let raw = response.headers().get(reqwest::header::RETRY_AFTER)?;
    let text = raw.to_str().ok()?;
    text.trim().parse::<u64>().ok().map(Duration::from_secs)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::error::SearchError;

    #[tokio::test]
    async fn classify_rate_limited_with_retry_after() {
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::any())
            .respond_with(wiremock::ResponseTemplate::new(429).insert_header("Retry-After", "7"))
            .mount(&server)
            .await;
        let response = reqwest::get(server.uri()).await.unwrap();
        let err = classify_status("x", &response);
        assert!(matches!(
            err,
            SearchError::RateLimited {
                provider: "x",
                retry_after: Some(d),
            } if d == Duration::from_secs(7)
        ));
    }

    #[tokio::test]
    async fn classify_auth_failed() {
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::any())
            .respond_with(wiremock::ResponseTemplate::new(403))
            .mount(&server)
            .await;
        let response = reqwest::get(server.uri()).await.unwrap();
        assert!(matches!(
            classify_status("x", &response),
            SearchError::AuthFailed {
                provider: "x",
                status: 403,
            }
        ));
    }

    #[tokio::test]
    async fn classify_server_error() {
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::any())
            .respond_with(wiremock::ResponseTemplate::new(503))
            .mount(&server)
            .await;
        let response = reqwest::get(server.uri()).await.unwrap();
        assert!(matches!(
            classify_status("x", &response),
            SearchError::ServerError {
                provider: "x",
                status: 503,
            }
        ));
    }
}
