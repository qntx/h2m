//! Emphasis (`<em>`, `<i>`) conversion rule.

use scraper::ElementRef;

use crate::context::Context;
use crate::converter::{Action, Rule};
use crate::dom;

/// Handles `<em>` and `<i>` elements.
#[derive(Debug, Clone, Copy)]
pub struct Emphasis;

impl Rule for Emphasis {
    fn tags(&self) -> &'static [&'static str] {
        &["em", "i"]
    }

    fn apply(&self, content: &str, element: &ElementRef<'_>, ctx: &mut Context<'_>) -> Action {
        // Nested dedup: if parent is also em/i, just pass through.
        if dom::parent_tag_is(element, "em") || dom::parent_tag_is(element, "i") {
            return Action::Replace(content.to_owned());
        }

        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Action::Skip;
        }

        let mut delim_buf = [0u8; 4];
        let delim = ctx
            .options()
            .em_delimiter()
            .char()
            .encode_utf8(&mut delim_buf);
        let wrapped = super::wrap_delimiter_per_line(trimmed, delim);
        Action::Replace(dom::add_space_if_necessary(element, wrapped))
    }
}
