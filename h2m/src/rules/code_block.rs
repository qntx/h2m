//! Code block (`<pre>`) conversion rule.

use std::fmt::Write;

use scraper::ElementRef;

use crate::context::Context;
use crate::options::CodeBlockStyle;
use crate::rule::{Action, Rule};
use crate::utils;

/// Handles `<pre>` elements (typically containing a `<code>` child).
#[derive(Debug, Clone, Copy)]
pub struct CodeBlockRule;

impl Rule for CodeBlockRule {
    fn tags(&self) -> &'static [&'static str] {
        &["pre"]
    }

    fn apply(&self, content: &str, element: &ElementRef<'_>, ctx: &Context) -> Action {
        match ctx.options().code_block_style {
            CodeBlockStyle::Fenced => Self::fenced(content, element, ctx),
            CodeBlockStyle::Indented => Self::indented(content),
        }
    }
}

impl CodeBlockRule {
    /// Renders a fenced code block.
    fn fenced(content: &str, element: &ElementRef<'_>, ctx: &Context) -> Action {
        let language = detect_language(element);
        let fence_char = ctx.options().fence.char();

        // Calculate fence length: must exceed longest consecutive run of the
        // fence character in the content.
        let max_run = utils::max_consecutive_char(content, fence_char);
        let fence_len = std::cmp::max(3, max_run + 1);
        let fence: String = std::iter::repeat_n(fence_char, fence_len).collect();

        let lang_tag = language.as_deref().unwrap_or_default();

        // Trim a single leading/trailing newline from content (html5ever often
        // leaves one).
        let trimmed = content
            .strip_prefix('\n')
            .unwrap_or(content)
            .strip_suffix('\n')
            .unwrap_or(content);

        Action::Replace(format!("\n\n{fence}{lang_tag}\n{trimmed}\n{fence}\n\n"))
    }

    /// Renders an indented code block (4-space indent).
    fn indented(content: &str) -> Action {
        let mut result = String::with_capacity(content.len() + content.lines().count() * 4 + 4);
        result.push_str("\n\n");
        for (i, line) in content.lines().enumerate() {
            if i > 0 {
                result.push('\n');
            }
            let _ = write!(result, "    {line}");
        }
        result.push_str("\n\n");
        Action::Replace(result)
    }
}

/// Attempts to detect the programming language from a `<code>` child's
/// `class` attribute (e.g., `class="language-rust"` or `class="lang-js"`).
fn detect_language(pre: &ElementRef<'_>) -> Option<String> {
    for child in pre.children() {
        if let Some(el) = child.value().as_element()
            && el.name() == "code"
            && let Some(class) = el.attr("class")
        {
            for cls in class.split_whitespace() {
                if let Some(lang) = cls
                    .strip_prefix("language-")
                    .or_else(|| cls.strip_prefix("lang-"))
                {
                    return Some(lang.to_owned());
                }
            }
            // Fall back to the first class as the language.
            return class
                .split_whitespace()
                .next()
                .filter(|f| !f.is_empty())
                .map(str::to_owned);
        }
    }
    None
}
