//! Conversion context tracking traversal state.

use std::collections::HashMap;

use ego_tree::NodeId;
use scraper::ElementRef;

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
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct ConversionContext {
    /// The conversion options.
    pub(crate) options: Options,
    /// Counter for generating reference-style link indices (reserved for
    /// future use).
    #[allow(dead_code)]
    pub(crate) link_index: usize,
    /// Accumulated footer lines (e.g., reference link definitions).
    pub(crate) footers: Vec<String>,
    /// Pre-computed list metadata keyed by the `<li>` element's node ID.
    pub(crate) list_metadata: HashMap<NodeId, ListMetadata>,
    /// Whether we are currently inside a `<pre>` element.
    pub(crate) in_pre: bool,
}

impl ConversionContext {
    /// Creates a new context with the given options.
    pub(crate) fn new(options: Options) -> Self {
        Self {
            options,
            link_index: 0,
            footers: Vec::new(),
            list_metadata: HashMap::new(),
            in_pre: false,
        }
    }

    /// Returns the conversion options.
    #[must_use]
    pub const fn options(&self) -> &Options {
        &self.options
    }

    /// Returns `true` if we are currently inside a `<pre>` element.
    #[must_use]
    pub const fn in_pre(&self) -> bool {
        self.in_pre
    }

    /// Returns the list metadata for the given node ID, if any.
    #[must_use]
    pub fn list_metadata(&self, id: NodeId) -> Option<&ListMetadata> {
        self.list_metadata.get(&id)
    }

    /// Allocates and returns the next link reference index (reserved for
    /// future reference-style link support).
    #[allow(dead_code)]
    pub(crate) const fn next_link_index(&mut self) -> usize {
        self.link_index += 1;
        self.link_index
    }
}

/// Returns the value of an attribute on an element.
#[must_use]
pub fn attr<'a>(element: &'a ElementRef<'_>, name: &str) -> Option<&'a str> {
    element.value().attr(name)
}

/// Returns `true` if the given element has an ancestor with the specified tag
/// name.
#[must_use]
pub fn has_ancestor(element: &ElementRef<'_>, target_tag: &str) -> bool {
    let mut current = element.parent();
    while let Some(parent) = current {
        if let Some(el) = parent.value().as_element()
            && el.name() == target_tag
        {
            return true;
        }
        current = parent.parent();
    }
    false
}

/// Returns `true` if the given tag is an inline element.
#[must_use]
pub fn is_inline_element(tag: &str) -> bool {
    matches!(
        tag,
        "a" | "abbr"
            | "b"
            | "bdi"
            | "bdo"
            | "br"
            | "cite"
            | "code"
            | "data"
            | "del"
            | "dfn"
            | "em"
            | "i"
            | "img"
            | "input"
            | "ins"
            | "kbd"
            | "mark"
            | "q"
            | "s"
            | "samp"
            | "small"
            | "span"
            | "strike"
            | "strong"
            | "sub"
            | "sup"
            | "time"
            | "tt"
            | "u"
            | "var"
            | "wbr"
    )
}
