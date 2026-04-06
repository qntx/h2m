//! Heading conversion tests.

mod common;

use common::with_options;
use h2m::{Options, convert};
use pretty_assertions::assert_eq;

#[test]
fn headings_atx_h1_through_h3() {
    assert_eq!(
        convert("<h1>One</h1><h2>Two</h2><h3>Three</h3>"),
        "# One\n\n## Two\n\n### Three"
    );
}

#[test]
fn headings_atx_h4_through_h6() {
    assert_eq!(
        convert("<h4>Four</h4><h5>Five</h5><h6>Six</h6>"),
        "#### Four\n\n##### Five\n\n###### Six"
    );
}

#[test]
fn headings_setext_h1_and_h2() {
    let opts = Options::default().heading_style(h2m::HeadingStyle::Setext);
    let c = with_options(opts);
    assert_eq!(
        c.convert("<h1>Title</h1><h2>Sub</h2>"),
        "Title\n=====\n\nSub\n---"
    );
}

#[test]
fn headings_setext_falls_back_to_atx_at_h3() {
    let opts = Options::default().heading_style(h2m::HeadingStyle::Setext);
    let c = with_options(opts);
    assert_eq!(c.convert("<h3>Three</h3>"), "### Three");
}

#[test]
fn headings_hash_in_content_escaped() {
    assert_eq!(convert("<h1>C# Guide</h1>"), "# C\\# Guide");
}

#[test]
fn headings_empty_content_skipped() {
    assert_eq!(convert("<h1></h1><p>text</p>"), "text");
}

#[test]
fn headings_newlines_collapsed_to_spaces() {
    assert_eq!(convert("<h1>line1\nline2</h1>"), "# line1 line2");
}

#[test]
fn headings_with_paragraph() {
    assert_eq!(convert("<h1>Hello</h1><p>World</p>"), "# Hello\n\nWorld");
}
