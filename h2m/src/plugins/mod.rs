//! Plugins for extending the converter with additional format support.

pub mod gfm;
pub(crate) mod strikethrough;
pub(crate) mod table;
pub(crate) mod task_list;

pub use gfm::Gfm;
