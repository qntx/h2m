//! Link (`<a>`) conversion rule.

use scraper::ElementRef;

use crate::context::Context;
use crate::rule::{Action, Rule};
use crate::utils;

/// Handles `<a>` elements as inline links: `[text](href "title")`.
#[derive(Debug, Clone, Copy)]
pub struct LinkRule;

impl Rule for LinkRule {
    fn tags(&self) -> &'static [&'static str] {
        &["a"]
    }

    fn apply(&self, content: &str, element: &ElementRef<'_>, _ctx: &Context) -> Action {
        let href = utils::attr(element, "href").unwrap_or("");
        let title = utils::attr(element, "title");

        // Skip empty links.
        if href.is_empty() && content.trim().is_empty() {
            return Action::Skip;
        }

        // If content is empty, use the URL as the text.
        let trimmed = content.trim();
        let display = if trimmed.is_empty() { href } else { trimmed };

        let title_part = title.map_or_else(String::new, |t| format!(" \"{t}\""));
        Action::Replace(format!("[{display}]({href}{title_part})"))
    }
}
