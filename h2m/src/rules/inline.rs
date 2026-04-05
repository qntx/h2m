//! Inline formatting rules: strong, emphasis, inline code.

use scraper::ElementRef;

use crate::context::ConversionContext;
use crate::rule::{Rule, RuleAction};

/// Handles `<strong>` and `<b>` elements.
#[derive(Debug, Clone, Copy)]
pub struct StrongRule;

impl Rule for StrongRule {
    fn tags(&self) -> &'static [&'static str] {
        &["strong", "b"]
    }

    fn apply(
        &self,
        content: &str,
        _element: &ElementRef<'_>,
        ctx: &ConversionContext,
    ) -> RuleAction {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return RuleAction::Skip;
        }

        let delim = ctx.options().strong_delimiter;

        // Preserve leading/trailing whitespace outside the delimiter.
        let leading = if content.starts_with(' ') { " " } else { "" };
        let trailing = if content.ends_with(' ') { " " } else { "" };

        RuleAction::Replace(format!("{leading}{delim}{trimmed}{delim}{trailing}"))
    }
}

/// Handles `<em>` and `<i>` elements.
#[derive(Debug, Clone, Copy)]
pub struct EmphasisRule;

impl Rule for EmphasisRule {
    fn tags(&self) -> &'static [&'static str] {
        &["em", "i"]
    }

    fn apply(
        &self,
        content: &str,
        _element: &ElementRef<'_>,
        ctx: &ConversionContext,
    ) -> RuleAction {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return RuleAction::Skip;
        }

        let delim = ctx.options().em_delimiter;

        let leading = if content.starts_with(' ') { " " } else { "" };
        let trailing = if content.ends_with(' ') { " " } else { "" };

        RuleAction::Replace(format!("{leading}{delim}{trimmed}{delim}{trailing}"))
    }
}

/// Handles `<code>`, `<kbd>`, `<samp>`, and `<tt>` inline code elements.
///
/// Does **not** handle `<code>` inside `<pre>` — that is handled by
/// [`super::code_block::CodeBlockRule`].
#[derive(Debug, Clone, Copy)]
pub struct InlineCodeRule;

impl Rule for InlineCodeRule {
    fn tags(&self) -> &'static [&'static str] {
        &["code", "kbd", "samp", "tt"]
    }

    fn apply(
        &self,
        content: &str,
        element: &ElementRef<'_>,
        _ctx: &ConversionContext,
    ) -> RuleAction {
        // If inside a <pre>, let the code block rule handle it.
        if crate::context::has_ancestor(element, "pre") {
            return RuleAction::Skip;
        }

        if content.is_empty() {
            return RuleAction::Skip;
        }

        // Calculate fence: use enough backticks to exceed the longest run in
        // the content.
        let max_backtick_run = max_consecutive_char(content, '`');
        let fence_len = max_backtick_run + 1;
        let fence: String = std::iter::repeat_n('`', fence_len).collect();

        // If content starts or ends with a backtick, add a space for clarity.
        let (pad_start, pad_end) = if content.starts_with('`') || content.ends_with('`') {
            (" ", " ")
        } else {
            ("", "")
        };

        RuleAction::Replace(format!("{fence}{pad_start}{content}{pad_end}{fence}"))
    }
}

/// Returns the length of the longest consecutive run of `needle` in `text`.
fn max_consecutive_char(text: &str, needle: char) -> usize {
    let mut max = 0usize;
    let mut current = 0usize;
    for c in text.chars() {
        if c == needle {
            current += 1;
            if current > max {
                max = current;
            }
        } else {
            current = 0;
        }
    }
    max
}
