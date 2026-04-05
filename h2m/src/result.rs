//! Conversion result types.

/// The result of converting a single HTML element to markdown.
///
/// Separates the output into three regions so that elements like reference-style
/// link definitions or footnotes can be collected and emitted at the document
/// boundaries.
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
#[allow(clippy::module_name_repetitions)]
pub struct AdvancedResult {
    /// Content to emit before the main document body (e.g., front matter).
    pub header: String,
    /// The main markdown content for this element.
    pub markdown: String,
    /// Content to emit after the main document body (e.g., link definitions).
    pub footer: String,
}

impl AdvancedResult {
    /// Creates a new result with only markdown content.
    #[must_use]
    pub fn markdown(content: String) -> Self {
        Self {
            markdown: content,
            ..Self::default()
        }
    }

    /// Returns `true` if all fields are empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.header.is_empty() && self.markdown.is_empty() && self.footer.is_empty()
    }
}
