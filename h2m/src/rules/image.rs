//! Image (`<img>`) conversion rule.

use scraper::ElementRef;

use crate::context::Context;
use crate::rule::{Action, Rule};
use crate::utils;

/// Handles `<img>` elements with absolute URL resolution.
#[derive(Debug, Clone, Copy)]
pub struct ImageRule;

impl Rule for ImageRule {
    fn tags(&self) -> &'static [&'static str] {
        &["img"]
    }

    fn apply(&self, _content: &str, element: &ElementRef<'_>, ctx: &mut Context) -> Action {
        let src = utils::attr(element, "src").unwrap_or("").trim();
        if src.is_empty() {
            return Action::Skip;
        }

        let absolute_src = utils::resolve_url(ctx.domain(), src);
        let alt = utils::attr(element, "alt").unwrap_or("").replace('\n', " ");
        let title = utils::attr(element, "title");

        let title_part = title.map_or_else(String::new, |t| format!(" \"{t}\""));
        Action::Replace(format!("![{alt}]({absolute_src}{title_part})"))
    }
}
