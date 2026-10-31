//! ScrollArea の位置とホイール。
mod support;
use gpui::{InputEvent, ScrollHandle, TestAppContext, canvas, point, prelude::*, px};
use md3_components::scrollbar::ScrollArea;
use std::{cell::Cell, rc::Rc};

#[gpui::test]
fn wheel_moves_the_content_and_the_handle(cx: &mut TestAppContext) {
    let handle = ScrollHandle::new();
    let origin = Rc::new(Cell::new(point(px(0.0), px(0.0))));
    let cx = support::open(cx, {
        let handle = handle.clone();
        let origin = origin.clone();
        move |_, _| {
            let origin = origin.clone();
            ScrollArea::new("area")
                .handle(handle.clone())
                .w(px(200.0))
                .h(px(200.0))
                .child(
                    canvas(
                        move |bounds, _, _| origin.set(bounds.origin),
                        |_, _, _, _| {},
                    )
                    .w(px(100.0))
                    .h(px(2000.0)),
                )
                .into_any_element()
        }
    });
    let before = origin.get();
    cx.update(|window, cx| {
        let event = gpui::ScrollEvent {
            position: before + point(px(20.0), px(20.0)),
            delta: gpui::ScrollDelta::Pixels(point(px(0.0), px(-100.0))),
            touch_phase: gpui::TouchPhase::Started,
            ..Default::default()
        };
        window.dispatch_event(event.to_platform_input(), cx);
    });
    support::draw(cx);
    assert_eq!(origin.get().y, before.y - px(100.0));
    assert_eq!(handle.offset().y, px(-100.0));
}
