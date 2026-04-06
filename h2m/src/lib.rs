//! # h2m
//!
//! Fast, extensible HTML-to-Markdown converter for Rust — `CommonMark` + GFM,
//! plugin architecture, zero `unsafe`.
//!
//! ## Quick start
//!
//! ```
//! let md = h2m::convert("<h1>Hello</h1><p>World</p>");
//! assert_eq!(md, "# Hello\n\nWorld");
//! ```
//!
//! ## Builder API
//!
//! ```
//! use h2m::{Converter, Options};
//! use h2m::plugins::Gfm;
//! use h2m::rules::CommonMark;
//!
//! let converter = Converter::builder()
//!     .options(Options::default())
//!     .use_plugin(CommonMark)
//!     .use_plugin(Gfm)
//!     .domain("example.com")
//!     .build();
//!
//! let md = converter.convert(r#"<a href="/about">About</a>"#);
//! assert_eq!(md, "[About](https://example.com/about)");
//! ```
//!
//! ## HTML utilities
//!
//! The [`html`] module provides helpers for extracting page metadata without
//! running a full conversion:
//!
//! ```
//! let title = h2m::html::extract_title("<title>Hello</title>");
//! assert_eq!(title.as_deref(), Some("Hello"));
//! ```

use std::sync::LazyLock;

pub mod html;
pub mod plugins;
pub mod rules;

#[cfg(feature = "fetch")]
pub mod fetch;

pub mod converter;
pub mod options;

mod context;
mod dom;
mod escape;
mod whitespace;

pub use context::Context;
pub use converter::{Action, Converter, ConverterBuilder, Plugin, Rule};
pub use options::{
    BulletMarker, CodeBlockStyle, EmDelimiter, EscapeMode, Fence, HeadingStyle, HorizontalRule,
    LinkReferenceStyle, LinkStyle, Options, StrongDelimiter,
};

/// Errors that can occur during HTML-to-Markdown conversion.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// An I/O error occurred while reading input.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// A specialized [`Result`] type for h2m operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Converts HTML to Markdown using default `CommonMark` settings.
///
/// This is a convenience function equivalent to:
///
/// ```
/// # use h2m::Converter;
/// # use h2m::rules::CommonMark;
/// let converter = Converter::builder()
///     .use_plugin(CommonMark)
///     .build();
/// ```
#[must_use]
pub fn convert(html: &str) -> String {
    static CONVERTER: LazyLock<Converter> =
        LazyLock::new(|| Converter::builder().use_plugin(rules::CommonMark).build());
    CONVERTER.convert(html)
}

/// Converts HTML to Markdown with GFM (GitHub Flavored Markdown) extensions.
#[must_use]
pub fn convert_gfm(html: &str) -> String {
    static CONVERTER: LazyLock<Converter> = LazyLock::new(|| {
        Converter::builder()
            .use_plugin(rules::CommonMark)
            .use_plugin(plugins::Gfm)
            .build()
    });
    CONVERTER.convert(html)
}
