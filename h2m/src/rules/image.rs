//! Image (`<img>`) conversion rule.

use scraper::ElementRef;

use crate::context::{self as ctx, ConversionContext};
use crate::rule::{Rule, RuleAction};

/// Handles `<img>` elements.
#[derive(Debug, Clone, Copy)]
pub struct ImageRule;

impl Rule for ImageRule {
    fn tags(&self) -> &'static [&'static str] {
        &["img"]
    }

    fn apply(
        &self,
        _content: &str,
        element: &ElementRef<'_>,
        _ctx: &ConversionContext,
    ) -> RuleAction {
        let src = ctx::attr(element, "src").unwrap_or("");
        let alt = ctx::attr(element, "alt").unwrap_or("");
        let title = ctx::attr(element, "title");

        if src.is_empty() {
            return RuleAction::Skip;
        }

        let title_part = title.map_or_else(String::new, |t| format!(" \"{t}\""));
        RuleAction::Replace(format!("![{alt}]({src}{title_part})"))
    }
}
