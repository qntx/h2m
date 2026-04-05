//! Miscellaneous rules: horizontal rules, line breaks, etc.

use scraper::ElementRef;

use crate::context::ConversionContext;
use crate::rule::{Rule, RuleAction};

/// Handles `<hr>` elements.
#[derive(Debug, Clone, Copy)]
pub struct HorizontalRuleRule;

impl Rule for HorizontalRuleRule {
    fn tags(&self) -> &'static [&'static str] {
        &["hr"]
    }

    fn apply(
        &self,
        _content: &str,
        _element: &ElementRef<'_>,
        ctx: &ConversionContext,
    ) -> RuleAction {
        let rule = ctx.options().horizontal_rule;
        RuleAction::Replace(format!("\n\n{rule}\n\n"))
    }
}

/// Handles `<br>` elements.
#[derive(Debug, Clone, Copy)]
pub struct LineBreakRule;

impl Rule for LineBreakRule {
    fn tags(&self) -> &'static [&'static str] {
        &["br"]
    }

    fn apply(
        &self,
        _content: &str,
        _element: &ElementRef<'_>,
        _ctx: &ConversionContext,
    ) -> RuleAction {
        // Two trailing spaces + newline for a hard break, or just newline.
        RuleAction::Replace("  \n".to_owned())
    }
}
