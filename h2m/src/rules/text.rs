//! Text node handling (not a tag-based rule; handled directly in the
//! converter's `process_text` method).
//!
//! This module is intentionally empty. Text nodes are processed by the
//! converter pipeline using [`crate::whitespace`] and [`crate::escape`].
