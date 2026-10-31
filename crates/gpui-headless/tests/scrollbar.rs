//! 装飾スクロールバーのポインタ操作とフォーカス非介入。
mod support;
use gpui::{
    Axis, InputEvent, ListAlignment, ListState, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, ScrollHandle, TestAppContext, VisualTestContext, div, list, point, prelude::*,
    px,
};
use gpui_headless::{Scrollbar, ScrollbarGeometry, ScrollbarInteraction, ScrollbarSource};
use std::{cell::Cell, rc::Rc};

fn send(cx: &mut VisualTestContext, event: impl InputEvent) {
    cx.update(|window, cx| {
        window.dispatch_event(event.to_platform_input(), cx);
    });
    support::draw(cx);
}

fn geometry() -> ScrollbarGeometry {
    ScrollbarGeometry {
        track_inset: px(2.0),
        cross_inset: px(0.0),
        thickness: px(4.0),
        min_thumb_length: px(24.0),
        max_thumb_fraction: 0.9,
        hit_width: px(12.0),
    }
}
#[gpui::test]
fn drag_outside_thumb_and_release(cx: &mut TestAppContext) {
    let scroll = ScrollHandle::new();
    let cx = support::open(cx, {
        let scroll = scroll.clone();
        move |window, cx| {
            div()
                .id("scroll")
                .w(px(200.0))
                .h(px(200.0))
                .overflow_y_scroll()
                .track_scroll(&scroll)
                .child(div().h(px(1000.0)))
                .child(Scrollbar::new("bar", scroll.clone(), geometry()).build(
                    window,
                    cx,
                    |_, _, _| {},
                ))
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
        move |window, cx| {
            div()
                .id("scroll")
                .w(px(200.0))
                .h(px(100.0))
                .overflow_x_scroll()
                .track_scroll(&scroll)
                .child(div().w(px(1000.0)).h(px(50.0)))
                .child(
                    Scrollbar::new("bar", scroll.clone(), geometry())
                        .axis(Axis::Horizontal)
                        .interaction(if passive.get() {
                            ScrollbarInteraction::Passive
                        } else {
                            ScrollbarInteraction::Draggable
                        })
                        .build(window, cx, |_, _, _| {}),
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
fn list_state_reports_positive_offsets(cx: &mut TestAppContext) {
    let state = ListState::new(20, ListAlignment::Top, px(100.0)).measure_all();
    let cx = support::open(cx, {
        let state = state.clone();
        move |_, _| {
            div()
                .w(px(200.0))
                .h(px(200.0))
                .child(
                    list(state.clone(), |_, _, _| {
                        div().h(px(50.0)).into_any_element()
                    })
                    .size_full(),
                )
                .into_any_element()
        }
    });
    assert_eq!(ScrollbarSource::viewport(&state).size.height, px(200.0));
    assert_eq!(
        ScrollbarSource::max_offset(&state, Axis::Vertical),
        px(800.0)
    );

    ScrollbarSource::set_offset(&state, Axis::Vertical, px(300.0));
    support::draw(cx);
    assert_eq!(ScrollbarSource::offset(&state, Axis::Vertical), px(300.0));

    ScrollbarSource::set_offset(&state, Axis::Horizontal, px(100.0));
    support::draw(cx);
    assert_eq!(ScrollbarSource::offset(&state, Axis::Vertical), px(300.0));
    assert_eq!(ScrollbarSource::offset(&state, Axis::Horizontal), px(0.0));
}

#[gpui::test]
fn list_state_keeps_its_range_while_dragging(cx: &mut TestAppContext) {
    let state = ListState::new(20, ListAlignment::Top, px(100.0)).measure_all();
    let cx = support::open(cx, {
        let state = state.clone();
        move |window, cx| {
            div()
                .relative()
                .w(px(200.0))
                .h(px(200.0))
                .child(
                    list(state.clone(), |_, _, _| {
                        div().h(px(50.0)).into_any_element()
                    })
                    .size_full(),
                )
                .child(Scrollbar::new("bar", state.clone(), geometry()).build(
                    window,
                    cx,
                    |_, _, _| {},
                ))
                .into_any_element()
        }
    });
    let bounds = state.viewport_bounds();
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
    assert!(state.is_scrollbar_dragging());
    send(
        cx,
        MouseUpEvent {
            position: start,
            button: MouseButton::Left,
            ..Default::default()
        },
    );
    assert!(!state.is_scrollbar_dragging());
}

#[gpui::test]
fn list_state_is_released_when_the_bar_goes_away(cx: &mut TestAppContext) {
    let state = ListState::new(20, ListAlignment::Top, px(100.0)).measure_all();
    let visible = Rc::new(Cell::new(true));
    let cx = support::open(cx, {
        let state = state.clone();
        let visible = visible.clone();
        move |window, cx| {
            let bar = visible.get().then(|| {
                Scrollbar::new("bar", state.clone(), geometry()).build(window, cx, |_, _, _| {})
            });
            div()
                .relative()
                .w(px(200.0))
                .h(px(200.0))
                .child(
                    list(state.clone(), |_, _, _| {
                        div().h(px(50.0)).into_any_element()
                    })
                    .size_full(),
                )
                .children(bar)
                .into_any_element()
        }
    });
    let bounds = state.viewport_bounds();
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
    assert!(state.is_scrollbar_dragging());

    visible.set(false);
    support::draw(cx);
    support::draw(cx);
    assert!(
        !state.is_scrollbar_dragging(),
        "バーが消えたら範囲の固定を解く"
    );
}
