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
        builder.add_rule(paragraph::ParagraphRule);
        builder.add_rule(heading::HeadingRule);
        builder.add_rule(code_block::CodeBlockRule);
        builder.add_rule(blockquote::BlockquoteRule);
        builder.add_rule(list::ListRule);
        builder.add_rule(list::ListItemRule);
        builder.add_rule(misc::HorizontalRuleRule);
        builder.add_rule(misc::LineBreakRule);
        builder.add_rule(inline::StrongRule);
        builder.add_rule(inline::EmphasisRule);
        builder.add_rule(inline::InlineCodeRule);
        builder.add_rule(link::LinkRule);
        builder.add_rule(image::ImageRule);
    }
}
