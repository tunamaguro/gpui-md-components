//! スイッチの状態とキー操作。

mod support;

use gpui::{Role, TestAppContext, Toggled, prelude::*};
use gpui_headless::Switch;
use std::{cell::Cell, rc::Rc};

#[gpui::test]
fn space_toggles_a_switch(cx: &mut TestAppContext) {
    let on = Rc::new(Cell::new(false));
    let cx = support::open(cx, {
        let on = on.clone();
        move |window, cx| {
            let set = on.clone();
            Switch::new("wifi", on.get(), "Wi-Fi")
                .on_change(move |next, _, _| set.set(next))
                .build(window, cx)
                .0
                .size(gpui::px(40.))
                .into_any_element()
        }
    });
    support::tab(cx);
    support::press(cx, "space");
    assert!(on.get());
    support::press(cx, "space");
    assert!(!on.get());
}

#[test]
fn switch_exposes_toggled_state() {
    let switch = Switch::new("s", false, "通知").a11y_node();
    assert_eq!(switch.role, Role::Switch);
    assert_eq!(switch.toggled, Some(Toggled::False));
}

#[test]
fn selected_switch_reports_toggled() {
    let node = Switch::new("s", true, "Wi-Fi").a11y_node();
    assert_eq!(node.role, Role::Switch);
    assert_eq!(node.toggled, Some(Toggled::True));
}

#[cfg(debug_assertions)]
#[gpui::test]
fn switch_releases_press_when_focus_moves(cx: &mut TestAppContext) {
    use gpui::{InputEvent, KeyDownEvent, KeyUpEvent, Keystroke, div, px};

    let changed = Rc::new(Cell::new(false));
    let cx = support::open(cx, {
        let changed = changed.clone();
        move |window, cx| {
            let changed = changed.clone();
            let (base, state) = Switch::new("wifi", false, "Wi-Fi")
                .on_change(move |_, _, _| changed.set(true))
                .build(window, cx);
            let next = Switch::new("next", false, "次").build(window, cx).0;
            div()
                .child(
                    base.size(px(if state.pressed { 40.0 } else { 20.0 }))
                        .debug_selector(|| "switch".to_string()),
                )
                .child(next.size(px(20.0)))
                .into_any_element()
        }
    });
    cx.update(|window, _| window.activate_window());
    support::draw(cx);
    for key in ["space", "enter"] {
        support::tab(cx);
        let keystroke = Keystroke::parse(key).expect("有効なキー");
        cx.update(|window, cx| {
            window.dispatch_event(
                KeyDownEvent {
                    keystroke: keystroke.clone(),
                    is_held: false,
                    prefer_character_input: false,
                }
                .to_platform_input(),
                cx,
            );
        });
        support::draw(cx);
        assert_eq!(
            cx.debug_bounds("switch").expect("スイッチ").size.width,
            px(40.0)
        );
        support::tab(cx);
        assert_eq!(
            cx.debug_bounds("switch").expect("スイッチ").size.width,
            px(20.0)
        );
        cx.update(|window, cx| {
            window.dispatch_event(KeyUpEvent { keystroke }.to_platform_input(), cx);
        });
        support::draw(cx);
        assert!(!changed.get());
    }
}
