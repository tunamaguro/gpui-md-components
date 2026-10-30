//! フォーカスを得た入力方法と、その後のマウス移動。

mod support;

use gpui::{
    FocusHandle, KeyBinding, Modifiers, TestAppContext, VisualTestContext, actions, div, point,
    prelude::*, px,
};
use md3_components::{buttons::Button, focus_handle, is_focus_visible};

actions!(
    focus_test,
    [
        /// 次の操作要素に移る。
        Next
    ]
);

fn open(cx: &mut TestAppContext) -> &mut VisualTestContext {
    cx.update(|cx| cx.bind_keys([KeyBinding::new("tab", Next, None)]));
    support::open(cx, |window, cx| {
        let root = focus_handle("test-root", window, cx).tab_stop(false);
        if window.focused(cx).is_none() {
            window.focus(&root, cx);
        }
        div()
            .id("root")
            .track_focus(&root)
            .on_action(|_: &Next, window, cx| window.focus_next(cx))
            .child(Button::new("first", "一つ目").on_click(|_, _, _| {}))
            .child(Button::new("second", "二つ目").on_click(|_, _, _| {}))
            .into_any_element()
    })
}

fn focused(cx: &mut VisualTestContext) -> FocusHandle {
    cx.update(|window, cx| window.focused(cx).expect("フォーカス中の要素"))
}

fn mouse_move_after_tab(cx: &mut TestAppContext, same_position: bool) {
    let cx = open(cx);
    let root = focused(cx);
    support::press(cx, "tab");
    let first = focused(cx);
    assert_ne!(first, root, "Tab で最初の操作要素へ移る");
    assert!(
        cx.update(|window, cx| is_focus_visible(&first, window, cx)),
        "Tab で最初のリングが見える"
    );
    cx.update(|window, cx| {
        let position = window.mouse_position();
        let position = if same_position {
            position
        } else {
            point(position.x + px(10.0), position.y + px(10.0))
        };
        window.simulate_mouse_move(position, cx);
    });
    support::draw(cx);
    assert!(
        cx.update(|window, cx| is_focus_visible(&first, window, cx)),
        "マウス移動後も最初のリングが見える"
    );
}

#[gpui::test]
fn keyboard_focus_survives_mouse_move_at_same_position(cx: &mut TestAppContext) {
    mouse_move_after_tab(cx, true);
}

#[gpui::test]
fn keyboard_focus_survives_mouse_move_to_different_position(cx: &mut TestAppContext) {
    mouse_move_after_tab(cx, false);
}

#[gpui::test]
fn tab_moves_the_visible_ring_to_the_next_control(cx: &mut TestAppContext) {
    let cx = open(cx);
    support::press(cx, "tab");
    let first = focused(cx);
    assert!(cx.update(|window, cx| is_focus_visible(&first, window, cx)));
    support::press(cx, "tab");
    let second = focused(cx);
    assert_ne!(first, second, "Tab で次の操作要素へ移る");
    assert!(
        cx.update(|window, cx| !is_focus_visible(&first, window, cx)
            && is_focus_visible(&second, window, cx))
    );
}

#[gpui::test]
fn mouse_focus_and_move_do_not_show_the_ring(cx: &mut TestAppContext) {
    let cx = open(cx);
    support::press(cx, "tab");
    let first = focused(cx);
    support::press(cx, "tab");
    let second = focused(cx);
    assert_ne!(first, second);
    cx.simulate_click(point(px(25.0), px(25.0)), Modifiers::none());
    support::draw(cx);
    assert_eq!(
        focused(cx),
        first,
        "クリックで最初の操作要素にフォーカスする"
    );
    assert!(!cx.update(|window, cx| is_focus_visible(&first, window, cx)));
    cx.update(|window, cx| window.simulate_mouse_move(point(px(30.0), px(30.0)), cx));
    support::draw(cx);
    assert!(!cx.update(|window, cx| is_focus_visible(&first, window, cx)));
}

#[gpui::test]
fn click_on_keyboard_focused_control_hides_ring(cx: &mut TestAppContext) {
    let cx = open(cx);
    support::press(cx, "tab");
    let first = focused(cx);
    assert!(cx.update(|window, cx| is_focus_visible(&first, window, cx)));
    cx.simulate_click(point(px(25.0), px(25.0)), Modifiers::none());
    support::draw(cx);
    assert_eq!(focused(cx), first);
    assert!(!cx.update(|window, cx| is_focus_visible(&first, window, cx)));
}
