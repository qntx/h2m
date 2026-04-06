//! Strong emphasis (`<strong>`, `<b>`) conversion rule.

use scraper::ElementRef;

use crate::context::Context;
use crate::converter::{Action, Rule};
use crate::dom;

/// Handles `<strong>` and `<b>` elements.
#[derive(Debug, Clone, Copy)]
pub struct Strong;

impl Rule for Strong {
    fn tags(&self) -> &'static [&'static str] {
        &["strong", "b"]
    }

    fn apply(&self, content: &str, element: &ElementRef<'_>, ctx: &mut Context<'_>) -> Action {
        // Nested dedup: if parent is also strong/b, just pass through.
        if dom::parent_tag_is(element, "strong") || dom::parent_tag_is(element, "b") {
            return Action::Replace(content.to_owned());
        }

        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Action::Skip;
        }

        let delim = ctx.options().strong_delimiter().as_str();
        let wrapped = super::wrap_delimiter_per_line(trimmed, delim);
        Action::Replace(dom::add_space_if_necessary(element, wrapped))
    }
}
