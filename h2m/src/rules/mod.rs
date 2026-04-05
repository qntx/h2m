//! Built-in `CommonMark` conversion rules.

mod blockquote;
mod code_block;
mod heading;
mod image;
mod inline;
mod link;
mod list;
mod misc;
mod paragraph;
mod text;

use crate::converter::ConverterBuilder;
use crate::plugin::Plugin;

/// The `CommonMark` plugin — registers all built-in rules for standard HTML
/// tags.
///
/// This plugin handles headings, paragraphs, emphasis, strong, inline code,
/// code blocks, links, images, lists, blockquotes, horizontal rules, and
/// line breaks.
#[derive(Debug, Clone, Copy)]
#[allow(clippy::exhaustive_structs)]
pub struct CommonMark;

impl Plugin for CommonMark {
    fn register(&self, builder: &mut ConverterBuilder) {
        // Block-level rules.
        builder.add_rule_boxed(Box::new(paragraph::ParagraphRule));
        builder.add_rule_boxed(Box::new(heading::HeadingRule));
        builder.add_rule_boxed(Box::new(code_block::CodeBlockRule));
        builder.add_rule_boxed(Box::new(blockquote::BlockquoteRule));
        builder.add_rule_boxed(Box::new(list::ListRule));
        builder.add_rule_boxed(Box::new(list::ListItemRule));
        builder.add_rule_boxed(Box::new(misc::HorizontalRuleRule));
        builder.add_rule_boxed(Box::new(misc::LineBreakRule));

        // Inline rules.
        builder.add_rule_boxed(Box::new(inline::StrongRule));
        builder.add_rule_boxed(Box::new(inline::EmphasisRule));
        builder.add_rule_boxed(Box::new(inline::InlineCodeRule));
        builder.add_rule_boxed(Box::new(link::LinkRule));
        builder.add_rule_boxed(Box::new(image::ImageRule));
    }
}
