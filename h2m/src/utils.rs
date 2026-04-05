//! Shared utility functions.

/// Returns the length of the longest consecutive run of `needle` in `text`.
#[inline]
pub fn max_consecutive_char(text: &str, needle: char) -> usize {
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
