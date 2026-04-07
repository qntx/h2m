//! Iframe (`<iframe>`) conversion rule.

use scraper::ElementRef;

use crate::context::Context;
use crate::converter::{Action, Rule};
use crate::dom;

/// Handles `<iframe>` elements by converting them to markdown links.
///
/// For `data:text/html` iframes, the embedded content is skipped (would
/// require a nested converter). For regular iframes, the `src` is rendered
/// as a link.
#[derive(Debug, Clone, Copy)]
pub(super) struct Iframe;

impl Rule for Iframe {
    fn tags(&self) -> &'static [&'static str] {
        &["iframe"]
    }

    fn apply(&self, _content: &str, element: &ElementRef<'_>, ctx: &mut Context<'_>) -> Action {
        let Some(src) = dom::attr(element, "src") else {
            return Action::Replace(String::new());
        };

        if src.trim().is_empty() {
            return Action::Replace(String::new());
        }

        // Skip data URIs (would need nested HTML parsing).
        if src.starts_with("data:") {
            return Action::Replace(String::new());
        }

        let absolute_url = ctx.resolve_url(src);
        Action::Replace(format!("[iframe]({absolute_url})"))
    }
}
