//! Miscellaneous rules: horizontal rules, line breaks.

use scraper::ElementRef;

use crate::context::Context;
use crate::rule::{Action, Rule};

/// Handles `<hr>` elements.
#[derive(Debug, Clone, Copy)]
pub struct HorizontalRuleRule;

impl Rule for HorizontalRuleRule {
    fn tags(&self) -> &'static [&'static str] {
        &["hr"]
    }

    fn apply(&self, _content: &str, _element: &ElementRef<'_>, ctx: &Context) -> Action {
        let rule = ctx.options().horizontal_rule;
        Action::Replace(format!("\n\n{rule}\n\n"))
    }
}

/// Handles `<br>` elements.
#[derive(Debug, Clone, Copy)]
pub struct LineBreakRule;

impl Rule for LineBreakRule {
    fn tags(&self) -> &'static [&'static str] {
        &["br"]
    }

    fn apply(&self, _content: &str, _element: &ElementRef<'_>, _ctx: &Context) -> Action {
        Action::Replace("  \n".to_owned())
    }
}
