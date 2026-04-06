//! Strikethrough (`<del>`, `<s>`, `<strike>`) conversion rule.

use scraper::ElementRef;

use crate::context::Context;
use crate::converter::{Action, Rule};
use crate::dom;

/// Handles strikethrough elements.
#[derive(Debug, Clone, Copy)]
pub struct Strikethrough;

impl Rule for Strikethrough {
    fn tags(&self) -> &'static [&'static str] {
        &["del", "s", "strike"]
    }

    fn apply(&self, content: &str, element: &ElementRef<'_>, _ctx: &mut Context) -> Action {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Action::Skip;
        }

        let text = format!("~~{trimmed}~~");
        Action::Replace(dom::add_space_if_necessary(element, text))
    }
}
