//! List, blockquote, and code block conversion tests.

mod common;

use common::with_options;
use h2m::{Options, convert};
use pretty_assertions::assert_eq;

#[test]
fn list_unordered() {
    assert_eq!(
        convert("<ul><li>one</li><li>two</li><li>three</li></ul>"),
        "- one\n- two\n- three"
    );
}

#[test]
fn list_ordered() {
    assert_eq!(
        convert("<ol><li>one</li><li>two</li><li>three</li></ol>"),
        "1. one\n2. two\n3. three"
    );
}

#[test]
fn list_ordered_start_attribute() {
    assert_eq!(
        convert(r#"<ol start="5"><li>five</li><li>six</li></ol>"#),
        "5. five\n6. six"
    );
}

#[test]
fn list_nested_unordered() {
    assert_eq!(
        convert("<ul><li>a<ul><li>b</li><li>c</li></ul></li><li>d</li></ul>"),
        "- a\n  - b\n  - c\n- d"
    );
}

#[test]
fn list_deeply_nested_three_levels() {
    assert_eq!(
        convert("<ul><li>1<ul><li>2<ul><li>3</li></ul></li></ul></li></ul>"),
        "- 1\n  - 2\n    - 3"
    );
}

#[test]
fn list_bullet_plus_option() {
    let opts = Options::default().with_bullet_marker(h2m::BulletMarker::Plus);
    assert_eq!(
        with_options(opts).convert("<ul><li>a</li><li>b</li></ul>"),
        "+ a\n+ b"
    );
}

#[test]
fn list_bullet_asterisk_option() {
    let opts = Options::default().with_bullet_marker(h2m::BulletMarker::Asterisk);
    assert_eq!(
        with_options(opts).convert("<ul><li>a</li><li>b</li></ul>"),
        "* a\n* b"
    );
}

#[test]
fn blockquote_basic() {
    assert_eq!(
        convert("<blockquote><p>quoted text</p></blockquote>"),
        "> quoted text"
    );
}

#[test]
fn blockquote_nested() {
    assert_eq!(
        convert("<blockquote><blockquote><p>deep</p></blockquote></blockquote>"),
        "> > deep"
    );
}

#[test]
fn blockquote_empty_skipped() {
    assert_eq!(convert("<blockquote></blockquote>"), "");
}

#[test]
fn blockquote_multi_paragraph() {
    let md = convert("<blockquote><p>first</p><p>second</p></blockquote>");
    let lines: Vec<&str> = md.lines().collect();
    assert!(lines.iter().all(|l| l.starts_with('>')));
    assert!(md.contains("first"));
    assert!(md.contains("second"));
}

#[test]
fn code_block_fenced_with_language() {
    assert_eq!(
        convert(r#"<pre><code class="language-rust">fn main() {}</code></pre>"#),
        "```rust\nfn main() {}\n```"
    );
}

#[test]
fn code_block_fenced_without_language() {
    assert_eq!(
        convert("<pre><code>hello world</code></pre>"),
        "```\nhello world\n```"
    );
}

#[test]
fn code_block_fence_escalation() {
    assert_eq!(
        convert("<pre><code>```\nsome code\n```</code></pre>"),
        "````\n```\nsome code\n```\n````"
    );
}

#[test]
fn code_block_tilde_fence() {
    let opts = Options::default().with_fence(h2m::Fence::Tilde);
    assert_eq!(
        with_options(opts).convert(r#"<pre><code class="language-py">pass</code></pre>"#),
        "~~~py\npass\n~~~"
    );
}

#[test]
fn code_block_indented_style() {
    let opts = Options::default().with_code_block_style(h2m::CodeBlockStyle::Indented);
    assert_eq!(
        with_options(opts).convert("<pre><code>line1\nline2</code></pre>"),
        "    line1\n    line2"
    );
}

#[test]
fn code_block_lang_prefix_detection() {
    assert_eq!(
        convert(r#"<pre><code class="lang-js">const x = 1;</code></pre>"#),
        "```js\nconst x = 1;\n```"
    );
}

#[test]
fn code_block_class_fallback_to_first_class() {
    assert_eq!(
        convert(r#"<pre><code class="ruby">puts 1</code></pre>"#),
        "```ruby\nputs 1\n```"
    );
}
