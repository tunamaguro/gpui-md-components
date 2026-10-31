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
                        .child(content)
                        .into_any_element(),
                    1 => BottomSheet::new("bottom")
                        .aria_label("長い本文")
                        .child(content)
                        .into_any_element(),
                    _ => SideSheet::new("side")
                        .headline("長い本文")
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

#[gpui::test]
fn multi_line_scrollbar_takes_clicks_only_while_rows_are_hidden(cx: &mut TestAppContext) {
    use gpui::{Modifiers, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, div};
    use md3_components::inputs::{TextField, TextFieldState};
    const LEFT: f32 = 16.0;
    const WIDTH: f32 = 300.0;
    for (text, takes_click) in [("短い本文", false), (&*"行\n".repeat(30), true)] {
        let mut app = cx.clone();
        let text = text.to_owned();
        let window = support::open(&mut app, {
            let state = std::cell::RefCell::new(None::<TextFieldState>);
            move |_, cx| {
                let field = state
                    .borrow_mut()
                    .get_or_insert_with(|| TextFieldState::with_text(text.clone(), cx))
                    .clone();
                div()
                    .w(px(WIDTH))
                    .child(TextField::new("body", &field).multi_line(1, 4))
                    .into_any_element()
            }
        });
        // 24dp の余白の中で、入力の行の高さの中ほど。
        let position = point(px(LEFT + WIDTH - 4.0), px(LEFT + 40.0));
        send(
            window,
            MouseMoveEvent {
                position,
                ..Default::default()
            },
        );
        send(
            window,
            MouseDownEvent {
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                first_mouse: false,
            },
        );
        send(
            window,
            MouseUpEvent {
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
            },
        );
        let focused = window.update(|window, cx| window.focused(cx).is_some());
        assert_eq!(
            focused, !takes_click,
            "隠れた行があるときだけバーがクリックを受ける"
        );
    }
}
