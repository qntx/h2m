//! Plugins for extending the converter with additional format support.

pub(crate) mod strikethrough;
pub(crate) mod table;
pub(crate) mod task_list;

use strikethrough::Strikethrough;
use table::{TableCellRule, TableRowRule, TableRule, TableSectionRule};
use task_list::TaskList;

use crate::converter::{ConverterBuilder, Plugin};

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
/// let md = converter.convert("<del>removed</del>");
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
        builder.add_rule(Strikethrough);
        builder.add_rule(TaskList);
    }
}
