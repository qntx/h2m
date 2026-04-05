//! Context-aware markdown character escaping.

use std::borrow::Cow;

use crate::options::EscapeMode;

/// Characters that are special in markdown and need escaping in normal text.
const MARKDOWN_SPECIAL: &[char] = &['\\', '*', '_', '`', '[', ']', '|', '<', '>'];

/// Escapes markdown special characters in text content.
///
/// Returns a borrowed [`Cow`] when no escaping is needed (common fast path).
pub fn escape_markdown(text: &str, mode: EscapeMode) -> Cow<'_, str> {
    if matches!(mode, EscapeMode::Disabled) {
        return Cow::Borrowed(text);
    }

    // Fast path: check if any escaping is needed at all.
    let needs_escape = text
        .chars()
        .any(|c| MARKDOWN_SPECIAL.contains(&c) || is_line_start_trigger(c));

    if !needs_escape {
        return Cow::Borrowed(text);
    }

    let mut result = String::with_capacity(text.len() + text.len() / 8);
    let mut chars = text.char_indices();

    while let Some((i, c)) = chars.next() {
        if MARKDOWN_SPECIAL.contains(&c) {
            result.push('\\');
            result.push(c);
        } else if i == 0 || text.as_bytes().get(i.wrapping_sub(1)) == Some(&b'\n') {
            // Check for line-start patterns that could be interpreted as
            // markdown structure.
            if let Some(escaped) = escape_line_start(&text[i..]) {
                result.push_str(&escaped);
                // Skip the characters we already handled.
                for _ in 0..escaped.len().saturating_sub(2) {
                    let _ = chars.next();
                }
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }

    Cow::Owned(result)
}

/// Checks if a character could begin a line-start markdown pattern.
#[inline]
const fn is_line_start_trigger(c: char) -> bool {
    matches!(c, '#' | '>' | '-' | '+') || c.is_ascii_digit()
}

/// If the text at the current position starts with a markdown structural
/// pattern, returns an escaped version.
fn escape_line_start(text: &str) -> Option<String> {
    // Heading: "# ", "## ", etc.
    if text.starts_with('#') {
        let hashes = text.chars().take_while(|&c| c == '#').count();
        if text.get(hashes..hashes + 1) == Some(" ") && hashes <= 6 {
            return Some(format!("\\{}", &text[..=hashes]));
        }
    }

    // Blockquote: "> "
    if text.starts_with("> ") {
        return Some("\\> ".to_owned());
    }

    // Unordered list: "- ", "+ "
    if text.starts_with("- ") || text.starts_with("+ ") {
        return Some(format!("\\{}", &text[..2]));
    }

    // Ordered list: "1. ", "2. ", etc.
    if let Some(first) = text.chars().next()
        && first.is_ascii_digit()
    {
        let digits: String = text.chars().take_while(char::is_ascii_digit).collect();
        let rest = &text[digits.len()..];
        if rest.starts_with(". ") || rest.starts_with(") ") {
            return Some(format!("{digits}\\{}", &rest[..2]));
        }
    }

    // Thematic breaks: "---", "***", "___"
    if text.starts_with("---") || text.starts_with("***") || text.starts_with("___") {
        return Some(format!("\\{}", &text[..1]));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_not_escaped() {
        let result = escape_markdown("hello world", EscapeMode::Basic);
        assert_eq!(result, "hello world");
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn special_chars_escaped() {
        let result = escape_markdown("a*b_c`d", EscapeMode::Basic);
        assert_eq!(result, "a\\*b\\_c\\`d");
    }

    #[test]
    fn disabled_mode_no_escape() {
        let result = escape_markdown("a*b", EscapeMode::Disabled);
        assert_eq!(result, "a*b");
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn brackets_escaped() {
        let result = escape_markdown("[link]", EscapeMode::Basic);
        assert_eq!(result, "\\[link\\]");
    }

    #[test]
    fn pipe_escaped() {
        let result = escape_markdown("a|b", EscapeMode::Basic);
        assert_eq!(result, "a\\|b");
    }
}
