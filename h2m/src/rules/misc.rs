//! Miscellaneous rules: horizontal rules, line breaks.

use scraper::ElementRef;

use crate::context::Context;
use crate::rule::{Action, Rule};
use crate::utils;

/// Handles `<hr>` elements.
///
/// Renders a thematic break, but suppresses it when the `<hr>` appears inside
/// a heading (which would look weird: `## --- Heading`).
#[derive(Debug, Clone, Copy)]
pub struct HorizontalRuleRule;

impl Rule for HorizontalRuleRule {
    fn tags(&self) -> &'static [&'static str] {
        &["hr"]
    }

    fn apply(&self, _content: &str, element: &ElementRef<'_>, ctx: &mut Context) -> Action {
        // Suppress hr inside headings.
        if utils::has_ancestor(element, "h1")
            || utils::has_ancestor(element, "h2")
            || utils::has_ancestor(element, "h3")
            || utils::has_ancestor(element, "h4")
            || utils::has_ancestor(element, "h5")
            || utils::has_ancestor(element, "h6")
        {
            return Action::Replace(String::new());
        }

        let rule = ctx.options().horizontal_rule;
        Action::Replace(format!("\n\n{rule}\n\n"))
    }
}

/// Handles `<br>` elements as `CommonMark` hard line breaks (`  \n`).
#[derive(Debug, Clone, Copy)]
pub struct LineBreakRule;

impl Rule for LineBreakRule {
    fn tags(&self) -> &'static [&'static str] {
        &["br"]
    }

    fn apply(&self, _content: &str, _element: &ElementRef<'_>, _ctx: &mut Context) -> Action {
        Action::Replace("  \n".to_owned())
    }
}
