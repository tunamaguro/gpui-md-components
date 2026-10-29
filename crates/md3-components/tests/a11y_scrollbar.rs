//! 装飾スクロールバーのポインタ操作とフォーカス非介入。
mod support;
use gpui::{
    Axis, InputEvent, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ScrollHandle,
    TestAppContext, VisualTestContext, div, point, prelude::*, px,
};
use md3_components::{Scrollbar, ScrollbarInteraction, ScrollbarSource};

fn send(cx: &mut VisualTestContext, event: impl InputEvent) {
    cx.update(|window, cx| {
        window.dispatch_event(event.to_platform_input(), cx);
    });
    support::draw(cx);
}

#[gpui::test]
fn drag_outside_thumb_and_release(cx: &mut TestAppContext) {
    let scroll = ScrollHandle::new();
    let cx = support::open(cx, {
        let scroll = scroll.clone();
        move |_, _| {
            div()
                .id("scroll")
                .w(px(200.0))
                .h(px(200.0))
                .overflow_y_scroll()
                .track_scroll(&scroll)
                .child(div().h(px(1000.0)))
                .child(Scrollbar::new("bar", scroll.clone()).fade(false))
                .into_any_element()
        }
    });
    let bounds = scroll.bounds();
    let start = point(bounds.right() - px(2.0), bounds.top() + px(10.0));
    send(
        cx,
        MouseMoveEvent {
            position: start,
            ..Default::default()
        },
    );
    send(
        cx,
        MouseDownEvent {
            position: start,
            button: MouseButton::Left,
            ..Default::default()
        },
    );
    let end = point(bounds.right() + px(80.0), bounds.bottom() + px(80.0));
    send(
        cx,
        MouseMoveEvent {
            position: end,
            pressed_button: Some(MouseButton::Left),
            ..Default::default()
        },
    );
    assert_eq!(scroll.offset().y, -scroll.max_offset().y);
    send(
        cx,
        MouseUpEvent {
            position: end,
            button: MouseButton::Left,
            ..Default::default()
        },
    );
    send(
        cx,
        MouseMoveEvent {
            position: start,
            pressed_button: Some(MouseButton::Left),
            ..Default::default()
        },
    );
    assert_eq!(
        scroll.offset().y,
        -scroll.max_offset().y,
        "離した後は動かない"
    );
    support::tab(cx);
    cx.update(|window, cx| assert!(window.focused(cx).is_none(), "バーは Tab 順に入らない"));
}

#[gpui::test]
fn horizontal_page_and_passive(cx: &mut TestAppContext) {
    let scroll = ScrollHandle::new();
    let passive = std::rc::Rc::new(std::cell::Cell::new(false));
    let cx = support::open(cx, {
        let scroll = scroll.clone();
        let passive = passive.clone();
        move |_, _| {
            div()
                .id("scroll")
                .w(px(200.0))
                .h(px(100.0))
                .overflow_x_scroll()
                .track_scroll(&scroll)
                .child(div().w(px(1000.0)).h(px(50.0)))
                .child(
                    Scrollbar::new("bar", scroll.clone())
                        .axis(Axis::Horizontal)
                        .fade(false)
                        .interaction(if passive.get() {
                            ScrollbarInteraction::Passive
                        } else {
                            ScrollbarInteraction::Draggable
                        }),
                )
                .into_any_element()
        }
    });
    let bounds = scroll.bounds();
    let position = point(bounds.right() - px(10.0), bounds.bottom() - px(2.0));
    send(
        cx,
        MouseMoveEvent {
            position,
            ..Default::default()
        },
    );
    send(
        cx,
        MouseDownEvent {
            position,
            button: MouseButton::Left,
            ..Default::default()
        },
    );
    assert_eq!(scroll.offset().x, px(-200.0));
    send(
        cx,
        MouseUpEvent {
            position,
            button: MouseButton::Left,
            ..Default::default()
        },
    );
    passive.set(true);
    support::draw(cx);
    send(
        cx,
        MouseDownEvent {
            position,
            button: MouseButton::Left,
            ..Default::default()
        },
    );
    assert_eq!(scroll.offset().x, px(-200.0));
    ScrollbarSource::set_offset(&scroll, Axis::Horizontal, px(9000.0));
    assert_eq!(scroll.offset().x, -scroll.max_offset().x);
    assert_eq!(scroll.offset().y, px(0.0));
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
                    0 => Dialog::new("dialog", "長い本文")
                        .inline()
                        .child(content)
                        .into_any_element(),
                    1 => BottomSheet::new("bottom", "長い本文")
                        .inline()
                        .child(content)
                        .into_any_element(),
                    _ => SideSheet::new("side", "長い本文")
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
