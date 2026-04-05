//! Strikethrough (`<del>`, `<s>`, `<strike>`) conversion rule.

use scraper::ElementRef;

use crate::context::ConversionContext;
use crate::rule::{Rule, RuleAction};

/// Handles strikethrough elements.
#[derive(Debug, Clone, Copy)]
pub struct StrikethroughRule;

impl Rule for StrikethroughRule {
    fn tags(&self) -> &'static [&'static str] {
        &["del", "s", "strike"]
    }

    fn apply(
        &self,
        content: &str,
        _element: &ElementRef<'_>,
        _ctx: &ConversionContext,
    ) -> RuleAction {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return RuleAction::Skip;
        }

        let leading = if content.starts_with(' ') { " " } else { "" };
        let trailing = if content.ends_with(' ') { " " } else { "" };

        RuleAction::Replace(format!("{leading}~~{trimmed}~~{trailing}"))
    }
}
