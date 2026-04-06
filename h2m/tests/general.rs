//! General tests: HR, BR, iframe, escaping, builder, edge cases.

mod common;

use common::with_options;
use h2m::{Converter, Options, convert};
use pretty_assertions::assert_eq;

#[test]
fn hr_default_dashes() {
    assert_eq!(
        convert("<p>before</p><hr/><p>after</p>"),
        "before\n\n---\n\nafter"
    );
}

#[test]
fn hr_asterisks_option() {
    let opts = Options::default().horizontal_rule(h2m::HorizontalRule::Asterisks);
    assert_eq!(
        with_options(opts).convert("<p>before</p><hr/><p>after</p>"),
        "before\n\n***\n\nafter"
    );
}

#[test]
fn hr_underscores_option() {
    let opts = Options::default().horizontal_rule(h2m::HorizontalRule::Underscores);
    assert_eq!(
        with_options(opts).convert("<p>before</p><hr/><p>after</p>"),
        "before\n\n___\n\nafter"
    );
}

#[test]
fn hr_inside_heading_suppressed() {
    let md = convert("<h2>Title<hr/>More</h2>");
    assert!(!md.contains("---"));
    assert!(md.contains("Title"));
}

#[test]
fn line_break_hard() {
    assert_eq!(convert("<p>line1<br/>line2</p>"), "line1  \nline2");
}

#[test]
fn multiple_br_produces_multiple_hard_breaks() {
    assert_eq!(convert("<p>a<br/>b<br/>c</p>"), "a  \nb  \nc");
}

#[test]
fn iframe_rendered_as_link() {
    assert_eq!(
        convert(r#"<iframe src="https://example.com/embed"></iframe>"#),
        "[iframe](https://example.com/embed)"
    );
}

#[test]
fn iframe_data_uri_skipped() {
    assert_eq!(
        convert(r#"<iframe src="data:text/html,<h1>hi</h1>"></iframe>"#),
        ""
    );
}

#[test]
fn html_entities_decoded_and_special_chars_escaped() {
    let md = convert("<p>&amp; &lt; &gt;</p>");
    assert!(md.contains("\\<"), "< should be escaped");
    assert!(md.contains("\\>"), "> should be escaped");
    assert!(
        md.contains('&'),
        "& should be preserved (not a markdown special)"
    );
}

#[test]
fn escape_mode_disabled() {
    let opts = Options::default().escape_mode(h2m::EscapeMode::Disabled);
    let md = with_options(opts).convert("<p>*not bold* and [not link]</p>");
    assert_eq!(md, "*not bold* and [not link]");
}

#[test]
fn markdown_special_chars_escaped_in_text() {
    let md = convert("<p>a*b_c</p>");
    assert_eq!(md, "a\\*b\\_c");
}

#[test]
fn builder_keep_tags_preserves_html() {
    let c = Converter::builder()
        .use_plugin(h2m::rules::CommonMark)
        .keep(&["custom-tag"])
        .build();
    let md = c.convert("<p>before</p><custom-tag>inside</custom-tag><p>after</p>");
    assert!(md.contains("<custom-tag>inside</custom-tag>"));
    assert!(md.contains("before"));
    assert!(md.contains("after"));
}

#[test]
fn builder_remove_tags_strips_content() {
    let c = Converter::builder()
        .use_plugin(h2m::rules::CommonMark)
        .remove(&["aside"])
        .build();
    let md = c.convert("<p>before</p><aside>sidebar</aside><p>after</p>");
    assert!(!md.contains("sidebar"));
    assert_eq!(md, "before\n\nafter");
}

#[test]
fn builder_default_options_produce_commonmark() {
    let c = Converter::builder()
        .use_plugin(h2m::rules::CommonMark)
        .build();
    let md = c.convert("<h1>Title</h1><p><strong>bold</strong></p><ul><li>item</li></ul>");
    assert_eq!(md, "# Title\n\n**bold**\n\n- item");
}

#[test]
fn empty_input() {
    assert_eq!(convert(""), "");
}

#[test]
fn whitespace_only_input() {
    assert_eq!(convert("   \n\t  "), "");
}

#[test]
fn malformed_html_recovered() {
    let md = convert("<p>unclosed <b>bold</p>");
    assert_eq!(md, "unclosed **bold**");
}

#[test]
fn script_removed() {
    assert_eq!(
        convert("<p>hello</p><script>alert(1)</script><p>world</p>"),
        "hello\n\nworld"
    );
}

#[test]
fn noscript_removed() {
    assert_eq!(
        convert("<p>hello</p><noscript>fallback</noscript><p>world</p>"),
        "hello\n\nworld"
    );
}

#[test]
fn plain_text_without_tags() {
    assert_eq!(convert("just text"), "just text");
}

#[test]
fn multiple_paragraphs_separated_by_blank_lines() {
    assert_eq!(
        convert("<p>first</p><p>second</p><p>third</p>"),
        "first\n\nsecond\n\nthird"
    );
}

#[test]
fn heading_inside_link_becomes_bold() {
    // html5ever restructures <a><h2>...</h2></a> due to block-in-inline rules.
    let md = convert(r#"<a href="/page"><h2>Title</h2></a>"#);
    assert!(
        md.contains("**Title**"),
        "heading-in-link should render as bold"
    );
    assert!(!md.contains("## "), "should not produce ATX heading prefix");
}

#[test]
fn convert_is_infallible() {
    let _: String = convert("<h1>test</h1>");
}

#[test]
#[allow(clippy::unwrap_used)]
fn convert_reader_from_byte_slice() {
    let c = Converter::builder()
        .use_plugin(h2m::rules::CommonMark)
        .build();
    let md = c.convert_reader(&b"<h1>Hello</h1>"[..]).unwrap();
    assert_eq!(md, "# Hello");
}

#[test]
fn div_treated_as_block_container() {
    assert_eq!(convert("<div>content</div>"), "content");
}

#[test]
fn section_treated_as_block_container() {
    assert_eq!(convert("<section><p>inside</p></section>"), "inside");
}

#[test]
fn head_elements_not_in_output() {
    let html = r#"<html><head><title>Page Title</title><meta name="description" content="desc"></head><body><p>Hello</p></body></html>"#;
    let md = convert(html);
    assert!(
        !md.contains("Page Title"),
        "title should not leak into body"
    );
    assert_eq!(md, "Hello");
}
