//! Strikethrough (`<del>`, `<s>`, `<strike>`) conversion rule.

use scraper::ElementRef;

use crate::context::Context;
use crate::rule::{Action, Rule};

/// Handles strikethrough elements.
#[derive(Debug, Clone, Copy)]
pub struct StrikethroughRule;

impl Rule for StrikethroughRule {
    fn tags(&self) -> &'static [&'static str] {
        &["del", "s", "strike"]
    }

    fn apply(&self, content: &str, _element: &ElementRef<'_>, _ctx: &Context) -> Action {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Action::Skip;
        }

        let leading = if content.starts_with(' ') { " " } else { "" };
        let trailing = if content.ends_with(' ') { " " } else { "" };

        Action::Replace(format!("{leading}~~{trimmed}~~{trailing}"))
    }
}
