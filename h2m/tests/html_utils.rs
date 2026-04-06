//! HTML utility function tests (`extract_links`, `readable_content`, `strip_noise`).

use pretty_assertions::assert_eq;

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
fn readable_content_phase1_semantic_selector() {
    let html = "<nav>menu</nav><article><p>Main content</p></article><footer>ft</footer>";
    let result = h2m::html::readable_content(html);
    assert_eq!(result, "<p>Main content</p>");
}

#[test]
fn readable_content_phase2_noise_stripping() {
    let html = "<nav>menu</nav><div><p>Hello world</p></div><footer>ft</footer>";
    let result = h2m::html::readable_content(html);
    assert!(!result.contains("menu"), "nav should be stripped");
    assert!(!result.contains("ft"), "footer should be stripped");
    assert!(
        result.contains("Hello world"),
        "content should be preserved"
    );
}

#[test]
fn strip_noise_removes_nav_footer_aside_header() {
    let html = "<header>hd</header><nav>nv</nav><p>content</p><aside>sd</aside><footer>ft</footer>";
    let result = h2m::html::strip_noise(html);
    assert!(!result.contains("hd"));
    assert!(!result.contains("nv"));
    assert!(!result.contains("sd"));
    assert!(!result.contains("ft"));
    assert!(result.contains("content"));
}

#[test]
fn strip_noise_removes_aria_roles() {
    let html =
        r#"<div role="navigation">nav</div><p>content</p><div role="contentinfo">info</div>"#;
    let result = h2m::html::strip_noise(html);
    assert!(!result.contains("nav</div>"));
    assert!(!result.contains("info"));
    assert!(result.contains("content"));
}

#[test]
fn strip_noise_no_noise_returns_original() {
    let html = "<div><p>plain content</p></div>";
    let result = h2m::html::strip_noise(html);
    assert!(result.contains("plain content"));
}

#[test]
fn readable_content_prefers_article_over_noise_strip() {
    let html = "<nav>menu</nav><article><p>article</p></article><div><p>other</p></div><footer>ft</footer>";
    let result = h2m::html::readable_content(html);
    assert_eq!(result, "<p>article</p>");
    assert!(!result.contains("other"));
}
