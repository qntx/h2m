//! Horizontal rule (`<hr>`) conversion rule.

use scraper::ElementRef;

use crate::context::Context;
use crate::converter::{Action, Rule};
use crate::dom;

/// Heading tags used to detect when `<hr>` is inside a heading.
const HEADING_TAGS: &[&str] = &["h1", "h2", "h3", "h4", "h5", "h6"];

/// Handles `<hr>` elements.
///
/// Renders a thematic break, but suppresses it when the `<hr>` appears inside
/// a heading (which would look weird: `## --- Heading`).
#[derive(Debug, Clone, Copy)]
pub struct HorizontalRule;

impl Rule for HorizontalRule {
    fn tags(&self) -> &'static [&'static str] {
        &["hr"]
    }

    fn apply(&self, _content: &str, element: &ElementRef<'_>, ctx: &mut Context<'_>) -> Action {
        if dom::has_ancestor_any(element, HEADING_TAGS) {
            return Action::Replace(String::new());
        }

        let rule = ctx.options().horizontal_rule().as_str();
        Action::Replace(format!("\n\n{rule}\n\n"))
    }
}
