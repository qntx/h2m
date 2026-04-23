//! CAPTCHA / anti-bot detection for the `DuckDuckGo` HTML endpoints.
//!
//! `DuckDuckGo` has no standardised "you hit the limit" response. Instead
//! it silently swaps the results page for an anomaly-challenge stub. This
//! module turns those stubs into a structured [`SearchError::CaptchaDetected`]
//! and classifies parse-level failures as recoverable via the lite endpoint.

use crate::error::SearchError;

/// Minimum body size for a legitimate HTML response. Anything tiny is
/// almost certainly an error / challenge page.
pub(super) const MIN_HTML_BODY_BYTES: usize = 512;

/// Tokens whose presence in the response body signals an anti-bot page.
const CAPTCHA_MARKERS: &[&str] = &[
    "anomaly-modal",
    "challenge-platform",
    "DDG.anomalyDetection",
    "If this error persists",
    "/assets/common/error.js",
];

/// Returns `true` if `body` looks like a CAPTCHA / challenge page.
pub(super) fn looks_like_captcha(body: &str) -> bool {
    if body.len() < MIN_HTML_BODY_BYTES {
        return true;
    }
    CAPTCHA_MARKERS.iter().any(|m| body.contains(m))
}

/// Returns `true` when the error is recoverable by falling back to the
/// lite endpoint.
pub(super) const fn is_recoverable_via_lite(err: &SearchError) -> bool {
    matches!(
        err,
        SearchError::CaptchaDetected { .. }
            | SearchError::ParseFailed { .. }
            | SearchError::RateLimited { .. }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captcha_markers_detected() {
        assert!(looks_like_captcha("short"));
        assert!(looks_like_captcha(&format!(
            "{pad}anomaly-modal{pad}",
            pad = "x".repeat(400)
        )));
        assert!(!looks_like_captcha(
            &"<html><body><div class=\"result\">ok</div></body></html>".repeat(20)
        ));
    }

    #[test]
    fn is_recoverable_only_for_fallback_eligible_errors() {
        assert!(is_recoverable_via_lite(&SearchError::CaptchaDetected {
            provider: "duckduckgo",
        }));
        assert!(is_recoverable_via_lite(&SearchError::ParseFailed {
            provider: "duckduckgo",
            message: String::new(),
        }));
        assert!(is_recoverable_via_lite(&SearchError::RateLimited {
            provider: "duckduckgo",
            retry_after: None,
        }));
        assert!(!is_recoverable_via_lite(&SearchError::AuthFailed {
            provider: "duckduckgo",
            status: 401,
        }));
    }
}
