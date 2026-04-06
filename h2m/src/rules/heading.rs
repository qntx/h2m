//! Heading rules for `<h1>` through `<h6>`.

use scraper::ElementRef;

use crate::context::Context;
use crate::converter::{Action, Rule};
use crate::dom;
use crate::options::HeadingStyle;

/// ATX heading prefixes indexed by level (0-indexed, level 1 = index 0).
const ATX_PREFIXES: [&str; 6] = ["#", "##", "###", "####", "#####", "######"];

/// Handles `<h1>` through `<h6>` elements.
#[derive(Debug, Clone, Copy)]
pub struct Heading;

impl Rule for Heading {
    fn tags(&self) -> &'static [&'static str] {
        &["h1", "h2", "h3", "h4", "h5", "h6"]
    }

    fn apply(&self, content: &str, element: &ElementRef<'_>, ctx: &mut Context) -> Action {
        let tag = element.value().name();
        let level = heading_level(tag);

        // Normalize: collapse newlines/carriage-returns to spaces, escape `#`.
        let trimmed = content
            .trim()
            .replace(['\n', '\r'], " ")
            .replace('#', "\\#");

        if trimmed.is_empty() {
            return Action::Skip;
        }

        // If the heading is inside an <a> link, render as bold instead.
        if dom::has_ancestor(element, "a") {
            let delim = ctx.options().strong_delimiter;
            let text = format!("{delim}{trimmed}{delim}");
            return Action::Replace(dom::add_space_if_necessary(element, text));
        }

        let md = match ctx.options().heading_style {
            HeadingStyle::Setext if level <= 2 => {
                let underline_char = if level == 1 { '=' } else { '-' };
                let underline =
                    std::iter::repeat_n(underline_char, trimmed.len()).collect::<String>();
                format!("\n\n{trimmed}\n{underline}\n\n")
            }
            _ => {
                let prefix = ATX_PREFIXES[level - 1];
                format!("\n\n{prefix} {trimmed}\n\n")
            }
        };

        Action::Replace(md)
    }
}

/// Extracts the heading level (1-6) from a tag name like `"h2"`.
#[inline]
fn heading_level(tag: &str) -> usize {
    tag.as_bytes()
        .get(1)
        .and_then(|&b| {
            if b.is_ascii_digit() {
                Some((b - b'0') as usize)
            } else {
                None
            }
        })
        .unwrap_or(1)
}
