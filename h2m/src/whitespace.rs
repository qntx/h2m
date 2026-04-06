//! Whitespace normalization utilities.

use std::borrow::Cow;

/// Collapses runs of whitespace (spaces, tabs, newlines) into a single space.
///
/// This mirrors browser rendering of normal-flow text content. Returns a
/// borrowed [`Cow`] when no collapsing is needed.
pub fn collapse_whitespace(text: &str) -> Cow<'_, str> {
    if !text.contains(|c: char| c.is_whitespace() && c != ' ') && !text.contains("  ") {
        return Cow::Borrowed(text);
    }

    let mut result = String::with_capacity(text.len());
    let mut prev_ws = false;

    for c in text.chars() {
        if c.is_whitespace() {
            if !prev_ws {
                result.push(' ');
            }
            prev_ws = true;
        } else {
            result.push(c);
            prev_ws = false;
        }
    }

    Cow::Owned(result)
}

/// Cleans up raw converter output:
///
/// - Collapses 3+ consecutive newlines into exactly two.
/// - Trims trailing whitespace from each line, **except** markdown hard breaks
///   (2+ trailing spaces before a newline are preserved as exactly two spaces).
/// - Strips leading newlines and trailing whitespace from the final result,
///   while preserving indentation on the first content line.
pub fn clean_output(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut consecutive_newlines = 0u32;

    for line in text.split('\n') {
        let trimmed = line.trim_end();
        let trailing_spaces = line.len() - trimmed.len();
        let hard_break = trailing_spaces >= 2;

        if trimmed.is_empty() {
            consecutive_newlines += 1;
            if consecutive_newlines <= 2 {
                result.push('\n');
            }
        } else {
            consecutive_newlines = 0;
            if !result.is_empty() && !result.ends_with('\n') {
                result.push('\n');
            }
            result.push_str(trimmed);
            if hard_break {
                result.push_str("  ");
            }
        }
    }

    // Strip leading newlines (but preserve indentation of first content line).
    let start = result.bytes().take_while(|&b| b == b'\n').count();
    if start > 0 {
        result.drain(..start);
    }
    // Strip trailing whitespace.
    let end = result.trim_end().len();
    result.truncate(end);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapse_empty() {
        let r = collapse_whitespace("");
        assert_eq!(r, "");
        assert!(matches!(r, Cow::Borrowed(_)));
    }

    #[test]
    fn collapse_no_op_borrowed() {
        let r = collapse_whitespace("hello world");
        assert_eq!(r, "hello world");
        assert!(matches!(r, Cow::Borrowed(_)));
    }

    #[test]
    fn collapse_multiple_spaces() {
        assert_eq!(collapse_whitespace("hello   world"), "hello world");
    }

    #[test]
    fn collapse_tabs_and_newlines() {
        assert_eq!(collapse_whitespace("hello\t\n  world"), "hello world");
    }

    #[test]
    fn collapse_leading_and_trailing() {
        assert_eq!(collapse_whitespace("  hello  "), " hello ");
    }

    #[test]
    fn clean_empty() {
        assert_eq!(clean_output(""), "");
    }

    #[test]
    fn clean_collapses_triple_newlines() {
        assert_eq!(clean_output("a\n\n\n\nb"), "a\n\nb");
    }

    #[test]
    fn clean_trims_single_trailing_space() {
        assert_eq!(clean_output("hello \nworld"), "hello\nworld");
    }

    #[test]
    fn clean_preserves_hard_break() {
        assert_eq!(clean_output("hello  \nworld"), "hello  \nworld");
    }

    #[test]
    fn clean_normalizes_hard_break_to_two_spaces() {
        assert_eq!(clean_output("hello     \nworld"), "hello  \nworld");
    }

    #[test]
    fn clean_preserves_leading_indentation() {
        assert_eq!(
            clean_output("\n\n    line1\n    line2\n\n"),
            "    line1\n    line2"
        );
    }

    #[test]
    fn clean_strips_leading_newlines_only() {
        assert_eq!(clean_output("\n\ntext"), "text");
    }

    #[test]
    fn clean_strips_trailing_whitespace() {
        assert_eq!(clean_output("text\n\n"), "text");
    }
}
