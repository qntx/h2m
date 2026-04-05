//! Task list (`<input type="checkbox">`) conversion rule.

use scraper::ElementRef;

use crate::context::{self as ctx, ConversionContext};
use crate::rule::{Rule, RuleAction};

/// Handles `<input type="checkbox">` elements inside list items.
#[derive(Debug, Clone, Copy)]
pub struct TaskListRule;

impl Rule for TaskListRule {
    fn tags(&self) -> &'static [&'static str] {
        &["input"]
    }

    fn apply(
        &self,
        _content: &str,
        element: &ElementRef<'_>,
        _ctx: &ConversionContext,
    ) -> RuleAction {
        // Only handle checkboxes inside list items.
        let is_checkbox =
            ctx::attr(element, "type").is_some_and(|t| t.eq_ignore_ascii_case("checkbox"));

        if !is_checkbox {
            return RuleAction::Skip;
        }

        // The trailing space after the checkbox marker is typically
        // provided by whitespace in the HTML between `<input>` and the
        // text node. We only emit the marker itself to avoid double
        // spacing.
        let checked = element.value().attr("checked").is_some();
        if checked {
            RuleAction::Replace("[x]".to_owned())
        } else {
            RuleAction::Replace("[ ]".to_owned())
        }
    }
}
