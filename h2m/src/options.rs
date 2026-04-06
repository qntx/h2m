//! Configuration options for the HTML-to-Markdown converter.

/// Style for rendering headings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum HeadingStyle {
    /// ATX-style headings using `#` prefixes.
    #[default]
    Atx,
    /// Setext-style headings using `===` and `---` underlines (h1/h2 only,
    /// falls back to ATX for h3+).
    Setext,
}

/// Style for rendering code blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum CodeBlockStyle {
    /// Fenced code blocks using backticks or tildes.
    #[default]
    Fenced,
    /// Indented code blocks using 4-space indent.
    Indented,
}

/// Fence character for fenced code blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Fence {
    /// Triple backtick fences.
    #[default]
    Backtick,
    /// Triple tilde fences.
    Tilde,
}

impl Fence {
    /// Returns the fence character.
    #[must_use]
    #[inline]
    pub const fn char(self) -> char {
        match self {
            Self::Backtick => '`',
            Self::Tilde => '~',
        }
    }
}

/// Bullet character for unordered lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum BulletMarker {
    /// Dash: `-`.
    #[default]
    Dash,
    /// Plus: `+`.
    Plus,
    /// Asterisk: `*`.
    Asterisk,
}

impl BulletMarker {
    /// Returns the bullet character.
    #[must_use]
    #[inline]
    pub const fn char(self) -> char {
        match self {
            Self::Dash => '-',
            Self::Plus => '+',
            Self::Asterisk => '*',
        }
    }
}

/// Delimiter for emphasis (italic) text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum EmDelimiter {
    /// Asterisk: `*text*`.
    #[default]
    Asterisk,
    /// Underscore: `_text_`.
    Underscore,
}

impl EmDelimiter {
    /// Returns the delimiter character.
    #[must_use]
    #[inline]
    pub const fn char(self) -> char {
        match self {
            Self::Asterisk => '*',
            Self::Underscore => '_',
        }
    }
}

/// Delimiter for strong emphasis (bold) text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum StrongDelimiter {
    /// Double asterisks: `**text**`.
    #[default]
    Asterisks,
    /// Double underscores: `__text__`.
    Underscores,
}

impl StrongDelimiter {
    /// Returns the delimiter string.
    #[must_use]
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Asterisks => "**",
            Self::Underscores => "__",
        }
    }
}

/// Horizontal rule style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum HorizontalRule {
    /// Three dashes: `---`.
    #[default]
    Dashes,
    /// Three asterisks: `***`.
    Asterisks,
    /// Three underscores: `___`.
    Underscores,
}

impl HorizontalRule {
    /// Returns the rule string.
    #[must_use]
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Dashes => "---",
            Self::Asterisks => "***",
            Self::Underscores => "___",
        }
    }
}

/// Mode for escaping markdown special characters in text content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum EscapeMode {
    /// Escape common markdown special characters.
    #[default]
    Basic,
    /// Do not escape any characters.
    Disabled,
}

/// Style for rendering links.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum LinkStyle {
    /// Inline links: `[text](url "title")`.
    #[default]
    Inlined,
    /// Reference-style links: `[text][id]` with a footer `[id]: url "title"`.
    Referenced,
}

/// Style for reference-style link identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum LinkReferenceStyle {
    /// Full reference: `[text][id]` / `[id]: url`.
    #[default]
    Full,
    /// Collapsed reference: `[text][]` / `[text]: url`.
    Collapsed,
    /// Shortcut reference: `[text]` / `[text]: url`.
    Shortcut,
}

/// Configuration options for the converter.
///
/// Use [`Default::default()`] for sensible `CommonMark` defaults.
#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub struct Options {
    /// Heading rendering style.
    pub heading_style: HeadingStyle,
    /// Bullet character for unordered lists.
    pub bullet_marker: BulletMarker,
    /// Code block rendering style.
    pub code_block_style: CodeBlockStyle,
    /// Fence character for fenced code blocks.
    pub fence: Fence,
    /// Delimiter for emphasis (italic).
    pub em_delimiter: EmDelimiter,
    /// Delimiter for strong emphasis (bold).
    pub strong_delimiter: StrongDelimiter,
    /// Horizontal rule string.
    pub horizontal_rule: HorizontalRule,
    /// Escape mode for markdown special characters.
    pub escape_mode: EscapeMode,
    /// Link rendering style.
    pub link_style: LinkStyle,
    /// Reference-style link identifier format.
    pub link_reference_style: LinkReferenceStyle,
}
