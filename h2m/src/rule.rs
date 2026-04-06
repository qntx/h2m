//! Rule trait and action types for HTML element conversion.

use crate::context::Context;

/// The action a rule returns to control how an element is converted.
#[derive(Debug)]
#[non_exhaustive]
pub enum Action {
    /// Replace the element with the given markdown string.
    Replace(String),
    /// Skip this rule and try the next registered rule for this tag.
    Skip,
    /// Remove the element and all its content from the output.
    Remove,
}

/// A conversion rule that handles one or more HTML tag types.
///
/// Rules are registered with the converter and dispatched by tag name.
/// Multiple rules can be registered for the same tag; they are tried in
/// reverse registration order (last-registered first). The first rule that
/// returns [`Action::Replace`] wins.
pub trait Rule: Send + Sync {
    /// Returns the HTML tag names this rule handles.
    fn tags(&self) -> &'static [&'static str];

    /// Applies this rule to an element.
    ///
    /// # Arguments
    ///
    /// * `content` - The already-converted markdown content of the element's
    ///   children.
    /// * `element` - The HTML element being converted.
    /// * `ctx` - The current conversion context with options and mutable state
    ///   (e.g. for accumulating reference-style link definitions).
    fn apply(&self, content: &str, element: &scraper::ElementRef<'_>, ctx: &mut Context) -> Action;
}
