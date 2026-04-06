//! Link (`<a>`) conversion rule.

use scraper::ElementRef;

use crate::context::Context;
use crate::converter::{Action, Rule};
use crate::dom;
use crate::options::{LinkReferenceStyle, LinkStyle};

/// Handles `<a>` elements with support for inline and reference-style links.
#[derive(Debug, Clone, Copy)]
pub struct Link;

impl Rule for Link {
    fn tags(&self) -> &'static [&'static str] {
        &["a"]
    }

    fn apply(&self, content: &str, element: &ElementRef<'_>, ctx: &mut Context) -> Action {
        let href = dom::attr(element, "href").unwrap_or("");

        // Skip non-link anchors.
        if href.is_empty() || href.trim() == "#" {
            return Action::Replace(content.to_owned());
        }

        let absolute_href = ctx.resolve_url(href);

        // Multiline content: escape newlines so the link text stays on one
        // logical line.
        let escaped_content = escape_multiline(content);

        // Title attribute (escape internal quotes).
        let title = dom::attr(element, "title").map(|t| t.replace('\n', " ").replace('"', "\\\""));

        // If content is empty, fall back to title or aria-label attribute.
        let display = if escaped_content.trim().is_empty() {
            let fallback = dom::attr(element, "title")
                .or_else(|| dom::attr(element, "aria-label"))
                .unwrap_or("")
                .to_owned();
            if fallback.is_empty() {
                return Action::Replace(String::new());
            }
            fallback
        } else {
            escaped_content
        };

        // If the content is a markdown image whose src matches the link href,
        // return just the image to avoid redundant `[![](url)](url)`.
        let trimmed_display = display.trim();
        if title.is_none()
            && trimmed_display.starts_with("![")
            && let Some(img_url) = extract_markdown_image_url(trimmed_display)
            && img_url == absolute_href
        {
            return Action::Replace(dom::add_space_if_necessary(
                element,
                trimmed_display.to_owned(),
            ));
        }

        let title_part = title.map_or_else(String::new, |t| format!(" \"{t}\""));

        let md = match ctx.options().link_style {
            LinkStyle::Inlined => {
                format!("[{display}]({absolute_href}{title_part})")
            }
            LinkStyle::Referenced => {
                build_reference_link(&display, &absolute_href, &title_part, ctx)
            }
        };

        Action::Replace(dom::add_space_if_necessary(element, md))
    }
}

/// Builds a reference-style link and pushes the definition into `ctx`.
fn build_reference_link(display: &str, href: &str, title_part: &str, ctx: &mut Context) -> String {
    match ctx.options().link_reference_style {
        LinkReferenceStyle::Full => {
            let idx = ctx.link_index + 1;
            ctx.push_reference(format!("[{idx}]: {href}{title_part}"));
            format!("[{display}][{idx}]")
        }
        LinkReferenceStyle::Collapsed => {
            ctx.push_reference(format!("[{display}]: {href}{title_part}"));
            format!("[{display}][]")
        }
        LinkReferenceStyle::Shortcut => {
            ctx.push_reference(format!("[{display}]: {href}{title_part}"));
            format!("[{display}]")
        }
    }
}

/// Escapes newlines in link content so multi-line text works inside `[...]`.
fn escape_multiline(content: &str) -> String {
    let trimmed = content.trim();
    if !trimmed.contains('\n') {
        return trimmed.to_owned();
    }
    trimmed.replace('\n', "\\\n")
}

/// Extracts the URL from a markdown image `![alt](url)`.
fn extract_markdown_image_url(md: &str) -> Option<&str> {
    let rest = md.strip_prefix("![")?;
    let after_alt = rest.find("](")?;
    let url_start = after_alt + 2;
    let url_part = &rest[url_start..];
    let end = url_part.find(')')?;
    let url = url_part[..end].trim();
    // Strip optional title in quotes.
    url.find([' ', '\t'])
        .map_or(Some(url), |idx| Some(url[..idx].trim()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_multiline_no_newlines() {
        assert_eq!(escape_multiline("hello"), "hello");
    }

    #[test]
    fn escape_multiline_with_newlines() {
        assert_eq!(escape_multiline("line1\nline2"), "line1\\\nline2");
    }

    #[test]
    fn escape_multiline_trims_whitespace() {
        assert_eq!(escape_multiline("  hello  "), "hello");
    }

    #[test]
    fn extract_image_url_basic() {
        assert_eq!(
            extract_markdown_image_url("![alt](https://example.com/img.png)"),
            Some("https://example.com/img.png")
        );
    }

    #[test]
    fn extract_image_url_with_title() {
        assert_eq!(
            extract_markdown_image_url(r#"![alt](img.png "title")"#),
            Some("img.png")
        );
    }

    #[test]
    fn extract_image_url_empty_alt() {
        assert_eq!(extract_markdown_image_url("![](img.png)"), Some("img.png"));
    }

    #[test]
    fn extract_image_url_not_image() {
        assert_eq!(extract_markdown_image_url("[text](url)"), None);
    }

    #[test]
    fn extract_image_url_no_closing_paren() {
        assert_eq!(extract_markdown_image_url("![alt](img.png"), None);
    }
}
