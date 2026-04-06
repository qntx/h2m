//! Horizontal rule (`<hr>`) conversion rule.

use scraper::ElementRef;

use crate::context::Context;
use crate::converter::{Action, Rule};

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

    fn apply(&self, _content: &str, element: &ElementRef<'_>, ctx: &mut Context) -> Action {
        if is_inside_heading(element) {
            return Action::Replace(String::new());
        }

        let rule = ctx.options().get_horizontal_rule().as_str();
        Action::Replace(format!("\n\n{rule}\n\n"))
    }
}

/// Returns `true` if the element is inside any heading tag (`h1`–`h6`).
fn is_inside_heading(element: &ElementRef<'_>) -> bool {
    let mut current = element.parent();
    while let Some(parent) = current {
        if let Some(el) = parent.value().as_element() {
            let name = el.name();
            if matches!(name, "h1" | "h2" | "h3" | "h4" | "h5" | "h6") {
                return true;
            }
        }
        current = parent.parent();
    }
    false
}
