//! Domain URL resolution tests.

mod common;

use common::with_domain;
use pretty_assertions::assert_eq;

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
