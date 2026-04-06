//! Conversion context tracking traversal state.

use std::borrow::Cow;
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

    /// Returns `true` if currently inside a preformatted element.
    #[inline]
    #[must_use]
    pub const fn in_pre(&self) -> bool {
        self.in_pre
    }

    /// Sets the preformatted context flag.
    #[inline]
    pub(crate) const fn set_in_pre(&mut self, value: bool) {
        self.in_pre = value;
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

    /// Resolves a potentially relative URL against the configured base domain.
    ///
    /// Returns the URL unchanged (as a borrow) when no resolution is needed.
    #[must_use]
    pub fn resolve_url<'a>(&self, raw_url: &'a str) -> Cow<'a, str> {
        let Some(domain) = self.domain.as_deref() else {
            return Cow::Borrowed(raw_url);
        };

        if domain.is_empty() {
            return Cow::Borrowed(raw_url);
        }

        // Already a valid absolute URL — return as-is.
        if url::Url::parse(raw_url).is_ok() {
            return Cow::Borrowed(raw_url);
        }

        // Construct a base URL from the domain and resolve against it.
        let base_str = if domain.contains("://") {
            Cow::Borrowed(domain)
        } else {
            Cow::Owned(format!("http://{domain}"))
        };

        let Ok(base) = url::Url::parse(&base_str) else {
            return Cow::Borrowed(raw_url);
        };

        base.join(raw_url)
            .map_or(Cow::Borrowed(raw_url), |u| Cow::Owned(u.to_string()))
    }

    /// Pushes a reference-style link definition and returns the next link
    /// index.
    pub fn push_reference(&mut self, reference: String) -> usize {
        self.link_index += 1;
        self.references.push(reference);
        self.link_index
    }

    /// Returns `true` if any reference-style link definitions were accumulated.
    #[inline]
    #[must_use]
    pub(crate) const fn has_references(&self) -> bool {
        !self.references.is_empty()
    }

    /// Takes all accumulated reference definitions, joining them into a single
    /// string and clearing the internal buffer.
    #[must_use]
    pub(crate) fn take_references(&mut self) -> String {
        let result = self.references.join("\n");
        self.references.clear();
        self.link_index = 0;
        result
    }

    /// Pre-computes [`ListMetadata`] for every `<li>` element in the document.
    pub(crate) fn annotate_lists(&mut self, root_id: NodeId, document: &scraper::Html) {
        Self::annotate_list_node(root_id, document, self, 0);
    }

    /// Recursively annotates list items with their prefix and indentation.
    fn annotate_list_node(
        node_id: NodeId,
        document: &scraper::Html,
        ctx: &mut Self,
        parent_indent: usize,
    ) {
        let Some(node_ref) = document.tree.get(node_id) else {
            return;
        };

        let is_list = node_ref.value().as_element().is_some_and(|el| {
            let name = el.name();
            name == "ul" || name == "ol"
        });

        if is_list {
            let el = node_ref.value().as_element();
            let is_ordered = el.is_some_and(|e| e.name() == "ol");
            let start: usize = el
                .and_then(|e| e.attr("start"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(1);

            let li_count = node_ref
                .children()
                .filter(|c| c.value().as_element().is_some_and(|e| e.name() == "li"))
                .count();

            let max_number = start + li_count.saturating_sub(1);
            let number_width = if is_ordered {
                digit_count(max_number)
            } else {
                0
            };

            let mut item_index = 0usize;
            for child in node_ref.children() {
                if child.value().as_element().is_some_and(|e| e.name() == "li") {
                    let prefix = if is_ordered {
                        let num = start + item_index;
                        format!("{num:>number_width$}. ")
                    } else {
                        format!("{} ", ctx.options.bullet_marker.char())
                    };
                    let prefix_width = prefix.len();

                    ctx.list_metadata.insert(
                        child.id(),
                        ListMetadata {
                            prefix,
                            prefix_width,
                            parent_indent,
                        },
                    );

                    Self::annotate_list_node(
                        child.id(),
                        document,
                        ctx,
                        parent_indent + prefix_width,
                    );

                    item_index += 1;
                }
            }
        } else {
            for child in node_ref.children() {
                Self::annotate_list_node(child.id(), document, ctx, parent_indent);
            }
        }
    }
}

/// Returns the number of decimal digits in `n`.
#[inline]
const fn digit_count(n: usize) -> usize {
    if n == 0 {
        return 1;
    }
    let mut count = 0;
    let mut val = n;
    while val > 0 {
        val /= 10;
        count += 1;
    }
    count
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;

    fn make_ctx(domain: Option<String>) -> Context {
        Context::new(Options::default(), domain)
    }

    #[test]
    fn resolve_no_domain() {
        let ctx = make_ctx(None);
        assert_eq!(ctx.resolve_url("/about"), "/about");
        assert!(matches!(ctx.resolve_url("/about"), Cow::Borrowed(_)));
    }

    #[test]
    fn resolve_empty_domain() {
        let ctx = make_ctx(Some(String::new()));
        assert_eq!(ctx.resolve_url("/about"), "/about");
    }

    #[test]
    fn resolve_absolute_url_unchanged() {
        let ctx = make_ctx(Some("example.com".to_owned()));
        let r = ctx.resolve_url("https://other.com/page");
        assert_eq!(r, "https://other.com/page");
        assert!(matches!(r, Cow::Borrowed(_)));
    }

    #[test]
    fn resolve_relative_with_bare_domain() {
        let ctx = make_ctx(Some("example.com".to_owned()));
        assert_eq!(ctx.resolve_url("/about"), "http://example.com/about");
    }

    #[test]
    fn resolve_relative_with_protocol() {
        let ctx = make_ctx(Some("https://example.com".to_owned()));
        assert_eq!(ctx.resolve_url("/about"), "https://example.com/about");
    }

    #[test]
    fn resolve_protocol_relative_url() {
        let ctx = make_ctx(Some("https://example.com".to_owned()));
        assert_eq!(
            ctx.resolve_url("//cdn.example.com/a.js"),
            "https://cdn.example.com/a.js"
        );
    }

    #[test]
    fn push_reference_increments_index() {
        let mut ctx = make_ctx(None);
        assert_eq!(ctx.push_reference("[1]: https://a.com".to_owned()), 1);
        assert_eq!(ctx.push_reference("[2]: https://b.com".to_owned()), 2);
        assert!(ctx.has_references());
    }

    #[test]
    fn take_references_joins_and_resets() {
        let mut ctx = make_ctx(None);
        ctx.push_reference("[1]: https://a.com".to_owned());
        ctx.push_reference("[2]: https://b.com".to_owned());

        let refs = ctx.take_references();
        assert_eq!(refs, "[1]: https://a.com\n[2]: https://b.com");
        assert!(!ctx.has_references());
        assert_eq!(ctx.link_index, 0);
    }

    #[test]
    fn take_references_empty() {
        let mut ctx = make_ctx(None);
        assert!(!ctx.has_references());
        assert_eq!(ctx.take_references(), "");
    }
}
