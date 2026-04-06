//! Image (`<img>`) conversion rule.

use scraper::ElementRef;

use crate::context::Context;
use crate::rule::{Action, Rule};
use crate::utils;

/// Handles `<img>` elements.
#[derive(Debug, Clone, Copy)]
pub struct ImageRule;

impl Rule for ImageRule {
    fn tags(&self) -> &'static [&'static str] {
        &["img"]
    }

    fn apply(&self, _content: &str, element: &ElementRef<'_>, _ctx: &Context) -> Action {
        let src = utils::attr(element, "src").unwrap_or("");
        let alt = utils::attr(element, "alt").unwrap_or("");
        let title = utils::attr(element, "title");

        if src.is_empty() {
            return Action::Skip;
        }

        let title_part = title.map_or_else(String::new, |t| format!(" \"{t}\""));
        Action::Replace(format!("![{alt}]({src}{title_part})"))
    }
}
