//! Inline formatting rules: strong, emphasis, inline code.

use scraper::ElementRef;

use crate::context::Context;
use crate::rule::{Action, Rule};
use crate::utils;

/// Handles `<strong>` and `<b>` elements.
#[derive(Debug, Clone, Copy)]
pub struct StrongRule;

impl Rule for StrongRule {
    fn tags(&self) -> &'static [&'static str] {
        &["strong", "b"]
    }

    fn apply(&self, content: &str, element: &ElementRef<'_>, ctx: &mut Context) -> Action {
        // Nested dedup: if parent is also strong/b, just pass through.
        if utils::parent_tag_is(element, "strong") || utils::parent_tag_is(element, "b") {
            return Action::Replace(content.to_owned());
        }

        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Action::Skip;
        }

        let delim = ctx.options().strong_delimiter;
        let wrapped = delimiter_for_every_line(trimmed, delim);
        Action::Replace(utils::add_space_if_necessary(element, wrapped))
    }
}

/// Handles `<em>` and `<i>` elements.
#[derive(Debug, Clone, Copy)]
pub struct EmphasisRule;

impl Rule for EmphasisRule {
    fn tags(&self) -> &'static [&'static str] {
        &["em", "i"]
    }

    fn apply(&self, content: &str, element: &ElementRef<'_>, ctx: &mut Context) -> Action {
        // Nested dedup: if parent is also em/i, just pass through.
        if utils::parent_tag_is(element, "em") || utils::parent_tag_is(element, "i") {
            return Action::Replace(content.to_owned());
        }

        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Action::Skip;
        }

        let delim_char = ctx.options().em_delimiter;
        let delim = &String::from(delim_char);
        let wrapped = delimiter_for_every_line(trimmed, delim);
        Action::Replace(utils::add_space_if_necessary(element, wrapped))
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

    fn apply(&self, content: &str, element: &ElementRef<'_>, _ctx: &mut Context) -> Action {
        // If inside a <pre>, let the code block rule handle it.
        if utils::has_ancestor(element, "pre") {
            return Action::Skip;
        }

        if content.is_empty() {
            return Action::Skip;
        }

        // Limit consecutive newlines to one (inline code shouldn't have
        // multiple blank lines, and markdown parsers won't recognize it).
        let code = collapse_newlines(content);

        // Calculate fence: use enough backticks to exceed the longest run.
        let max_backtick_run = utils::max_consecutive_char(&code, '`');
        let fence_len = max_backtick_run + 1;
        let fence: String = std::iter::repeat_n('`', fence_len).collect();

        // If content starts or ends with a backtick, add a space for clarity.
        let (pad_start, pad_end) = if code.starts_with('`') || code.ends_with('`') {
            (" ", " ")
        } else {
            ("", "")
        };

        let text = format!("{fence}{pad_start}{code}{pad_end}{fence}");
        Action::Replace(utils::add_space_if_necessary(element, text))
    }
}

/// Wraps each non-empty line with `delimiter` so that bold/italic spanning
/// multiple lines is rendered correctly.
fn delimiter_for_every_line(text: &str, delimiter: &str) -> String {
    if !text.contains('\n') {
        return format!("{delimiter}{text}{delimiter}");
    }

    let mut result = String::with_capacity(text.len() + delimiter.len() * 4);
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            result.push('\n');
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        result.push_str(delimiter);
        result.push_str(trimmed);
        result.push_str(delimiter);
    }
    result
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
