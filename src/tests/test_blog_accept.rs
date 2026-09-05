//! Tests for the Accept header handling in the blog post view.
//!
//! These tests verify that the show_post handler gracefully handles
//! Accept headers that don't use the standard format that `headers_accept`
//! would reject with a 400 error. This originally affected the Crush Fetch tool.

use axum::http::HeaderMap;
use headers::HeaderValue;

use crate::front::views::blog::parse_accept_prefer_markdown;

/// Test that a standard Accept header with text/markdown returns true
#[test]
fn test_accept_header_with_markdown() {
    let mut headers = HeaderMap::new();
    headers.insert(
        "accept",
        HeaderValue::from_static("text/markdown, text/html, */*"),
    );
    assert!(parse_accept_prefer_markdown(&headers));
}

/// Test that text/html only (no markdown) returns false
#[test]
fn test_accept_header_without_markdown() {
    let mut headers = HeaderMap::new();
    headers.insert(
        "accept",
        HeaderValue::from_static("text/html, application/xhtml+xml, */*"),
    );
    assert!(!parse_accept_prefer_markdown(&headers));
}

/// Test that */* by itself returns false (no explicit markdown preference)
#[test]
fn test_accept_header_wildcard_only() {
    let mut headers = HeaderMap::new();
    headers
        .insert("accept", HeaderValue::from_static("*/*"));
    assert!(!parse_accept_prefer_markdown(&headers));
}

/// Test that a missing Accept header returns false (no Accept header at all)
#[test]
fn test_accept_header_missing() {
    let headers = HeaderMap::new();
    assert!(!parse_accept_prefer_markdown(&headers));
}

/// Test Accept headers with media type parameters (e.g. charset)
#[test]
fn test_accept_header_with_parameters() {
    let mut headers = HeaderMap::new();
    headers.insert(
        "accept",
        HeaderValue::from_static("text/markdown;charset=utf-8"),
    );
    assert!(parse_accept_prefer_markdown(&headers));
}

/// Test that malformed Accept headers are handled gracefully (no panic)
/// This is the key regression test for the Crush Fetch 400 bug.
#[test]
fn test_accept_header_malformed_utf8() {
    let mut headers = HeaderMap::new();
    // Insert a HeaderValue with invalid UTF-8 — this simulates what some
    // misbehaving HTTP clients send, which would cause headers_accept to
    // return a 400 error.
    let malformed = HeaderValue::from_bytes(b"\xff\xfe").unwrap();
    headers.insert("accept", malformed);
    assert!(!parse_accept_prefer_markdown(&headers));
}

/// Test multiple Accept headers — none containing markdown
#[test]
fn test_multiple_accept_no_markdown() {
    let mut headers = HeaderMap::new();
    headers.append("accept", HeaderValue::from_static("text/html"));
    headers.append("accept", HeaderValue::from_static("*/*"));
    assert!(!parse_accept_prefer_markdown(&headers));
}
