//! # h2m
//!
//! A high-quality HTML-to-Markdown converter for Rust.
//!
//! ## Quick start
//!
//! ```
//! let md = h2m::convert("<h1>Hello</h1><p>World</p>");
//! assert_eq!(md, "# Hello\n\nWorld");
//! ```
//!
//! ## Custom options
//!
//! ```
//! use h2m::{Converter, Options};
//! use h2m::plugins::Gfm;
//!
//! let converter = Converter::builder()
//!     .options(Options::default())
//!     .use_plugin(Gfm)
//!     .build();
//!
//! let md = converter.convert("<del>old</del>");
//! assert_eq!(md, "~~old~~");
//! ```

pub mod html;
pub mod plugins;
pub mod rules;

#[cfg(feature = "fetch")]
pub mod fetch;

pub(crate) mod converter;
pub(crate) mod options;

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
    let converter = Converter::builder().use_plugin(rules::CommonMark).build();
    converter.convert(html)
}

/// Converts HTML to Markdown with GFM (GitHub Flavored Markdown) extensions.
#[must_use]
pub fn convert_gfm(html: &str) -> String {
    let converter = Converter::builder()
        .use_plugin(rules::CommonMark)
        .use_plugin(plugins::Gfm)
        .build();
    converter.convert(html)
}
