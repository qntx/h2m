//! Link (`<a>`) conversion rule.

use scraper::ElementRef;

use crate::context::{self as ctx, ConversionContext};
use crate::options::LinkStyle;
use crate::rule::{Rule, RuleAction};

/// Handles `<a>` elements.
#[derive(Debug, Clone, Copy)]
pub struct LinkRule;

impl Rule for LinkRule {
    fn tags(&self) -> &'static [&'static str] {
        &["a"]
    }

    fn apply(
        &self,
        content: &str,
        element: &ElementRef<'_>,
        ctx: &ConversionContext,
    ) -> RuleAction {
        let href = ctx::attr(element, "href").unwrap_or("");
        let title = ctx::attr(element, "title");

        // Skip empty links.
        if href.is_empty() && content.trim().is_empty() {
            return RuleAction::Skip;
        }

        let content_trimmed = content.trim();

        // If content is empty, use the URL as the text.
        let display = if content_trimmed.is_empty() {
            href
        } else {
            content_trimmed
        };

        match ctx.options().link_style {
            LinkStyle::Inlined => {
                let title_part = title.map_or_else(String::new, |t| format!(" \"{t}\""));
                RuleAction::Replace(format!("[{display}]({href}{title_part})"))
            }
            LinkStyle::Referenced => {
                // Reference-style links are more complex — for now, fall back
                // to inlined. Full reference-style support can be added later
                // by accumulating references in the context footer.
                let title_part = title.map_or_else(String::new, |t| format!(" \"{t}\""));
                RuleAction::Replace(format!("[{display}]({href}{title_part})"))
            }
        }
    }
}
