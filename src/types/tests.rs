use notzero::nz;

use crate::matomo::{self, is_ai_agent, is_ignored_url, build_matomo_url, ChatbotTarget};
use crate::types::{PageLinkItem, Paginator};

#[test]
fn gen_pagination_items_for_total_2_pages() {
    let paginator = Paginator {
        current_page: nz!(1u16),
        total_pages: nz!(2u16),
    };
    let items = paginator.generate_items();
    let expected = vec![
        PageLinkItem::new(nz!(1u16), true, false),
        PageLinkItem::new(nz!(2u16), false, false),
    ];
    assert_eq!(items, expected);
}

#[test]
fn gen_pagination_items_for_total_3_pages() {
    let paginator = Paginator {
        current_page: nz!(2u16),
        total_pages: nz!(3u16),
    };
    let items = paginator.generate_items();
    let expected = vec![
        PageLinkItem::new(nz!(1u16), false, false),
        PageLinkItem::new(nz!(2u16), true, false),
        PageLinkItem::new(nz!(3u16), false, false),
    ];
    assert_eq!(items, expected);
}
#[test]

fn gen_pagination_items_for_total_4_pages() {
    let paginator = Paginator {
        current_page: nz!(4u16),
        total_pages: nz!(4u16),
    };
    let items = paginator.generate_items();
    let expected = vec![
        PageLinkItem::new(nz!(1u16), false, false),
        PageLinkItem::new(nz!(2u16), false, false),
        PageLinkItem::new(nz!(3u16), false, false),
        PageLinkItem::new(nz!(4u16), true, false),
    ];
    assert_eq!(items, expected);
}

#[test]
fn gen_pagination_items_for_total_7_pages() {
    let paginator = Paginator {
        current_page: nz!(5u16),
        total_pages: nz!(7u16),
    };
    let items = paginator.generate_items();
    let expected = vec![
        PageLinkItem::new(nz!(1u16), false, false),
        PageLinkItem::new(nz!(2u16), false, false),
        PageLinkItem::new(nz!(3u16), false, false),
        PageLinkItem::new(nz!(4u16), false, false),
        PageLinkItem::new(nz!(5u16), true, false),
        PageLinkItem::new(nz!(6u16), false, false),
        PageLinkItem::new(nz!(7u16), false, false),
    ];
    assert_eq!(items, expected);
}

#[test]
fn gen_pagination_items_for_total_8_pages_current_at_3() {
    let paginator = Paginator {
        current_page: nz!(3u16),
        total_pages: nz!(8u16),
    };
    let items = paginator.generate_items();
    let expected = vec![
        PageLinkItem::new(nz!(1u16), false, false),
        PageLinkItem::new(nz!(2u16), false, false),
        // Current
        PageLinkItem::new(nz!(3u16), true, false),
        PageLinkItem::new(nz!(4u16), false, false),
        // 5th page is not generated
        // This one is ellipsis
        PageLinkItem::new(nz!(6u16), false, true),
        PageLinkItem::new(nz!(7u16), false, false),
        PageLinkItem::new(nz!(8u16), false, false),
    ];
    assert_eq!(items, expected);
}

#[test]
fn gen_pagination_items_for_total_8_pages_current_at_4() {
    let paginator = Paginator {
        current_page: nz!(4u16),
        total_pages: nz!(8u16),
    };
    let items = paginator.generate_items();
    let expected = vec![
        PageLinkItem::new(nz!(1u16), false, false),
        // This one is ellipsis
        PageLinkItem::new(nz!(2u16), false, true),
        PageLinkItem::new(nz!(3u16), false, false),
        // Current
        PageLinkItem::new(nz!(4u16), true, false),
        PageLinkItem::new(nz!(5u16), false, false),
        // This one is ellipsis
        PageLinkItem::new(nz!(6u16), false, true),
        // 7th page is not generated
        PageLinkItem::new(nz!(8u16), false, false),
    ];
    assert_eq!(items, expected);
}

#[test]
fn detects_known_ai_agent_user_agents() {
    assert!(is_ai_agent("Mozilla/5.0 ChatGPT-User/1.0"));
    assert!(is_ai_agent("Mozilla/5.0 (compatible; Perplexity-User/1.0)"));
    assert!(is_ai_agent("MistralAI-User/1.0"));
}

#[test]
fn ignores_non_ai_user_agents() {
    assert!(!is_ai_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)"));
    assert!(!is_ai_agent("curl/7.68.0"));
}

#[test]
fn ignores_static_resource_urls() {
    assert!(is_ignored_url("/static/app.js"));
    assert!(is_ignored_url("/static/app.css"));
    assert!(is_ignored_url("/static/favicon.ico"));
    assert!(!is_ignored_url("/blog/hello-world/"));
    assert!(!is_ignored_url("/blog/post.pdf"));
}

#[test]
fn classifies_download_vs_page_targets() {
    let page_uri: http::Uri = "/blog/post/".parse().unwrap();
    let (url, target) = matomo::classify_target("https", "example.com", &page_uri);
    assert_eq!(url, "https://example.com/blog/post/");
    assert!(matches!(target, ChatbotTarget::Page(_)));

    let download_uri: http::Uri = "/files/report.pdf".parse().unwrap();
    let (url, target) = matomo::classify_target("https", "example.com", &download_uri);
    assert_eq!(url, "https://example.com/files/report.pdf");
    assert!(matches!(target, ChatbotTarget::Download(_)));
}

#[test]
fn matomo_url_encodes_components() {
    let target = ChatbotTarget::Page("https://example.com/hello world".to_string());
    let ua = "ChatGPT-User/1.0; foo&bar";
    let cdt = chrono::DateTime::UNIX_EPOCH;
    let url = build_matomo_url(&target, ua, &cdt);
    assert!(url.starts_with("https://matomo.quan.hoabinh.vn/matomo.php?"));
    assert!(url.contains("url=https%3A%2F%2Fexample.com%2Fhello+world"), "{url}");
    assert!(url.contains("ua=ChatGPT-User%2F1.0%3B+foo%26bar"), "{url}");
    assert!(url.contains("bots=1"), "{url}");
    assert!(url.contains("rec=1"), "{url}");
    assert!(url.contains("idsite=1"), "{url}");
}
