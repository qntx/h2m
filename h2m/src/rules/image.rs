//! Image (`<img>`) conversion rule.

use scraper::ElementRef;

use crate::context::Context;
use crate::converter::{Action, Rule};
use crate::dom;

/// Handles `<img>` elements with absolute URL resolution.
#[derive(Debug, Clone, Copy)]
pub struct Image;

impl Rule for Image {
    fn tags(&self) -> &'static [&'static str] {
        &["img"]
    }

    fn apply(&self, _content: &str, element: &ElementRef<'_>, ctx: &mut Context) -> Action {
        let src = dom::attr(element, "src").unwrap_or("").trim();
        if src.is_empty() {
            return Action::Skip;
        }

        let absolute_src = dom::resolve_url(ctx.domain(), src);
        let alt = dom::attr(element, "alt").unwrap_or("").replace('\n', " ");
        let title = dom::attr(element, "title");

        let title_part = title.map_or_else(String::new, |t| format!(" \"{t}\""));
        Action::Replace(format!("![{alt}]({absolute_src}{title_part})"))
    }
}
