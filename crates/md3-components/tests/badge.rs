//! アンカーとバッジの描画位置。
#![cfg(debug_assertions)]

mod support;

use gpui::{TestAppContext, div, prelude::*, px};
use md3_components::selection::{Badge, BadgedBox};

#[gpui::test]
fn badge_overlaps_without_changing_anchor_size(cx: &mut TestAppContext) {
    for flex_parent in [true, false] {
        for (badge, horizontal, vertical) in [
            (Badge::new("3"), 12.0, 14.0),
            (Badge::new("999+"), 12.0, 14.0),
            (Badge::dot(), 6.0, 6.0),
        ] {
            let cx = support::open(cx, move |_, _| {
                div()
                    .w(px(200.0))
                    .when(flex_parent, |parent| parent.flex())
                    .debug_selector(|| "parent".into())
                    .child(BadgedBox::new(
                        div().size(px(24.0)).debug_selector(|| "anchor".into()),
                        badge.clone(),
                    ))
                    .into_any_element()
            });
            let parent = cx.debug_bounds("parent").expect("親がある");
            let anchor = cx.debug_bounds("anchor").expect("アンカーがある");
            let anchor_box = cx.debug_bounds("md3-badged-box").expect("基準の箱がある");
            let badge = cx
                .debug_bounds("md3-badged-box-badge")
                .expect("バッジがある");
            assert_eq!(parent.size.width, px(200.0), "親の幅は 200px");
            assert_eq!(anchor.size.width, px(24.0), "アンカーの幅は 24px");
            assert_eq!(anchor_box, anchor, "基準の箱はアンカーと同じ位置と大きさ");
            assert_eq!(badge.left(), anchor.right() - px(horizontal));
            assert_eq!(badge.bottom(), anchor.top() + px(vertical));
        }
    }
}
