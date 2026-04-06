//! Smoke tests for h2m — basic sanity checks across feature areas.
#![allow(clippy::unwrap_used)]

use h2m::convert;
use pretty_assertions::assert_eq;

#[test]
fn heading_and_paragraph() {
    let md = convert("<h1>Hello</h1><p>World</p>").unwrap();
    assert_eq!(md, "# Hello\n\nWorld");
}

#[test]
fn multiple_headings() {
    let md = convert("<h1>One</h1><h2>Two</h2><h3>Three</h3>").unwrap();
    assert_eq!(md, "# One\n\n## Two\n\n### Three");
}

#[test]
fn heading_all_levels() {
    let md = convert("<h4>Four</h4><h5>Five</h5><h6>Six</h6>").unwrap();
    assert_eq!(md, "#### Four\n\n##### Five\n\n###### Six");
}

#[test]
fn setext_headings() {
    let mut opts = h2m::Options::default();
    opts.heading_style = h2m::HeadingStyle::Setext;
    let converter = h2m::Converter::builder()
        .options(opts)
        .use_plugin(h2m::rules::CommonMark)
        .build();
    let md = converter
        .convert("<h1>Title</h1><h2>Sub</h2><h3>Three</h3>")
        .unwrap();
    assert!(md.contains("Title\n====="));
    assert!(md.contains("Sub\n---"));
    assert!(md.contains("### Three"));
}

#[test]
fn strong_and_em() {
    let md = convert("<p><strong>bold</strong> and <em>italic</em></p>").unwrap();
    assert_eq!(md, "**bold** and *italic*");
}

#[test]
fn inline_code() {
    let md = convert("<p>use <code>cargo</code></p>").unwrap();
    assert_eq!(md, "use `cargo`");
}

#[test]
fn inline_code_with_backticks() {
    let md = convert("<p><code>a`b`c</code></p>").unwrap();
    assert_eq!(md, "``a`b`c``");
}

#[test]
fn link() {
    let md = convert(r#"<p><a href="https://rust-lang.org">Rust</a></p>"#).unwrap();
    assert_eq!(md, "[Rust](https://rust-lang.org)");
}

#[test]
fn link_with_title() {
    let md =
        convert(r#"<p><a href="https://rust-lang.org" title="Rust site">Rust</a></p>"#).unwrap();
    assert_eq!(md, r#"[Rust](https://rust-lang.org "Rust site")"#);
}

#[test]
fn image() {
    let md = convert(r#"<p><img src="cat.png" alt="A cat"/></p>"#).unwrap();
    assert_eq!(md, "![A cat](cat.png)");
}

#[test]
fn unordered_list() {
    let md = convert("<ul><li>one</li><li>two</li><li>three</li></ul>").unwrap();
    assert_eq!(md, "- one\n- two\n- three");
}

#[test]
fn ordered_list() {
    let md = convert("<ol><li>one</li><li>two</li><li>three</li></ol>").unwrap();
    assert_eq!(md, "1. one\n2. two\n3. three");
}

#[test]
fn ordered_list_with_start() {
    let md = convert(r#"<ol start="5"><li>five</li><li>six</li></ol>"#).unwrap();
    assert_eq!(md, "5. five\n6. six");
}

#[test]
fn nested_list() {
    let html = "<ul><li>a<ul><li>b</li><li>c</li></ul></li><li>d</li></ul>";
    let md = convert(html).unwrap();
    assert_eq!(md, "- a\n  - b\n  - c\n- d");
}

#[test]
fn deeply_nested_list() {
    let html = "<ul><li>1<ul><li>2<ul><li>3</li></ul></li></ul></li></ul>";
    let md = convert(html).unwrap();
    assert_eq!(md, "- 1\n  - 2\n    - 3");
}

#[test]
fn blockquote() {
    let md = convert("<blockquote><p>quoted text</p></blockquote>").unwrap();
    assert_eq!(md, "> quoted text");
}

#[test]
fn nested_blockquote() {
    let html = "<blockquote><blockquote><p>deep</p></blockquote></blockquote>";
    let md = convert(html).unwrap();
    assert!(md.contains("> > deep"));
}

#[test]
fn code_block_with_language() {
    let html = r#"<pre><code class="language-rust">fn main() {}</code></pre>"#;
    let md = convert(html).unwrap();
    assert_eq!(md, "```rust\nfn main() {}\n```");
}

#[test]
fn code_block_fence_escalation() {
    // Content contains triple backticks — fence must escalate.
    let html = "<pre><code>```\nsome code\n```</code></pre>";
    let md = convert(html).unwrap();
    assert!(md.starts_with("````"));
    assert!(md.contains("```\nsome code\n```"));
}

#[test]
fn horizontal_rule() {
    let md = convert("<p>before</p><hr/><p>after</p>").unwrap();
    assert_eq!(md, "before\n\n---\n\nafter");
}

#[test]
fn line_break() {
    let md = convert("<p>line1<br/>line2</p>").unwrap();
    assert!(md.contains("line1"));
    assert!(md.contains("line2"));
}

#[test]
fn script_removed() {
    let md = convert("<p>hello</p><script>alert(1)</script><p>world</p>").unwrap();
    assert_eq!(md, "hello\n\nworld");
}

#[test]
fn empty_input() {
    let md = convert("").unwrap();
    assert_eq!(md, "");
}

#[test]
fn whitespace_only() {
    let md = convert("   \n\t  ").unwrap();
    assert_eq!(md, "");
}

#[test]
fn malformed_html() {
    // html5ever recovers gracefully.
    let md = convert("<p>unclosed <b>bold</p>").unwrap();
    assert!(md.contains("**bold**"));
}

#[test]
fn gfm_strikethrough() {
    let md = h2m::convert_gfm("<p><del>removed</del></p>").unwrap();
    assert_eq!(md, "~~removed~~");
}

#[test]
fn gfm_table() {
    let html = "<table><thead><tr><th>Name</th><th>Age</th></tr></thead>\
                <tbody><tr><td>Alice</td><td>30</td></tr></tbody></table>";
    let md = h2m::convert_gfm(html).unwrap();
    assert!(md.contains("| Name"));
    assert!(md.contains("| Alice"));
    assert!(md.contains("---"));
}

#[test]
fn gfm_table_with_alignment() {
    let html = r#"<table><thead><tr><th align="left">L</th><th align="center">C</th><th align="right">R</th></tr></thead>
                  <tbody><tr><td>a</td><td>b</td><td>c</td></tr></tbody></table>"#;
    let md = h2m::convert_gfm(html).unwrap();
    assert!(md.contains(":--"));
    assert!(md.contains("--:"));
}

#[test]
fn gfm_task_list() {
    let html = r#"<ul><li><input type="checkbox" checked/> done</li><li><input type="checkbox"/> todo</li></ul>"#;
    let md = h2m::convert_gfm(html).unwrap();
    assert!(md.contains("[x]"));
    assert!(md.contains("[ ]"));
    assert!(md.contains("done"));
    assert!(md.contains("todo"));
}

#[test]
fn html_entities_decoded() {
    // html5ever decodes entities during parsing.
    let md = convert("<p>&amp; &lt; &gt;</p>").unwrap();
    // The decoded characters get escaped by our escape module.
    assert!(md.contains('&'));
    assert!(md.contains('<') || md.contains("\\<"));
}
