//! Integration tests for h2m.

use h2m::{Converter, Options, convert, convert_gfm};
use pretty_assertions::assert_eq;

fn with_options(opts: Options) -> Converter {
    Converter::builder()
        .options(opts)
        .use_plugin(h2m::rules::CommonMark)
        .build()
}

fn with_domain(domain: &str) -> Converter {
    Converter::builder()
        .use_plugin(h2m::rules::CommonMark)
        .domain(domain)
        .build()
}

fn ref_converter(style: h2m::LinkReferenceStyle) -> Converter {
    let mut opts = Options::default();
    opts.link_style = h2m::LinkStyle::Referenced;
    opts.link_reference_style = style;
    with_options(opts)
}

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
    let mut opts = Options::default();
    opts.heading_style = h2m::HeadingStyle::Setext;
    let c = with_options(opts);
    assert_eq!(
        c.convert("<h1>Title</h1><h2>Sub</h2>"),
        "Title\n=====\n\nSub\n---"
    );
}

#[test]
fn headings_setext_falls_back_to_atx_at_h3() {
    let mut opts = Options::default();
    opts.heading_style = h2m::HeadingStyle::Setext;
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

#[test]
fn emphasis_default_asterisk() {
    assert_eq!(convert("<p><em>italic</em></p>"), "*italic*");
}

#[test]
fn emphasis_underscore_option() {
    let mut opts = Options::default();
    opts.em_delimiter = h2m::EmDelimiter::Underscore;
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
    let mut opts = Options::default();
    opts.strong_delimiter = h2m::StrongDelimiter::Underscores;
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
    let mut opts = Options::default();
    opts.bullet_marker = h2m::BulletMarker::Plus;
    assert_eq!(
        with_options(opts).convert("<ul><li>a</li><li>b</li></ul>"),
        "+ a\n+ b"
    );
}

#[test]
fn list_bullet_asterisk_option() {
    let mut opts = Options::default();
    opts.bullet_marker = h2m::BulletMarker::Asterisk;
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
    let mut opts = Options::default();
    opts.fence = h2m::Fence::Tilde;
    assert_eq!(
        with_options(opts).convert(r#"<pre><code class="language-py">pass</code></pre>"#),
        "~~~py\npass\n~~~"
    );
}

#[test]
fn code_block_indented_style() {
    let mut opts = Options::default();
    opts.code_block_style = h2m::CodeBlockStyle::Indented;
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
fn hr_default_dashes() {
    assert_eq!(
        convert("<p>before</p><hr/><p>after</p>"),
        "before\n\n---\n\nafter"
    );
}

#[test]
fn hr_asterisks_option() {
    let mut opts = Options::default();
    opts.horizontal_rule = h2m::HorizontalRule::Asterisks;
    assert_eq!(
        with_options(opts).convert("<p>before</p><hr/><p>after</p>"),
        "before\n\n***\n\nafter"
    );
}

#[test]
fn hr_underscores_option() {
    let mut opts = Options::default();
    opts.horizontal_rule = h2m::HorizontalRule::Underscores;
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
fn domain_resolves_relative_image() {
    let c = with_domain("example.com");
    assert_eq!(
        c.convert(r#"<img src="/img/cat.png" alt="cat"/>"#),
        "![cat](https://example.com/img/cat.png)"
    );
}

#[test]
fn domain_resolves_relative_link() {
    let c = with_domain("example.com");
    assert_eq!(
        c.convert(r#"<a href="/about">About</a>"#),
        "[About](https://example.com/about)"
    );
}

#[test]
fn domain_absolute_url_unchanged() {
    let c = with_domain("example.com");
    assert_eq!(
        c.convert(r#"<a href="https://other.com/page">Link</a>"#),
        "[Link](https://other.com/page)"
    );
}

#[test]
fn domain_with_protocol_preserves_scheme() {
    let c = with_domain("https://example.com");
    assert_eq!(
        c.convert(r#"<a href="/about">About</a>"#),
        "[About](https://example.com/about)"
    );
}

#[test]
fn domain_resolves_relative_iframe() {
    let c = with_domain("example.com");
    assert_eq!(
        c.convert(r#"<iframe src="/embed/video"></iframe>"#),
        "[iframe](https://example.com/embed/video)"
    );
}

#[test]
fn gfm_del_tag() {
    assert_eq!(convert_gfm("<p><del>removed</del></p>"), "~~removed~~");
}

#[test]
fn gfm_s_tag() {
    assert_eq!(convert_gfm("<p><s>removed</s></p>"), "~~removed~~");
}

#[test]
fn gfm_strike_tag() {
    assert_eq!(
        convert_gfm("<p><strike>removed</strike></p>"),
        "~~removed~~"
    );
}

#[test]
fn gfm_table_basic_structure() {
    let html = "<table><thead><tr><th>Name</th><th>Age</th></tr></thead>\
                <tbody><tr><td>Alice</td><td>30</td></tr></tbody></table>";
    let md = convert_gfm(html);
    let lines: Vec<&str> = md.lines().collect();
    assert!(lines.len() >= 3, "table should have at least 3 lines");
    assert!(lines[0].contains("Name"));
    assert!(lines[0].contains("Age"));
    assert!(lines[1].contains("---"));
    assert!(lines[2].contains("Alice"));
    assert!(lines[2].contains("30"));
}

#[test]
fn gfm_table_alignment() {
    let html = r#"<table><thead><tr>
        <th align="left">L</th><th align="center">C</th><th align="right">R</th>
    </tr></thead><tbody><tr><td>a</td><td>b</td><td>c</td></tr></tbody></table>"#;
    let md = convert_gfm(html);
    let lines: Vec<&str> = md.lines().collect();
    assert!(lines.len() >= 2, "table should have a separator row");
    let sep = lines[1];
    assert!(sep.contains(":--"), "left alignment marker");
    assert!(
        sep.contains(":-") && sep.contains("-:"),
        "center alignment marker"
    );
    assert!(sep.contains("--:"), "right alignment marker");
}

#[test]
fn gfm_task_list_checked_and_unchecked() {
    let html = r#"<ul>
        <li><input type="checkbox" checked/> done</li>
        <li><input type="checkbox"/> todo</li>
    </ul>"#;
    let md = convert_gfm(html);
    let lines: Vec<&str> = md.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("[x]") && lines[0].contains("done"));
    assert!(lines[1].contains("[ ]") && lines[1].contains("todo"));
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
    let mut opts = Options::default();
    opts.escape_mode = h2m::EscapeMode::Disabled;
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
fn convert_gfm_includes_all_extensions() {
    let md = convert_gfm("<p><del>strike</del></p>");
    assert_eq!(md, "~~strike~~");
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
fn samp_tag_rendered_as_inline_code() {
    assert_eq!(convert("<p>output: <samp>42</samp></p>"), "output: `42`");
}

#[test]
fn tt_tag_rendered_as_inline_code() {
    assert_eq!(convert("<p>use <tt>monospace</tt></p>"), "use `monospace`");
}

#[test]
fn multiple_br_produces_multiple_hard_breaks() {
    assert_eq!(convert("<p>a<br/>b<br/>c</p>"), "a  \nb  \nc");
}

#[test]
fn code_block_class_fallback_to_first_class() {
    assert_eq!(
        convert(r#"<pre><code class="ruby">puts 1</code></pre>"#),
        "```ruby\nputs 1\n```"
    );
}

#[test]
fn link_referenced_full_with_title() {
    let c = ref_converter(h2m::LinkReferenceStyle::Full);
    let md = c.convert(r#"<p><a href="https://example.com" title="Example">click</a></p>"#);
    assert!(md.contains("[click][1]"));
    assert!(md.contains(r#"[1]: https://example.com "Example""#));
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
fn strong_multiline_wraps_per_line() {
    assert_eq!(
        convert("<p><strong>line1<br/>line2</strong></p>"),
        "**line1**\n**line2**"
    );
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
fn head_elements_not_in_output() {
    let html = r#"<html><head><title>Page Title</title><meta name="description" content="desc"></head><body><p>Hello</p></body></html>"#;
    let md = convert(html);
    assert!(
        !md.contains("Page Title"),
        "title should not leak into body"
    );
    assert_eq!(md, "Hello");
}

#[test]
fn extract_links_with_base_resolves_relative() {
    let links = h2m::html::extract_links_with_base(
        r#"<a href="/about">About</a><a href="https://other.com">Other</a>"#,
        "https://example.com/page",
    );
    assert_eq!(
        links,
        vec!["https://example.com/about", "https://other.com"]
    );
}

#[test]
fn extract_links_without_base_returns_raw() {
    let links = h2m::html::extract_links(r#"<a href="/about">About</a>"#);
    assert_eq!(links, vec!["/about"]);
}

#[test]
fn domain_bare_defaults_to_https() {
    let c = with_domain("example.com");
    assert_eq!(
        c.convert(r#"<a href="/page">Link</a>"#),
        "[Link](https://example.com/page)"
    );
}

#[test]
fn domain_explicit_http_preserved() {
    let c = with_domain("http://example.com");
    assert_eq!(
        c.convert(r#"<a href="/page">Link</a>"#),
        "[Link](http://example.com/page)"
    );
}
