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
    if text.as_bytes().first().is_some_and(u8::is_ascii_digit) {
        let digit_len = text.bytes().take_while(u8::is_ascii_digit).count();
        let rest = &text[digit_len..];
        if rest.starts_with(". ") || rest.starts_with(") ") {
            return Some(format!("{}\\{}", &text[..digit_len], &rest[..2]));
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
    fn empty_string_borrowed() {
        let r = escape_markdown("", EscapeMode::Basic);
        assert_eq!(r, "");
        assert!(matches!(r, Cow::Borrowed(_)));
    }

    #[test]
    fn plain_text_borrowed() {
        let r = escape_markdown("hello world", EscapeMode::Basic);
        assert_eq!(r, "hello world");
        assert!(matches!(r, Cow::Borrowed(_)));
    }

    #[test]
    fn disabled_mode_preserves_all() {
        let r = escape_markdown("*bold* [link] <html>", EscapeMode::Disabled);
        assert_eq!(r, "*bold* [link] <html>");
        assert!(matches!(r, Cow::Borrowed(_)));
    }

    #[test]
    fn backslash() {
        assert_eq!(escape_markdown("a\\b", EscapeMode::Basic), "a\\\\b");
    }

    #[test]
    fn asterisk_underscore_backtick() {
        assert_eq!(
            escape_markdown("a*b_c`d", EscapeMode::Basic),
            "a\\*b\\_c\\`d"
        );
    }

    #[test]
    fn brackets() {
        assert_eq!(escape_markdown("[link]", EscapeMode::Basic), "\\[link\\]");
    }

    #[test]
    fn angle_brackets() {
        assert_eq!(escape_markdown("<tag>", EscapeMode::Basic), "\\<tag\\>");
    }

    #[test]
    fn pipe() {
        assert_eq!(escape_markdown("a|b", EscapeMode::Basic), "a\\|b");
    }

    #[test]
    fn heading_at_line_start() {
        assert_eq!(escape_markdown("# h1", EscapeMode::Basic), "\\# h1");
        assert_eq!(escape_markdown("## h2", EscapeMode::Basic), "\\## h2");
    }

    #[test]
    fn heading_after_newline() {
        assert_eq!(
            escape_markdown("text\n## sub", EscapeMode::Basic),
            "text\n\\## sub"
        );
    }

    #[test]
    fn blockquote_at_line_start() {
        assert_eq!(escape_markdown("> quote", EscapeMode::Basic), "\\> quote");
    }

    #[test]
    fn unordered_list_at_line_start() {
        assert_eq!(escape_markdown("- item", EscapeMode::Basic), "\\- item");
        assert_eq!(escape_markdown("+ item", EscapeMode::Basic), "\\+ item");
    }

    #[test]
    fn ordered_list_at_line_start() {
        assert_eq!(escape_markdown("1. first", EscapeMode::Basic), "1\\. first");
    }

    #[test]
    fn thematic_break_at_line_start() {
        assert_eq!(escape_markdown("---", EscapeMode::Basic), "\\---");
    }

    #[test]
    fn hash_mid_line_not_escaped() {
        assert_eq!(escape_markdown("C# lang", EscapeMode::Basic), "C# lang");
    }

    #[test]
    fn dash_mid_line_not_escaped() {
        assert_eq!(escape_markdown("a - b", EscapeMode::Basic), "a - b");
    }
}
