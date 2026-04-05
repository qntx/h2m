//! GitHub Flavored Markdown (GFM) plugin.
//!
//! Bundles table, strikethrough, and task list support.

use super::strikethrough::StrikethroughRule;
use super::table::{TableCellRule, TableRowRule, TableRule, TableSectionRule};
use super::task_list::TaskListRule;
use crate::converter::ConverterBuilder;
use crate::plugin::Plugin;

/// GFM plugin — adds table, strikethrough, and task list rules.
///
/// # Example
///
/// ```
/// use h2m::{Converter, Options};
/// use h2m::plugins::Gfm;
/// use h2m::rules::CommonMark;
///
/// let converter = Converter::builder()
///     .use_plugin(CommonMark)
///     .use_plugin(Gfm)
///     .build();
///
/// let md = converter.convert("<del>removed</del>").unwrap();
/// assert_eq!(md, "~~removed~~");
/// ```
#[derive(Debug, Clone, Copy)]
#[allow(clippy::exhaustive_structs)]
pub struct Gfm;

impl Plugin for Gfm {
    fn register(&self, builder: &mut ConverterBuilder) {
        builder.add_rule(TableRule);
        builder.add_rule(TableSectionRule);
        builder.add_rule(TableRowRule);
        builder.add_rule(TableCellRule);
        builder.add_rule(StrikethroughRule);
        builder.add_rule(TaskListRule);
    }
}
