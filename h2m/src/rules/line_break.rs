//! Line break (`<br>`) conversion rule.

use scraper::ElementRef;

use crate::context::Context;
use crate::converter::{Action, Rule};

/// Handles `<br>` elements as `CommonMark` hard line breaks (`  \n`).
#[derive(Debug, Clone, Copy)]
pub struct LineBreak;

impl Rule for LineBreak {
    fn tags(&self) -> &'static [&'static str] {
        &["br"]
    }

    fn apply(&self, _content: &str, _element: &ElementRef<'_>, _ctx: &mut Context<'_>) -> Action {
        Action::Replace("  \n".to_owned())
    }
}
