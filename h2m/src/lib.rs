//! # h2m
//!
//! A high-quality HTML-to-Markdown converter for Rust.
//!
//! ## Quick start
//!
//! ```
//! let md = h2m::convert("<h1>Hello</h1><p>World</p>").unwrap();
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
//! let md = converter.convert("<del>old</del>").unwrap();
//! assert_eq!(md, "~~old~~");
//! ```

pub mod plugins;
pub mod rules;

pub(crate) mod converter;
pub(crate) mod error;
pub(crate) mod options;
pub(crate) mod plugin;
pub(crate) mod rule;

mod context;
mod escape;
mod utils;
mod whitespace;

// Re-export `Context` so external `Rule` implementors can use it.
pub use context::Context;
pub use converter::{Converter, ConverterBuilder};
pub use error::{Error, Result};
pub use options::{
    CodeBlockStyle, EscapeMode, Fence, HeadingStyle, LinkReferenceStyle, LinkStyle, Options,
};
pub use rule::{Action, Rule};

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
///
/// # Errors
///
/// Returns an error if I/O fails. HTML parsing itself is infallible since
/// html5ever recovers from malformed input.
pub fn convert(html: &str) -> Result<String> {
    let converter = Converter::builder().use_plugin(rules::CommonMark).build();
    converter.convert(html)
}

/// Converts HTML to Markdown with GFM (GitHub Flavored Markdown) extensions.
///
/// # Errors
///
/// Returns an error if I/O fails.
pub fn convert_gfm(html: &str) -> Result<String> {
    let converter = Converter::builder()
        .use_plugin(rules::CommonMark)
        .use_plugin(plugins::Gfm)
        .build();
    converter.convert(html)
}
