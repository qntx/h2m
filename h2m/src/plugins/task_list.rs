//! Task list (`<input type="checkbox">`) conversion rule.

use scraper::ElementRef;

use crate::context::Context;
use crate::converter::{Action, Rule};
use crate::dom;

/// Handles `<input type="checkbox">` elements inside list items.
#[derive(Debug, Clone, Copy)]
pub struct TaskList;

impl Rule for TaskList {
    fn tags(&self) -> &'static [&'static str] {
        &["input"]
    }

    fn apply(&self, _content: &str, element: &ElementRef<'_>, _ctx: &mut Context) -> Action {
        let is_checkbox =
            dom::attr(element, "type").is_some_and(|t| t.eq_ignore_ascii_case("checkbox"));

        if !is_checkbox {
            return Action::Skip;
        }

        // Only convert checkboxes that are direct children of a <li>.
        if !dom::parent_tag_is(element, "li") {
            return Action::Skip;
        }

        let checked = element.value().attr("checked").is_some();
        if checked {
            Action::Replace("[x]".to_owned())
        } else {
            Action::Replace("[ ]".to_owned())
        }
    }
}
