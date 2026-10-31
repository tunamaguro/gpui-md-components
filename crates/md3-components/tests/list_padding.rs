//! List のスクロールする余白。
#![cfg(debug_assertions)]
mod support;
use gpui::{Edges, SharedString, TestAppContext, div, point, prelude::*, px};
use md3_components::containment::{List, ListItem};

fn last_selector() -> &'static str {
    let id = gpui::ElementId::NamedChild(
        std::sync::Arc::new("padded".into()),
        SharedString::from("k9"),
    );
    Box::leak(format!("md3-list-item-{id}").into_boxed_str())
}

#[gpui::test]
fn padding_scrolls_with_items_and_focus_reveal_still_works(cx: &mut TestAppContext) {
    let cx = support::open(cx, |_, _| {
        let list = (0..10).fold(
            List::new("padded")
                .max_height(px(180.))
                .content_padding(Edges {
                    bottom: px(80.),
                    ..Default::default()
                }),
            |list, i| {
                list.item(
                    format!("k{i}"),
                    ListItem::new(format!("項目 {i}")).on_select(|_, _| {}),
                )
            },
        );
        div().w(px(300.)).child(list).into_any_element()
    });
    let viewport = cx.debug_bounds("md3-list-padded").expect("リスト");
    support::tab(cx);
    support::press(cx, "end");
    let last = cx.debug_bounds(last_selector()).expect("末尾");
    assert!(last.bottom() <= viewport.bottom());
    cx.update(|window, cx| {
        window.dispatch_event(
            gpui::PlatformInput::Scroll(gpui::ScrollEvent {
                position: point(viewport.left() + px(20.), viewport.top() + px(20.)),
                delta: gpui::ScrollDelta::Pixels(point(px(0.), px(-1000.))),
                touch_phase: gpui::TouchPhase::Moved,
                ..Default::default()
            }),
            cx,
        )
    });
    support::draw(cx);
    let last = cx.debug_bounds(last_selector()).expect("末尾");
    assert_eq!(viewport.bottom() - last.bottom(), px(80.));
}

#[gpui::test]
fn padding_does_not_enlarge_a_height_limited_list(cx: &mut TestAppContext) {
    let cx = support::open(cx, |_, _| {
        div()
            .flex()
            .gap_4()
            .children([0., 80.].into_iter().enumerate().map(|(index, bottom)| {
                let list = (0..10).fold(
                    List::new(format!("list-{index}"))
                        .max_height(px(180.))
                        .content_padding(Edges {
                            bottom: px(bottom),
                            ..Default::default()
                        }),
                    |list, i| list.item(format!("{i}"), ListItem::new("項目")),
                );
                div().w(px(200.)).child(list)
            }))
            .into_any_element()
    });
    let plain = cx.debug_bounds("md3-list-list-0").expect("余白なし");
    let padded = cx.debug_bounds("md3-list-list-1").expect("余白あり");
    assert_eq!(plain.size.height, padded.size.height);
}
