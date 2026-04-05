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
fn link() {
    let md = convert(r#"<p><a href="https://rust-lang.org">Rust</a></p>"#).unwrap();
    assert_eq!(md, "[Rust](https://rust-lang.org)");
}

#[test]
fn link_with_title() {
    let md =
        convert(r#"<p><a href="https://rust-lang.org" title="Rust site">Rust</a></p>"#).unwrap();
    assert_eq!(md, "[Rust](https://rust-lang.org \"Rust site\")");
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
fn nested_list() {
    let html = "<ul><li>a<ul><li>b</li><li>c</li></ul></li><li>d</li></ul>";
    let md = convert(html).unwrap();
    assert_eq!(md, "- a\n  - b\n  - c\n- d");
}

#[test]
fn blockquote() {
    let md = convert("<blockquote><p>quoted text</p></blockquote>").unwrap();
    assert_eq!(md, "> quoted text");
}

#[test]
fn code_block_with_language() {
    let html = r#"<pre><code class="language-rust">fn main() {}</code></pre>"#;
    let md = convert(html).unwrap();
    assert_eq!(md, "```rust\nfn main() {}\n```");
}

#[test]
fn horizontal_rule() {
    let md = convert("<p>before</p><hr/><p>after</p>").unwrap();
    assert_eq!(md, "before\n\n---\n\nafter");
}

#[test]
fn script_removed() {
    let md = convert("<p>hello</p><script>alert(1)</script><p>world</p>").unwrap();
    assert_eq!(md, "hello\n\nworld");
}

#[test]
fn gfm_strikethrough() {
    let md = h2m::convert_gfm("<p><del>removed</del></p>").unwrap();
    assert_eq!(md, "~~removed~~");
}

#[test]
fn gfm_table() {
    let html = "<table><thead><tr><th>Name</th><th>Age</th></tr></thead>\
                <tbody><tr><td>Alice</td><td>30</td></tr><tr><td>Bob</td><td>25</td></tr></tbody></table>";
    let md = h2m::convert_gfm(html).unwrap();
    assert!(md.contains("| Name"));
    assert!(md.contains("| Alice"));
    assert!(md.contains("---"));
}

#[test]
fn gfm_task_list() {
    let html = r#"<ul><li><input type="checkbox" checked/> done</li><li><input type="checkbox"/> todo</li></ul>"#;
    let md = h2m::convert_gfm(html).unwrap();
    assert!(md.contains("[x] done"));
    assert!(md.contains("[ ] todo"));
}
