//! Inline code (`<code>`, `<kbd>`, `<samp>`, `<tt>`) conversion rule.

use scraper::ElementRef;

use crate::context::Context;
use crate::converter::{Action, Rule};
use crate::dom;

/// Handles `<code>`, `<kbd>`, `<samp>`, and `<tt>` inline code elements.
///
/// Does **not** handle `<code>` inside `<pre>` — that is handled by
/// [`super::code_block::CodeBlock`].
#[derive(Debug, Clone, Copy)]
pub struct InlineCode;

impl Rule for InlineCode {
    fn tags(&self) -> &'static [&'static str] {
        &["code", "kbd", "samp", "tt"]
    }

    fn apply(&self, content: &str, element: &ElementRef<'_>, _ctx: &mut Context) -> Action {
        // If inside a <pre>, let the code block rule handle it.
        if dom::has_ancestor(element, "pre") {
            return Action::Skip;
        }

        if content.is_empty() {
            return Action::Skip;
        }

        // Limit consecutive newlines to one (inline code shouldn't have
        // multiple blank lines, and markdown parsers won't recognize it).
        let code = collapse_newlines(content);

        // Calculate fence: use enough backticks to exceed the longest run.
        let max_backtick_run = dom::max_consecutive_char(&code, '`');
        let fence_len = max_backtick_run + 1;
        let fence: String = std::iter::repeat_n('`', fence_len).collect();

        // If content starts or ends with a backtick, add a space for clarity.
        let (pad_start, pad_end) = if code.starts_with('`') || code.ends_with('`') {
            (" ", " ")
        } else {
            ("", "")
        };

        let text = format!("{fence}{pad_start}{code}{pad_end}{fence}");
        Action::Replace(dom::add_space_if_necessary(element, text))
    }
}

/// Collapses runs of 2+ newlines into a single newline.
fn collapse_newlines(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut prev_newline = false;
    for c in text.chars() {
        if c == '\n' {
            if !prev_newline {
                result.push('\n');
            }
            prev_newline = true;
        } else {
            result.push(c);
            prev_newline = false;
        }
    }
    result
}
