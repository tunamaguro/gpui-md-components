//! スライダーの状態とキー操作。
mod support;
use gpui::{Role, TestAppContext, prelude::*};
use gpui_headless::Slider;
use std::{cell::Cell, rc::Rc};

#[gpui::test]
fn slider_keys_follow_apg(cx: &mut TestAppContext) {
    let value = Rc::new(Cell::new(5.0_f32));
    let cx = support::open(cx, {
        let value = value.clone();
        move |window, cx| {
            let set = value.clone();
            Slider::new("volume", value.get(), "音量")
                .range(0.0, 10.0)
                .step(1.0)
                .on_change(move |next, _, _| set.set(next))
                .build(window, cx)
                .0
                .size(gpui::px(40.0))
                .into_any_element()
        }
    });
    support::tab(cx);
    support::press(cx, "right");
    assert_eq!(value.get(), 6.0);
    support::press(cx, "down");
    assert_eq!(value.get(), 5.0);
    support::press(cx, "end");
    assert_eq!(value.get(), 10.0);
    support::press(cx, "pagedown");
    assert_eq!(value.get(), 0.0);
    support::press(cx, "home");
    assert_eq!(value.get(), 0.0);
}

#[test]
fn slider_exposes_range() {
    let slider = Slider::new("s", 3.0, "明るさ")
        .range(0.0, 10.0)
        .step(1.0)
        .a11y_node();
    assert_eq!(slider.role, Role::Slider);
    let range = slider.range.expect("range");
    assert_eq!(
        (range.value, range.min, range.max, range.step),
        (3.0, 0.0, 10.0, Some(1.0))
    );
}

#[gpui::test]
fn drag_tracks_position_and_releases_outside(cx: &mut TestAppContext) {
    use gpui::{InputEvent, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, point, px};
    let value = Rc::new(Cell::new(0.0_f32));
    let dragging = Rc::new(Cell::new(false));
    let cx = support::open(cx, {
        let value = value.clone();
        let dragging = dragging.clone();
        move |window, cx| {
            let set = value.clone();
            let (base, state) = Slider::new("drag", value.get(), "音量")
                .on_change(move |next, _, _| set.set(next))
                .build(window, cx);
            dragging.set(state.dragging);
            base.w(px(200.0)).h(px(40.0)).into_any_element()
        }
    });
    let start = point(px(66.0), px(30.0));
    cx.update(|window, cx| {
        window.dispatch_event(
            MouseDownEvent {
                position: start,
                button: MouseButton::Left,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
    });
    support::draw(cx);
    assert!((value.get() - 0.25).abs() < 0.01);
    assert!(dragging.get());
    let end = point(px(300.0), px(30.0));
    cx.update(|window, cx| {
        window.dispatch_event(
            MouseMoveEvent {
                position: end,
                pressed_button: Some(MouseButton::Left),
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
    });
    support::draw(cx);
    assert_eq!(value.get(), 1.0);
    cx.update(|window, cx| {
        window.dispatch_event(
            MouseUpEvent {
                position: end,
                button: MouseButton::Left,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
    });
    support::draw(cx);
    assert!(!dragging.get());
}
