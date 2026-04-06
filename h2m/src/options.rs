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
/// Use [`Default::default()`] for sensible `CommonMark` defaults, then
/// override individual fields with the provided setter methods:
///
/// ```
/// use h2m::Options;
///
/// let opts = Options::default()
///     .with_heading_style(h2m::HeadingStyle::Setext)
///     .with_bullet_marker(h2m::BulletMarker::Asterisk);
/// ```
#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub struct Options {
    /// Heading rendering style.
    heading_style: HeadingStyle,
    /// Bullet character for unordered lists.
    bullet_marker: BulletMarker,
    /// Code block rendering style.
    code_block_style: CodeBlockStyle,
    /// Fence character for fenced code blocks.
    fence: Fence,
    /// Delimiter for emphasis (italic).
    em_delimiter: EmDelimiter,
    /// Delimiter for strong emphasis (bold).
    strong_delimiter: StrongDelimiter,
    /// Horizontal rule string.
    horizontal_rule: HorizontalRule,
    /// Escape mode for markdown special characters.
    escape_mode: EscapeMode,
    /// Link rendering style.
    link_style: LinkStyle,
    /// Reference-style link identifier format.
    link_reference_style: LinkReferenceStyle,
}

impl Options {
    /// Returns the heading rendering style.
    #[inline]
    #[must_use]
    pub const fn heading_style(&self) -> HeadingStyle {
        self.heading_style
    }

    /// Returns the bullet character for unordered lists.
    #[inline]
    #[must_use]
    pub const fn bullet_marker(&self) -> BulletMarker {
        self.bullet_marker
    }

    /// Returns the code block rendering style.
    #[inline]
    #[must_use]
    pub const fn code_block_style(&self) -> CodeBlockStyle {
        self.code_block_style
    }

    /// Returns the fence character for fenced code blocks.
    #[inline]
    #[must_use]
    pub const fn fence(&self) -> Fence {
        self.fence
    }

    /// Returns the delimiter for emphasis (italic) text.
    #[inline]
    #[must_use]
    pub const fn em_delimiter(&self) -> EmDelimiter {
        self.em_delimiter
    }

    /// Returns the delimiter for strong emphasis (bold) text.
    #[inline]
    #[must_use]
    pub const fn strong_delimiter(&self) -> StrongDelimiter {
        self.strong_delimiter
    }

    /// Returns the horizontal rule style.
    #[inline]
    #[must_use]
    pub const fn horizontal_rule(&self) -> HorizontalRule {
        self.horizontal_rule
    }

    /// Returns the escape mode for markdown special characters.
    #[inline]
    #[must_use]
    pub const fn escape_mode(&self) -> EscapeMode {
        self.escape_mode
    }

    /// Returns the link rendering style.
    #[inline]
    #[must_use]
    pub const fn link_style(&self) -> LinkStyle {
        self.link_style
    }

    /// Returns the reference-style link identifier format.
    #[inline]
    #[must_use]
    pub const fn link_reference_style(&self) -> LinkReferenceStyle {
        self.link_reference_style
    }

    /// Sets the heading rendering style.
    #[must_use]
    pub const fn with_heading_style(mut self, style: HeadingStyle) -> Self {
        self.heading_style = style;
        self
    }

    /// Sets the bullet character for unordered lists.
    #[must_use]
    pub const fn with_bullet_marker(mut self, marker: BulletMarker) -> Self {
        self.bullet_marker = marker;
        self
    }

    /// Sets the code block rendering style.
    #[must_use]
    pub const fn with_code_block_style(mut self, style: CodeBlockStyle) -> Self {
        self.code_block_style = style;
        self
    }

    /// Sets the fence character for fenced code blocks.
    #[must_use]
    pub const fn with_fence(mut self, fence: Fence) -> Self {
        self.fence = fence;
        self
    }

    /// Sets the delimiter for emphasis (italic) text.
    #[must_use]
    pub const fn with_em_delimiter(mut self, delim: EmDelimiter) -> Self {
        self.em_delimiter = delim;
        self
    }

    /// Sets the delimiter for strong emphasis (bold) text.
    #[must_use]
    pub const fn with_strong_delimiter(mut self, delim: StrongDelimiter) -> Self {
        self.strong_delimiter = delim;
        self
    }

    /// Sets the horizontal rule style.
    #[must_use]
    pub const fn with_horizontal_rule(mut self, rule: HorizontalRule) -> Self {
        self.horizontal_rule = rule;
        self
    }

    /// Sets the escape mode for markdown special characters.
    #[must_use]
    pub const fn with_escape_mode(mut self, mode: EscapeMode) -> Self {
        self.escape_mode = mode;
        self
    }

    /// Sets the link rendering style.
    #[must_use]
    pub const fn with_link_style(mut self, style: LinkStyle) -> Self {
        self.link_style = style;
        self
    }

    /// Sets the reference-style link identifier format.
    #[must_use]
    pub const fn with_link_reference_style(mut self, style: LinkReferenceStyle) -> Self {
        self.link_reference_style = style;
        self
    }
}
