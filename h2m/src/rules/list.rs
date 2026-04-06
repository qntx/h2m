//! List (`<ul>`, `<ol>`, `<li>`) conversion rules.

use scraper::ElementRef;

use crate::context::Context;
use crate::converter::{Action, Rule};
use crate::dom;

/// Handles `<ul>` and `<ol>` list wrapper elements.
#[derive(Debug, Clone, Copy)]
pub struct List;

impl Rule for List {
    fn tags(&self) -> &'static [&'static str] {
        &["ul", "ol"]
    }

    fn apply(&self, content: &str, element: &ElementRef<'_>, _ctx: &mut Context) -> Action {
        let trimmed = content.trim_end_matches('\n');
        if trimmed.is_empty() {
            return Action::Skip;
        }

        if dom::has_ancestor(element, "li") {
            Action::Replace(format!("\n{trimmed}"))
        } else {
            Action::Replace(format!("\n\n{trimmed}\n\n"))
        }
    }
}

/// Handles `<li>` elements using pre-computed list metadata.
#[derive(Debug, Clone, Copy)]
pub struct ListItem;

impl Rule for ListItem {
    fn tags(&self) -> &'static [&'static str] {
        &["li"]
    }

    fn apply(&self, content: &str, element: &ElementRef<'_>, ctx: &mut Context) -> Action {
        let node_id = element.id();
        let Some(meta) = ctx.list_metadata(node_id) else {
            let trimmed = content.trim();
            return Action::Replace(format!("- {trimmed}\n"));
        };

        // Continuation lines are indented by the prefix width so they align
        // with the first line's content. We do NOT add parent_indent here
        // because the parent `<li>` already indents this item's output as
        // part of its own continuation lines.
        let continuation_indent = " ".repeat(meta.prefix_width);
        let trimmed = content.trim();

        let mut result = String::with_capacity(trimmed.len() + meta.prefix.len() + 8);
        for (i, line) in trimmed.lines().enumerate() {
            if i == 0 {
                result.push_str(&meta.prefix);
                result.push_str(line.trim_start());
            } else {
                result.push('\n');
                result.push_str(&continuation_indent);
                if !line.trim().is_empty() {
                    result.push_str(line);
                }
            }
        }
        result.push('\n');

        Action::Replace(result)
    }
}
