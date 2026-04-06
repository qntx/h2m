//! Link and image conversion tests.

mod common;

use common::ref_converter;
use h2m::convert;
use pretty_assertions::assert_eq;

#[test]
fn link_inline_basic() {
    assert_eq!(
        convert(r#"<p><a href="https://rust-lang.org">Rust</a></p>"#),
        "[Rust](https://rust-lang.org)"
    );
}

#[test]
fn link_inline_with_title() {
    assert_eq!(
        convert(r#"<p><a href="https://example.com" title="Example">click</a></p>"#),
        r#"[click](https://example.com "Example")"#
    );
}

#[test]
fn link_empty_content_fallback_to_title() {
    assert_eq!(
        convert(r#"<p><a href="https://example.com" title="Example"></a></p>"#),
        r#"[Example](https://example.com "Example")"#
    );
}

#[test]
fn link_empty_content_fallback_to_aria_label() {
    assert_eq!(
        convert(r#"<p><a href="https://example.com" aria-label="Example"></a></p>"#),
        "[Example](https://example.com)"
    );
}

#[test]
fn link_no_href_passthrough() {
    assert_eq!(convert("<p><a>just text</a></p>"), "just text");
}

#[test]
fn link_hash_only_passthrough() {
    assert_eq!(convert(r##"<p><a href="#">click</a></p>"##), "click");
}

#[test]
fn link_referenced_full_single() {
    let c = ref_converter(h2m::LinkReferenceStyle::Full);
    let md = c.convert(r#"<p><a href="https://rust-lang.org">Rust</a></p>"#);
    assert_eq!(md, "[Rust][1]\n\n[1]: https://rust-lang.org");
}

#[test]
fn link_referenced_full_multiple() {
    let c = ref_converter(h2m::LinkReferenceStyle::Full);
    let md = c.convert(r#"<p><a href="https://a.com">A</a> and <a href="https://b.com">B</a></p>"#);
    assert!(md.contains("[A][1]"));
    assert!(md.contains("[B][2]"));
    assert!(md.contains("[1]: https://a.com"));
    assert!(md.contains("[2]: https://b.com"));
}

#[test]
fn link_referenced_collapsed() {
    let c = ref_converter(h2m::LinkReferenceStyle::Collapsed);
    let md = c.convert(r#"<p><a href="https://rust-lang.org">Rust</a></p>"#);
    assert_eq!(md, "[Rust][]\n\n[Rust]: https://rust-lang.org");
}

#[test]
fn link_referenced_shortcut() {
    let c = ref_converter(h2m::LinkReferenceStyle::Shortcut);
    let md = c.convert(r#"<p><a href="https://rust-lang.org">Rust</a></p>"#);
    assert_eq!(md, "[Rust]\n\n[Rust]: https://rust-lang.org");
}

#[test]
fn link_referenced_full_with_title() {
    let c = ref_converter(h2m::LinkReferenceStyle::Full);
    let md = c.convert(r#"<p><a href="https://example.com" title="Example">click</a></p>"#);
    assert!(md.contains("[click][1]"));
    assert!(md.contains(r#"[1]: https://example.com "Example""#));
}

#[test]
fn link_multiline_content_escaped() {
    let md = convert(
        r#"<p><a href="https://example.com">line1
line2</a></p>"#,
    );
    assert!(md.contains("[line1"));
    assert!(md.contains("](https://example.com)"));
}

#[test]
fn image_basic() {
    assert_eq!(
        convert(r#"<p><img src="cat.png" alt="A cat"/></p>"#),
        "![A cat](cat.png)"
    );
}

#[test]
fn image_with_title() {
    assert_eq!(
        convert(r#"<p><img src="cat.png" alt="A cat" title="Cute cat"/></p>"#),
        r#"![A cat](cat.png "Cute cat")"#
    );
}

#[test]
fn image_no_src_skipped() {
    assert_eq!(convert(r#"<p><img src="" alt="nothing"/></p>"#), "");
}

#[test]
fn image_alt_newline_normalized_to_space() {
    assert_eq!(
        convert("<p><img src=\"cat.png\" alt=\"A\ncat\"/></p>"),
        "![A cat](cat.png)"
    );
}
