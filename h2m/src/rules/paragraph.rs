//! Paragraph and generic block container rules.

use scraper::ElementRef;

use crate::context::ConversionContext;
use crate::rule::{Rule, RuleAction};

/// Handles `<p>`, `<div>`, `<section>`, `<article>`, `<main>`, `<header>`,
/// `<footer>`, and `<nav>` elements.
#[derive(Debug, Clone, Copy)]
pub struct ParagraphRule;

impl Rule for ParagraphRule {
    fn tags(&self) -> &'static [&'static str] {
        &[
            "p", "div", "section", "article", "main", "header", "footer", "nav",
        ]
    }

    fn apply(
        &self,
        content: &str,
        _element: &ElementRef<'_>,
        _ctx: &ConversionContext,
    ) -> RuleAction {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return RuleAction::Skip;
        }
        RuleAction::Replace(format!("\n\n{trimmed}\n\n"))
    }
}
