//! Plugin trait for composable converter extensions.

use crate::converter::ConverterBuilder;

/// A plugin that registers rules and hooks with a converter.
///
/// Plugins provide a composable way to extend the converter with additional
/// tag handlers. For example, the GFM plugin bundles table, strikethrough,
/// and task list support.
pub trait Plugin {
    /// Registers this plugin's rules and hooks with the given builder.
    fn register(&self, builder: &mut ConverterBuilder);
}
