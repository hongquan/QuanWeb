use notzero::nz;

use super::{PageLinkItem, Paginator};

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
