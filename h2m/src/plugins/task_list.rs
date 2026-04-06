//! Task list (`<input type="checkbox">`) conversion rule.

use scraper::ElementRef;

use crate::context::Context;
use crate::rule::{Action, Rule};
use crate::utils;

/// Handles `<input type="checkbox">` elements inside list items.
#[derive(Debug, Clone, Copy)]
pub struct TaskListRule;

impl Rule for TaskListRule {
    fn tags(&self) -> &'static [&'static str] {
        &["input"]
    }

    fn apply(&self, _content: &str, element: &ElementRef<'_>, _ctx: &Context) -> Action {
        let is_checkbox =
            utils::attr(element, "type").is_some_and(|t| t.eq_ignore_ascii_case("checkbox"));

        if !is_checkbox {
            return Action::Skip;
        }

        // The trailing space is typically provided by whitespace in the HTML.
        let checked = element.value().attr("checked").is_some();
        if checked {
            Action::Replace("[x]".to_owned())
        } else {
            Action::Replace("[ ]".to_owned())
        }
    }
}
