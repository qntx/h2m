//! Core converter: builder, frozen converter, and traversal pipeline.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use ego_tree::NodeId;
use scraper::node::Node;
use scraper::{ElementRef, Html};

use crate::context::{Context, ListMetadata};
use crate::escape;
use crate::options::Options;
use crate::plugin::Plugin;
use crate::rule::{Action, Rule};
use crate::whitespace;

/// Builder for constructing a [`Converter`] with custom rules and options.
#[derive(Default)]
#[allow(clippy::module_name_repetitions)]
pub struct ConverterBuilder {
    /// Conversion options.
    options: Options,
    /// Registered rules, keyed by tag name.
    rules: HashMap<&'static str, Vec<Arc<dyn Rule>>>,
    /// Tags whose raw HTML should be preserved in the output.
    keep_tags: HashSet<String>,
    /// Tags (and their content) to remove entirely from the output.
    remove_tags: HashSet<String>,
    /// Base domain for resolving relative URLs.
    domain: Option<String>,
}

impl std::fmt::Debug for ConverterBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConverterBuilder")
            .field("options", &self.options)
            .field(
                "rule_count",
                &self.rules.values().map(Vec::len).sum::<usize>(),
            )
            .field("keep_tags", &self.keep_tags)
            .field("remove_tags", &self.remove_tags)
            .field("domain", &self.domain)
            .finish()
    }
}

impl ConverterBuilder {
    /// Creates a new builder with default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the conversion options.
    #[must_use]
    pub const fn options(mut self, opts: Options) -> Self {
        self.options = opts;
        self
    }

    /// Registers a conversion rule.
    ///
    /// Rules registered later take priority over earlier rules for the same
    /// tag.
    pub fn add_rule(&mut self, rule: impl Rule + 'static) {
        let arc: Arc<dyn Rule> = Arc::new(rule);
        for &tag in arc.tags() {
            self.rules.entry(tag).or_default().push(Arc::clone(&arc));
        }
    }

    /// Applies a plugin, which may register rules and hooks.
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn use_plugin(mut self, plugin: impl Plugin) -> Self {
        plugin.register(&mut self);
        self
    }

    /// Adds tags whose raw HTML should be kept in the output.
    #[must_use]
    pub fn keep(mut self, tags: &[&str]) -> Self {
        self.keep_tags.extend(tags.iter().map(|&s| s.to_owned()));
        self
    }

    /// Adds tags (and their content) to remove from the output.
    #[must_use]
    pub fn remove(mut self, tags: &[&str]) -> Self {
        self.remove_tags.extend(tags.iter().map(|&s| s.to_owned()));
        self
    }

    /// Sets the base domain for resolving relative URLs to absolute.
    ///
    /// For example, setting `"example.com"` will turn `/page.html` into
    /// `http://example.com/page.html`.
    #[must_use]
    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    /// Builds the frozen [`Converter`].
    #[must_use]
    pub fn build(self) -> Converter {
        Converter {
            options: self.options,
            rules: self.rules,
            keep_tags: self.keep_tags,
            remove_tags: self.remove_tags,
            domain: self.domain,
        }
    }
}

/// A frozen, reusable HTML-to-Markdown converter.
///
/// Construct via [`Converter::builder()`] or the convenience
/// [`crate::convert()`] function.
pub struct Converter {
    /// Conversion options.
    options: Options,
    /// Rules keyed by tag name, tried in reverse order.
    rules: HashMap<&'static str, Vec<Arc<dyn Rule>>>,
    /// Tags to preserve as raw HTML.
    keep_tags: HashSet<String>,
    /// Tags to remove entirely.
    remove_tags: HashSet<String>,
    /// Base domain for resolving relative URLs.
    domain: Option<String>,
}

impl std::fmt::Debug for Converter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Converter")
            .field("options", &self.options)
            .field(
                "rule_count",
                &self.rules.values().map(Vec::len).sum::<usize>(),
            )
            .field("keep_tags", &self.keep_tags)
            .field("remove_tags", &self.remove_tags)
            .field("domain", &self.domain)
            .finish()
    }
}

// Compile-time assertion that `Converter` is `Send + Sync`.
const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Converter>();
};

impl Converter {
    /// Returns a new [`ConverterBuilder`].
    #[must_use]
    pub fn builder() -> ConverterBuilder {
        ConverterBuilder::new()
    }

    /// Converts an HTML string to Markdown.
    ///
    /// # Errors
    ///
    /// Returns an error if I/O fails during reading (when using reader-based
    /// APIs). Direct string conversion is infallible at the parsing level since
    /// html5ever recovers from malformed HTML.
    pub fn convert(&self, html: &str) -> crate::Result<String> {
        let document = Html::parse_document(html);
        let mut ctx = Context::new(self.options, self.domain.clone());

        // Pre-pass: compute list metadata.
        self.annotate_lists(&document, &mut ctx);

        // Traverse from the root element (<html>).
        let root_id = document.root_element().id();
        let mut output = self.process_node(root_id, &document, &mut ctx);

        // Append reference-style link definitions if any.
        if !ctx.references.is_empty() {
            output.push_str("\n\n");
            output.push_str(&ctx.references.join("\n"));
        }

        Ok(whitespace::clean_output(&output))
    }

    /// Converts HTML read from a reader to Markdown.
    ///
    /// # Errors
    ///
    /// Returns an error if reading from the reader fails.
    pub fn convert_reader(&self, mut reader: impl std::io::Read) -> crate::Result<String> {
        let mut html = String::new();
        reader.read_to_string(&mut html)?;
        self.convert(&html)
    }

    /// Walks the DOM to compute [`ListMetadata`] for every `<li>` element.
    fn annotate_lists(&self, document: &Html, ctx: &mut Context) {
        Self::annotate_list_node(
            &self.options,
            document.root_element().id(),
            document,
            ctx,
            0,
        );
    }

    /// Recursively annotates list items with their prefix and indentation.
    fn annotate_list_node(
        options: &Options,
        node_id: NodeId,
        document: &Html,
        ctx: &mut Context,
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
                max_number.to_string().len()
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
                        format!("{} ", options.bullet_marker)
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
                        options,
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
                Self::annotate_list_node(options, child.id(), document, ctx, parent_indent);
            }
        }
    }

    /// Processes a DOM node and returns its markdown representation.
    fn process_node(&self, node_id: NodeId, document: &Html, ctx: &mut Context) -> String {
        let Some(node_ref) = document.tree.get(node_id) else {
            return String::new();
        };

        match node_ref.value() {
            Node::Text(text) => Self::process_text(text, ctx),
            Node::Element(_) => {
                let Some(element_ref) = ElementRef::wrap(node_ref) else {
                    return String::new();
                };
                self.process_element(&element_ref, document, ctx)
            }
            Node::Document => {
                let mut combined = String::new();
                for child in node_ref.children() {
                    combined.push_str(&self.process_node(child.id(), document, ctx));
                }
                combined
            }
            _ => String::new(),
        }
    }

    /// Processes a text node.
    fn process_text(text: &scraper::node::Text, ctx: &Context) -> String {
        let raw: &str = text;

        if ctx.in_pre {
            return raw.to_owned();
        }

        let collapsed = whitespace::collapse_whitespace(raw);
        let escaped = escape::escape_markdown(&collapsed, ctx.options.escape_mode);

        escaped.into_owned()
    }

    /// Processes an element node by converting children first, then applying
    /// rules.
    fn process_element(
        &self,
        element: &ElementRef<'_>,
        document: &Html,
        ctx: &mut Context,
    ) -> String {
        let tag = element.value().name();

        // Check if this tag should be removed entirely.
        if self.remove_tags.contains(tag) || matches!(tag, "script" | "style" | "noscript") {
            return String::new();
        }

        // Track preformatted context — suppress whitespace collapse and
        // escaping inside `<pre>` and inline `<code>`/`<kbd>`/`<samp>`/`<tt>`.
        let was_in_pre = ctx.in_pre;
        if matches!(tag, "pre" | "code" | "kbd" | "samp" | "tt") {
            ctx.in_pre = true;
        }

        // Recursively convert children.
        let mut content = String::new();
        let Some(node_ref) = document.tree.get(element.id()) else {
            return String::new();
        };
        for child in node_ref.children() {
            content.push_str(&self.process_node(child.id(), document, ctx));
        }

        // Restore `<pre>` context.
        ctx.in_pre = was_in_pre;

        // Check if raw HTML should be kept.
        if self.keep_tags.contains(tag) {
            return element.html();
        }

        // Dispatch to rules (LIFO — last registered wins).
        if let Some(rules) = self.rules.get(tag) {
            for rule in rules.iter().rev() {
                match rule.apply(&content, element, &mut *ctx) {
                    Action::Replace(md) => return md,
                    Action::Remove => return String::new(),
                    Action::Skip => {}
                }
            }
        }

        // No rule matched — transparent passthrough.
        content
    }
}
