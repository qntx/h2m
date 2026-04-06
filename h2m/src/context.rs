//! Conversion context tracking traversal state.

use std::collections::HashMap;

use ego_tree::NodeId;

use crate::options::Options;

/// Metadata computed during the list pre-pass for each `<li>` element.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ListMetadata {
    /// The list item prefix (e.g., `"- "`, `"1. "`, `"10. "`).
    pub prefix: String,
    /// The character width of the prefix, used for continuation indent.
    pub prefix_width: usize,
    /// The total indentation from all ancestor lists.
    pub parent_indent: usize,
}

/// State maintained during the conversion traversal.
///
/// Passed to [`Rule::apply`](crate::Rule::apply) so rules can access
/// conversion options and list metadata.
#[derive(Debug)]
pub struct Context {
    /// The conversion options.
    pub(crate) options: Options,
    /// Pre-computed list metadata keyed by the `<li>` element's node ID.
    pub(crate) list_metadata: HashMap<NodeId, ListMetadata>,
    /// Whether we are currently inside a `<pre>` or inline `<code>` element,
    /// where text should not be whitespace-collapsed or escaped.
    pub(crate) in_pre: bool,
    /// Optional base domain for resolving relative URLs to absolute.
    pub(crate) domain: Option<String>,
    /// Accumulated reference-style link definitions (appended after body).
    pub(crate) references: Vec<String>,
    /// Monotonically increasing link index for `LinkReferenceStyle::Full`.
    pub(crate) link_index: usize,
}

impl Context {
    /// Creates a new context with the given options and optional domain.
    pub(crate) fn new(options: Options, domain: Option<String>) -> Self {
        Self {
            options,
            list_metadata: HashMap::new(),
            in_pre: false,
            domain,
            references: Vec::new(),
            link_index: 0,
        }
    }

    /// Returns the conversion options.
    #[inline]
    #[must_use]
    pub const fn options(&self) -> &Options {
        &self.options
    }

    /// Returns the list metadata for the given node ID, if any.
    #[inline]
    #[must_use]
    pub fn list_metadata(&self, id: NodeId) -> Option<&ListMetadata> {
        self.list_metadata.get(&id)
    }

    /// Returns the base domain used for resolving relative URLs.
    #[inline]
    #[must_use]
    pub fn domain(&self) -> Option<&str> {
        self.domain.as_deref()
    }

    /// Pushes a reference-style link definition and returns the next link
    /// index.
    pub fn push_reference(&mut self, reference: String) -> usize {
        self.link_index += 1;
        self.references.push(reference);
        self.link_index
    }
}
