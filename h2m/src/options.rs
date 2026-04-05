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
    pub const fn char(self) -> char {
        match self {
            Self::Backtick => '`',
            Self::Tilde => '~',
        }
    }
}

/// Style for rendering links.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum LinkStyle {
    /// Inline links: `[text](url "title")`.
    #[default]
    Inlined,
    /// Reference-style links: `[text][ref]` with definitions at the bottom.
    Referenced,
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

/// Configuration options for the converter.
///
/// Use [`Default::default()`] for sensible `CommonMark` defaults.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Options {
    /// Heading rendering style.
    pub heading_style: HeadingStyle,
    /// Bullet character for unordered lists.
    pub bullet_marker: char,
    /// Code block rendering style.
    pub code_block_style: CodeBlockStyle,
    /// Fence character for fenced code blocks.
    pub fence: Fence,
    /// Delimiter for emphasis (italic).
    pub em_delimiter: char,
    /// Delimiter for strong emphasis (bold).
    pub strong_delimiter: &'static str,
    /// Link rendering style.
    pub link_style: LinkStyle,
    /// Horizontal rule string.
    pub horizontal_rule: &'static str,
    /// Escape mode for markdown special characters.
    pub escape_mode: EscapeMode,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            heading_style: HeadingStyle::default(),
            bullet_marker: '-',
            code_block_style: CodeBlockStyle::default(),
            fence: Fence::default(),
            em_delimiter: '*',
            strong_delimiter: "**",
            link_style: LinkStyle::default(),
            horizontal_rule: "---",
            escape_mode: EscapeMode::default(),
        }
    }
}
