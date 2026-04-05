//! Heading rules for `<h1>` through `<h6>`.

use scraper::ElementRef;

use crate::context::ConversionContext;
use crate::options::HeadingStyle;
use crate::rule::{Rule, RuleAction};

/// Handles `<h1>` through `<h6>` elements.
#[derive(Debug, Clone, Copy)]
pub struct HeadingRule;

impl Rule for HeadingRule {
    fn tags(&self) -> &'static [&'static str] {
        &["h1", "h2", "h3", "h4", "h5", "h6"]
    }

    fn apply(
        &self,
        content: &str,
        element: &ElementRef<'_>,
        ctx: &ConversionContext,
    ) -> RuleAction {
        let tag = element.value().name();
        let level = heading_level(tag);
        let trimmed = content.trim().replace('\n', " ");

        if trimmed.is_empty() {
            return RuleAction::Skip;
        }

        let md = match ctx.options().heading_style {
            HeadingStyle::Setext if level <= 2 => {
                let underline_char = if level == 1 { '=' } else { '-' };
                let underline =
                    std::iter::repeat_n(underline_char, trimmed.len()).collect::<String>();
                format!("\n\n{trimmed}\n{underline}\n\n")
            }
            _ => {
                let hashes = "#".repeat(level);
                format!("\n\n{hashes} {trimmed}\n\n")
            }
        };

        RuleAction::Replace(md)
    }
}

/// Extracts the heading level (1–6) from a tag name like `"h2"`.
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
