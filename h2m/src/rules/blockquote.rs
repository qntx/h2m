//! Blockquote (`<blockquote>`) conversion rule.

use scraper::ElementRef;

use crate::context::ConversionContext;
use crate::rule::{Rule, RuleAction};

/// Handles `<blockquote>` elements, including nested blockquotes.
#[derive(Debug, Clone, Copy)]
pub struct BlockquoteRule;

impl Rule for BlockquoteRule {
    fn tags(&self) -> &'static [&'static str] {
        &["blockquote"]
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

        // Prefix every line with "> ".
        let quoted: String = trimmed
            .lines()
            .map(|line| {
                if line.is_empty() {
                    ">".to_owned()
                } else {
                    format!("> {line}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        RuleAction::Replace(format!("\n\n{quoted}\n\n"))
    }
}
