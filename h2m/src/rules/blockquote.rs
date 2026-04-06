//! Blockquote (`<blockquote>`) conversion rule.

use scraper::ElementRef;

use crate::context::Context;
use crate::converter::{Action, Rule};

/// Handles `<blockquote>` elements, including nested blockquotes.
#[derive(Debug, Clone, Copy)]
pub struct Blockquote;

impl Rule for Blockquote {
    fn tags(&self) -> &'static [&'static str] {
        &["blockquote"]
    }

    fn apply(&self, content: &str, _element: &ElementRef<'_>, _ctx: &mut Context<'_>) -> Action {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Action::Skip;
        }

        // Pre-allocate: each line gets "> " (2 chars) prefix.
        let mut quoted = String::with_capacity(trimmed.len() + trimmed.lines().count() * 2);
        for (i, line) in trimmed.lines().enumerate() {
            if i > 0 {
                quoted.push('\n');
            }
            if line.is_empty() {
                quoted.push('>');
            } else {
                quoted.push_str("> ");
                quoted.push_str(line);
            }
        }

        Action::Replace(format!("\n\n{quoted}\n\n"))
    }
}
