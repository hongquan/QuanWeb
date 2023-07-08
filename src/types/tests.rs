
use nonzero_ext::nonzero;

use super::{Paginator, PageLinkItem};

#[test]
fn gen_pagination_items_for_total_2_pages() {
    let paginator = Paginator {
        current_page: nonzero!(1u16),
        total_pages: nonzero!(2u16),
    };
    let items = paginator.generate_items();
    let expected = vec![
        PageLinkItem::new(nonzero!(1u16), true, false),
        PageLinkItem::new(nonzero!(2u16), false, false),
    ];
    assert_eq!(items, expected);
}

#[test]
fn gen_pagination_items_for_total_3_pages() {
    let paginator = Paginator {
        current_page: nonzero!(2u16),
        total_pages: nonzero!(3u16),
    };
    let items = paginator.generate_items();
    let expected = vec![
        PageLinkItem::new(nonzero!(1u16), false, false),
        PageLinkItem::new(nonzero!(2u16), true, false),
        PageLinkItem::new(nonzero!(3u16), false, false),
    ];
    assert_eq!(items, expected);
}
#[test]

fn gen_pagination_items_for_total_4_pages() {
    let paginator = Paginator {
        current_page: nonzero!(4u16),
        total_pages: nonzero!(4u16),
    };
    let items = paginator.generate_items();
    let expected = vec![
        PageLinkItem::new(nonzero!(1u16), false, false),
        PageLinkItem::new(nonzero!(2u16), false, false),
        PageLinkItem::new(nonzero!(3u16), false, false),
        PageLinkItem::new(nonzero!(4u16), true, false),
    ];
    assert_eq!(items, expected);
}

#[test]
fn gen_pagination_items_for_total_7_pages() {
    let paginator = Paginator {
        current_page: nonzero!(5u16),
        total_pages: nonzero!(7u16),
    };
    let items = paginator.generate_items();
    let expected = vec![
        PageLinkItem::new(nonzero!(1u16), false, false),
        PageLinkItem::new(nonzero!(2u16), false, false),
        PageLinkItem::new(nonzero!(3u16), false, false),
        PageLinkItem::new(nonzero!(4u16), false, false),
        PageLinkItem::new(nonzero!(5u16), true, false),
        PageLinkItem::new(nonzero!(6u16), false, false),
        PageLinkItem::new(nonzero!(7u16), false, false),
    ];
    assert_eq!(items, expected);
}

#[test]
fn gen_pagination_items_for_total_8_pages_current_at_3() {
    let paginator = Paginator {
        current_page: nonzero!(3u16),
        total_pages: nonzero!(8u16),
    };
    let items = paginator.generate_items();
    let expected = vec![
        PageLinkItem::new(nonzero!(1u16), false, false),
        PageLinkItem::new(nonzero!(2u16), false, false),
        // Current
        PageLinkItem::new(nonzero!(3u16), true, false),
        PageLinkItem::new(nonzero!(4u16), false, false),
        // 5th page is not generated
        // This one is ellipsis
        PageLinkItem::new(nonzero!(6u16), false, true),
        PageLinkItem::new(nonzero!(7u16), false, false),
        PageLinkItem::new(nonzero!(8u16), false, false),
    ];
    assert_eq!(items, expected);
}

#[test]
fn gen_pagination_items_for_total_8_pages_current_at_4() {
    let paginator = Paginator {
        current_page: nonzero!(4u16),
        total_pages: nonzero!(8u16),
    };
    let items = paginator.generate_items();
    let expected = vec![
        PageLinkItem::new(nonzero!(1u16), false, false),
        // This one is ellipsis
        PageLinkItem::new(nonzero!(2u16), false, true),
        PageLinkItem::new(nonzero!(3u16), false, false),
        // Current
        PageLinkItem::new(nonzero!(4u16), true, false),
        PageLinkItem::new(nonzero!(5u16), false, false),
        // This one is ellipsis
        PageLinkItem::new(nonzero!(6u16), false, true),
        // 7th page is not generated
        PageLinkItem::new(nonzero!(8u16), false, false),
    ];
    assert_eq!(items, expected);
}
