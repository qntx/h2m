//! Exponential-backoff retry policy honouring `Retry-After` headers.
//!
//! The policy only retries [`SearchError::RateLimited`] and
//! [`SearchError::ServerError`]. Auth, validation and client errors are
//! returned immediately.
//!
//! # Example
//!
//! ```ignore
//! let response = retry(&RetryPolicy::default(), || async {
//!     do_request().await
//! }).await?;
//! ```

use std::future::Future;
use std::time::Duration;

use tracing::{debug, warn};

use crate::error::SearchError;

/// Parameters for the exponential-backoff retry loop.
#[derive(Debug, Clone, Copy)]
pub struct RetryPolicy {
    /// Maximum number of retries (total attempts = `max_retries + 1`).
    pub max_retries: u32,
    /// Initial backoff interval.
    pub base_delay: Duration,
    /// Cap applied to every computed backoff.
    pub max_delay: Duration,
}

impl RetryPolicy {
    /// Policy that disables retries entirely.
    pub const NONE: Self = Self {
        max_retries: 0,
        base_delay: Duration::ZERO,
        max_delay: Duration::ZERO,
    };

    /// Production default: 3 retries, 500 ms base, 10 s cap.
    pub const PRODUCTION: Self = Self {
        max_retries: 3,
        base_delay: Duration::from_millis(500),
        max_delay: Duration::from_secs(10),
    };
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::PRODUCTION
    }
}

/// Runs `operation` with the given retry policy.
///
/// # Errors
///
/// Returns whatever the final attempt produced. Non-retryable errors
/// (auth, invalid response, config) short-circuit immediately.
pub(crate) async fn retry<F, Fut, T>(
    policy: &RetryPolicy,
    mut operation: F,
) -> Result<T, SearchError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, SearchError>>,
{
    let mut attempt: u32 = 0;
    loop {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(err) => {
                if !err.is_retryable() || attempt >= policy.max_retries {
                    return Err(err);
                }
                let delay = backoff_delay(policy, &err, attempt);
                warn!(
                    attempt = attempt + 1,
                    max_attempts = policy.max_retries + 1,
                    delay_ms = u64::try_from(delay.as_millis()).unwrap_or(u64::MAX),
                    error = %err,
                    "retrying search request",
                );
                tokio::time::sleep(delay).await;
                attempt = attempt.saturating_add(1);
            }
        }
    }
}

/// Computes the delay for the next retry attempt.
///
/// Honours `Retry-After` when the provider surfaced one, otherwise falls
/// back to exponential backoff capped at `policy.max_delay`.
fn backoff_delay(policy: &RetryPolicy, err: &SearchError, attempt: u32) -> Duration {
    if let SearchError::RateLimited {
        retry_after: Some(hint),
        ..
    } = err
    {
        let hint_ms = u64::try_from(hint.as_millis()).unwrap_or(u64::MAX);
        debug!(hint_ms, "honouring Retry-After");
        return (*hint).min(policy.max_delay);
    }

    let exp = policy.base_delay.saturating_mul(1_u32 << attempt.min(10));
    exp.min(policy.max_delay)
}

#[cfg(test)]
#[allow(
    clippy::excessive_nesting,
    reason = "tokio::test closures with async blocks nest naturally"
)]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};

    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn ok_on_first_attempt_skips_sleep() {
        let policy = RetryPolicy::PRODUCTION;
        let counter = AtomicU32::new(0);
        let result: Result<u32, SearchError> = retry(&policy, || async {
            counter.fetch_add(1, Ordering::SeqCst);
            Ok(42)
        })
        .await;
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn retries_on_rate_limited_then_succeeds() {
        let policy = RetryPolicy {
            max_retries: 3,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(5),
        };
        let counter = AtomicU32::new(0);
        let result: Result<u32, SearchError> = retry(&policy, || {
            let c = counter.fetch_add(1, Ordering::SeqCst);
            async move {
                if c < 2 {
                    Err(SearchError::RateLimited {
                        provider: "searxng",
                        retry_after: Some(Duration::from_millis(1)),
                    })
                } else {
                    Ok(7)
                }
            }
        })
        .await;
        assert_eq!(result.unwrap(), 7);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn auth_errors_short_circuit() {
        let policy = RetryPolicy::PRODUCTION;
        let counter = AtomicU32::new(0);
        let result: Result<u32, SearchError> = retry(&policy, || {
            counter.fetch_add(1, Ordering::SeqCst);
            async {
                Err(SearchError::AuthFailed {
                    provider: "brave",
                    status: 401,
                })
            }
        })
        .await;
        assert!(matches!(result, Err(SearchError::AuthFailed { .. })));
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn backoff_grows_exponentially() {
        let policy = RetryPolicy {
            max_retries: 5,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
        };
        let err = SearchError::ServerError {
            provider: "x",
            status: 503,
        };
        assert_eq!(backoff_delay(&policy, &err, 0), Duration::from_millis(100));
        assert_eq!(backoff_delay(&policy, &err, 1), Duration::from_millis(200));
        assert_eq!(backoff_delay(&policy, &err, 2), Duration::from_millis(400));
    }

    #[test]
    fn backoff_respects_cap() {
        let policy = RetryPolicy {
            max_retries: 10,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(5),
        };
        let err = SearchError::ServerError {
            provider: "x",
            status: 500,
        };
        assert_eq!(backoff_delay(&policy, &err, 10), Duration::from_secs(5));
    }

    #[test]
    fn retry_after_wins_over_exponential() {
        let policy = RetryPolicy {
            max_retries: 5,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(10),
        };
        let err = SearchError::RateLimited {
            provider: "x",
            retry_after: Some(Duration::from_secs(2)),
        };
        assert_eq!(backoff_delay(&policy, &err, 3), Duration::from_secs(2));
    }
}
