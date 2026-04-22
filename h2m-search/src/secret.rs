//! Minimal secret-handling primitives.
//!
//! [`SecretString`] wraps a credential and prevents accidental disclosure
//! via `Debug`, `Display`, or serde. The underlying bytes can only be
//! inspected through [`SecretString::expose`], which forces the caller
//! to acknowledge the leak.
//!
//! Intentionally zero-dependency — the [`secrecy`](https://crates.io/crates/secrecy)
//! crate provides a richer API (including `zeroize`), but pulling it in
//! would inflate the binary surface and the lint lockdown.

use std::fmt;

/// Opaque wrapper around a credential string.
///
/// - `Debug` / `Display` print `[REDACTED]`.
/// - `Clone` is intentionally preserved; callers that want single-owner
///   semantics should move the value instead.
/// - No `serde` impls — secrets must never be serialised accidentally.
#[derive(Clone)]
pub struct SecretString(String);

impl SecretString {
    /// Wraps a string as a secret.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the raw secret. The name is deliberately loud to make
    /// audit log greps easy — do not use this outside of the single
    /// network call that needs it.
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }

    /// Returns `true` if the secret is empty (trimmed).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.trim().is_empty()
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SecretString").field(&"[REDACTED]").finish()
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl From<String> for SecretString {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SecretString {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn debug_is_redacted() {
        let s = SecretString::new("supersecret");
        assert_eq!(format!("{s:?}"), r#"SecretString("[REDACTED]")"#);
    }

    #[test]
    fn display_is_redacted() {
        let s = SecretString::new("supersecret");
        assert_eq!(format!("{s}"), "[REDACTED]");
    }

    #[test]
    fn expose_returns_raw() {
        let s = SecretString::new("supersecret");
        assert_eq!(s.expose(), "supersecret");
    }

    #[test]
    fn empty_detection_trims() {
        assert!(SecretString::new("").is_empty());
        assert!(SecretString::new("   ").is_empty());
        assert!(!SecretString::new("x").is_empty());
    }
}
