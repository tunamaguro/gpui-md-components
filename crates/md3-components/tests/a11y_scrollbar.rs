//! 装飾スクロールバーのポインタ操作とフォーカス非介入。
mod support;
use gpui::{InputEvent, TestAppContext, VisualTestContext, point, prelude::*, px};

fn send(cx: &mut VisualTestContext, event: impl InputEvent) {
    cx.update(|window, cx| {
        window.dispatch_event(event.to_platform_input(), cx);
    });
    support::draw(cx);
}

#[gpui::test]
fn tall_overlay_bodies_scroll(cx: &mut TestAppContext) {
    use md3_components::overlays::{BottomSheet, Dialog, SideSheet};
    use std::{cell::Cell, rc::Rc};
    for kind in 0..3 {
        let origin = Rc::new(Cell::new(point(px(0.0), px(0.0))));
        let cx = support::open(cx, {
            let origin = origin.clone();
            move |_, _| {
                let origin = origin.clone();
                let content = gpui::canvas(
                    move |bounds, _, _| origin.set(bounds.origin),
                    |_, _, _, _| {},
                )
                .w(px(100.0))
                .h(px(2000.0));
                match kind {
                    0 => Dialog::new("dialog")
                        .headline("長い本文")
                        .inline()
                        .child(content)
                        .into_any_element(),
                    1 => BottomSheet::new("bottom")
                        .aria_label("長い本文")
                        .inline()
                        .child(content)
                        .into_any_element(),
                    _ => SideSheet::new("side")
                        .headline("長い本文")
                        .inline()
                        .child(content)
                        .into_any_element(),
                }
            }
        });
        let before = origin.get();
        send(
            cx,
            gpui::ScrollEvent {
                position: before + point(px(20.0), px(20.0)),
                delta: gpui::ScrollDelta::Pixels(point(px(0.0), px(-100.0))),
                touch_phase: gpui::TouchPhase::Started,
                ..Default::default()
            },
        );
        assert_eq!(
            origin.get().y,
            before.y - px(100.0),
            "本文がスクロールする: {kind}"
        );
    }
}
