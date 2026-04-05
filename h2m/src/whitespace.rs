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

/// Trims trailing whitespace from each line and collapses 3+ consecutive
/// newlines into exactly two. Returns the final output trimmed.
pub fn clean_output(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut consecutive_newlines = 0u32;

    for line in text.split('\n') {
        let trimmed = line.trim_end();

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
        }
    }

    // Trim in-place rather than allocating via `.trim().to_owned()`.
    let trimmed = result.trim();
    if trimmed.len() == result.len() {
        return result;
    }
    let start = result.len() - result.trim_start().len();
    let end = result.trim_end().len();
    result.truncate(end);
    result.drain(..start);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapse_preserves_single_spaces() {
        assert_eq!(collapse_whitespace("hello world"), "hello world");
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
    fn clean_output_collapses_newlines() {
        assert_eq!(clean_output("a\n\n\n\nb"), "a\n\nb");
    }

    #[test]
    fn clean_output_trims_trailing_spaces() {
        assert_eq!(clean_output("hello   \nworld  "), "hello\nworld");
    }
}
