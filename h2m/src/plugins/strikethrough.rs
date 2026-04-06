//! Strikethrough (`<del>`, `<s>`, `<strike>`) conversion rule.

use scraper::ElementRef;

use crate::context::Context;
use crate::rule::{Action, Rule};
use crate::utils;

/// Handles strikethrough elements.
#[derive(Debug, Clone, Copy)]
pub struct StrikethroughRule;

impl Rule for StrikethroughRule {
    fn tags(&self) -> &'static [&'static str] {
        &["del", "s", "strike"]
    }

    fn apply(&self, content: &str, element: &ElementRef<'_>, _ctx: &mut Context) -> Action {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Action::Skip;
        }

        let text = format!("~~{trimmed}~~");
        Action::Replace(utils::add_space_if_necessary(element, text))
    }
}
