//! Shared utility functions.

use scraper::ElementRef;

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

/// Returns the value of an attribute on an element.
#[inline]
#[must_use]
pub fn attr<'a>(element: &'a ElementRef<'_>, name: &str) -> Option<&'a str> {
    element.value().attr(name)
}

/// Returns `true` if the given element has an ancestor with the specified tag
/// name.
#[must_use]
pub fn has_ancestor(element: &ElementRef<'_>, target_tag: &str) -> bool {
    let mut current = element.parent();
    while let Some(parent) = current {
        if let Some(el) = parent.value().as_element()
            && el.name() == target_tag
        {
            return true;
        }
        current = parent.parent();
    }
    false
}
