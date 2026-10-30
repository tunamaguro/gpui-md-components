//! 見た目を持たない操作要素のフォーカスと追従。
mod support;

use std::{cell::RefCell, rc::Rc};

use gpui::{
    FocusHandle, KeyBinding, Role, ScrollHandle, TestAppContext, VisualTestContext, actions, div,
    point, prelude::*, px,
};
use gpui_headless::{A11yExt, A11yNode, ScrollScope, control, focus_handle, is_focus_visible};

actions!(
    control_test,
    [
        /// 次の要素に移る。
        Next
    ]
);

fn item(
    id: &'static str,
    disabled: bool,
    tab_stop: bool,
    window: &mut gpui::Window,
    cx: &mut gpui::App,
) -> gpui::Stateful<gpui::Div> {
    let (base, state) = control(
        &id.into(),
        disabled,
        tab_stop,
        window,
        cx,
        px(5.0),
        id.into(),
    );
    assert_eq!(state.disabled, disabled);
    base.a11y(A11yNode::new(Role::Button).label(id).disabled(disabled))
        .w(px(40.0))
        .h(px(40.0))
}

fn focused(cx: &mut VisualTestContext) -> FocusHandle {
    cx.update(|window, cx| window.focused(cx).expect("フォーカス"))
}

#[gpui::test]
fn tab_skips_disabled_and_non_tab_stops_and_latches_focus(cx: &mut TestAppContext) {
    cx.update(|cx| cx.bind_keys([KeyBinding::new("tab", Next, None)]));
    let handles = Rc::new(RefCell::new(Vec::new()));
    let stored = handles.clone();
    let cx = support::open(cx, move |window, cx| {
        let root = focus_handle("root", window, cx).tab_stop(false);
        if window.focused(cx).is_none() {
            window.focus(&root, cx);
        }
        *stored.borrow_mut() = vec![
            focus_handle("first", window, cx),
            focus_handle("last", window, cx),
        ];
        let first = item("first", false, true, window, cx);
        let disabled = item("disabled", true, true, window, cx);
        let skipped = item("skipped", false, false, window, cx);
        let last = item("last", false, true, window, cx);
        div()
            .id("root")
            .track_focus(&root)
            .on_action(|_: &Next, window, cx| window.focus_next(cx))
            .child(first)
            .child(disabled)
            .child(skipped)
            .child(last)
            .into_any_element()
    });
    support::press(cx, "tab");
    let first = focused(cx);
    assert_eq!(first, handles.borrow()[0]);
    assert!(cx.update(|window, cx| is_focus_visible(&first, window, cx)));
    cx.update(|window, cx| window.simulate_mouse_move(point(px(30.0), px(30.0)), cx));
    support::draw(cx);
    assert!(cx.update(|window, cx| is_focus_visible(&first, window, cx)));
    support::press(cx, "tab");
    assert_eq!(focused(cx), handles.borrow()[1]);
}

#[gpui::test]
fn focused_control_is_revealed_in_scroll_scope(cx: &mut TestAppContext) {
    cx.update(|cx| cx.bind_keys([KeyBinding::new("tab", Next, None)]));
    let scroll = ScrollHandle::new();
    let handles = Rc::new(RefCell::new(Vec::new()));
    let stored = handles.clone();
    let cx = support::open(cx, {
        let scroll = scroll.clone();
        move |window, cx| {
            let root = focus_handle("root", window, cx).tab_stop(false);
            if window.focused(cx).is_none() {
                window.focus(&root, cx);
            }
            *stored.borrow_mut() = vec![
                focus_handle("first", window, cx),
                focus_handle("last", window, cx),
            ];
            let first = item("first", false, true, window, cx);
            let last = item("last", false, true, window, cx);
            div()
                .id("root")
                .track_focus(&root)
                .on_action(|_: &Next, window, cx| window.focus_next(cx))
                .child(ScrollScope::new(
                    scroll.clone(),
                    div()
                        .id("scroll")
                        .w(px(120.0))
                        .h(px(100.0))
                        .overflow_y_scroll()
                        .track_scroll(&scroll)
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .p(px(10.0))
                                .gap(px(150.0))
                                .child(first)
                                .child(last),
                        ),
                ))
                .into_any_element()
        }
    });
    support::press(cx, "tab");
    support::press(cx, "tab");
    assert_eq!(focused(cx), handles.borrow()[1]);
    assert!(scroll.offset().y < px(0.0));
}
