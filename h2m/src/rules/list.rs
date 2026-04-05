//! List (`<ul>`, `<ol>`, `<li>`) conversion rules.

use scraper::ElementRef;

use crate::context::{self as ctx, ConversionContext};
use crate::rule::{Rule, RuleAction};

/// Handles `<ul>` and `<ol>` list wrapper elements.
#[derive(Debug, Clone, Copy)]
pub struct ListRule;

impl Rule for ListRule {
    fn tags(&self) -> &'static [&'static str] {
        &["ul", "ol"]
    }

    fn apply(
        &self,
        content: &str,
        element: &ElementRef<'_>,
        _ctx: &ConversionContext,
    ) -> RuleAction {
        let trimmed = content.trim_end_matches('\n');
        if trimmed.is_empty() {
            return RuleAction::Skip;
        }

        // If this list is nested inside another list item, don't add extra
        // blank lines — just a single newline before.
        let is_nested = ctx::has_ancestor(element, "li");
        if is_nested {
            RuleAction::Replace(format!("\n{trimmed}"))
        } else {
            RuleAction::Replace(format!("\n\n{trimmed}\n\n"))
        }
    }
}

/// Handles `<li>` elements using pre-computed [`ListMetadata`].
#[derive(Debug, Clone, Copy)]
pub struct ListItemRule;

impl Rule for ListItemRule {
    fn tags(&self) -> &'static [&'static str] {
        &["li"]
    }

    fn apply(
        &self,
        content: &str,
        element: &ElementRef<'_>,
        ctx: &ConversionContext,
    ) -> RuleAction {
        let node_id = element.id();
        let Some(meta) = ctx.list_metadata(node_id) else {
            // No metadata — fall back to a simple bullet.
            let trimmed = content.trim();
            return RuleAction::Replace(format!("- {trimmed}\n"));
        };

        let indent = " ".repeat(meta.parent_indent);
        let prefix = &meta.prefix;

        // Handle multi-line content: continuation lines get indented to align
        // with the first line's content.
        let continuation_indent = " ".repeat(meta.parent_indent + meta.prefix_width);
        let trimmed = content.trim();

        let mut result = String::new();
        for (i, line) in trimmed.lines().enumerate() {
            if i == 0 {
                result.push_str(&indent);
                result.push_str(prefix);
                result.push_str(line.trim_start());
            } else {
                result.push('\n');
                result.push_str(&continuation_indent);
                if !line.trim().is_empty() {
                    result.push_str(line.trim_start());
                }
            }
        }
        result.push('\n');

        RuleAction::Replace(result)
    }
}
