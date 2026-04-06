//! Emphasis, strong, and inline code conversion tests.

mod common;

use common::with_options;
use h2m::{Options, convert};
use pretty_assertions::assert_eq;

#[test]
fn emphasis_default_asterisk() {
    assert_eq!(convert("<p><em>italic</em></p>"), "*italic*");
}

#[test]
fn emphasis_underscore_option() {
    let opts = Options::default().em_delimiter(h2m::EmDelimiter::Underscore);
    assert_eq!(
        with_options(opts).convert("<p><em>italic</em></p>"),
        "_italic_"
    );
}

#[test]
fn strong_default_double_asterisks() {
    assert_eq!(convert("<p><strong>bold</strong></p>"), "**bold**");
}

#[test]
fn strong_underscores_option() {
    let opts = Options::default().strong_delimiter(h2m::StrongDelimiter::Underscores);
    assert_eq!(
        with_options(opts).convert("<p><strong>bold</strong></p>"),
        "__bold__"
    );
}

#[test]
fn b_tag_treated_as_strong() {
    assert_eq!(convert("<p><b>bold</b></p>"), "**bold**");
}

#[test]
fn i_tag_treated_as_em() {
    assert_eq!(convert("<p><i>italic</i></p>"), "*italic*");
}

#[test]
fn strong_and_em_combined() {
    assert_eq!(
        convert("<p><strong>bold</strong> and <em>italic</em></p>"),
        "**bold** and *italic*"
    );
}

#[test]
fn nested_strong_deduplication() {
    assert_eq!(
        convert("<p><strong><strong>bold</strong></strong></p>"),
        "**bold**"
    );
}

#[test]
fn nested_em_deduplication() {
    assert_eq!(convert("<p><em><em>italic</em></em></p>"), "*italic*");
}

#[test]
fn empty_strong_skipped() {
    assert_eq!(convert("<p>a<strong></strong>b</p>"), "ab");
}

#[test]
fn empty_em_skipped() {
    assert_eq!(convert("<p>a<em></em>b</p>"), "ab");
}

#[test]
fn strong_multiline_wraps_per_line() {
    assert_eq!(
        convert("<p><strong>line1<br/>line2</strong></p>"),
        "**line1**\n**line2**"
    );
}

#[test]
fn inline_code_basic() {
    assert_eq!(convert("<p>use <code>cargo</code></p>"), "use `cargo`");
}

#[test]
fn inline_code_containing_backticks() {
    assert_eq!(convert("<p><code>a`b`c</code></p>"), "``a`b`c``");
}

#[test]
fn inline_code_leading_backtick_padded() {
    assert_eq!(convert("<p><code>`start</code></p>"), "`` `start ``");
}

#[test]
fn inline_code_trailing_backtick_padded() {
    assert_eq!(convert("<p><code>end`</code></p>"), "`` end` ``");
}

#[test]
fn kbd_tag_rendered_as_inline_code() {
    assert_eq!(convert("<p>Press <kbd>Ctrl+C</kbd></p>"), "Press `Ctrl+C`");
}

#[test]
fn empty_inline_code_skipped() {
    assert_eq!(convert("<p>a<code></code>b</p>"), "ab");
}

#[test]
fn samp_tag_rendered_as_inline_code() {
    assert_eq!(convert("<p>output: <samp>42</samp></p>"), "output: `42`");
}

#[test]
fn tt_tag_rendered_as_inline_code() {
    assert_eq!(convert("<p>use <tt>monospace</tt></p>"), "use `monospace`");
}
