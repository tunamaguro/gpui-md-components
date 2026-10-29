//! Checkbox / Switch のキー操作と a11y 情報。

mod support;

use std::{cell::Cell, rc::Rc};

use gpui::{Role, TestAppContext, Toggled, prelude::*};
use md3_components::selection::{Checkbox, CheckboxState, Switch};

#[gpui::test]
fn space_toggles_a_checkbox(cx: &mut TestAppContext) {
    let state = Rc::new(Cell::new(CheckboxState::Indeterminate));
    let cx = support::open(cx, {
        let state = state.clone();
        move |_, _| {
            let set = state.clone();
            Checkbox::new("all", state.get(), "すべて")
                .on_change(move |next, _, _| set.set(next))
                .into_any_element()
        }
    });
    support::tab(cx);
    support::press(cx, "space");
    assert_eq!(state.get(), CheckboxState::Checked);
    support::press(cx, "space");
    assert_eq!(state.get(), CheckboxState::Unchecked);
}

#[gpui::test]
fn space_toggles_a_switch(cx: &mut TestAppContext) {
    let on = Rc::new(Cell::new(false));
    let cx = support::open(cx, {
        let on = on.clone();
        move |_, _| {
            let set = on.clone();
            Switch::new("wifi", on.get(), "Wi-Fi")
                .on_change(move |next, _, _| set.set(next))
                .into_any_element()
        }
    });
    support::tab(cx);
    support::press(cx, "space");
    assert!(on.get());
    support::press(cx, "space");
    assert!(!on.get());
}

#[cfg(debug_assertions)]
#[gpui::test]
fn switch_cancels_keyboard_press_when_focus_moves_away(cx: &mut TestAppContext) {
    use gpui::{InputEvent, KeyDownEvent, KeyUpEvent, Keystroke, div, px};
    use md3_components::theme::md3_tokens::component::switch as tokens;

    cx.update(|cx| cx.set_reduce_motion(true));
    let changed = Rc::new(Cell::new(false));
    let cx = support::open(cx, {
        let changed = changed.clone();
        move |_, _| {
            let changed = changed.clone();
            div()
                .child(
                    Switch::new("wifi", false, "Wi-Fi").on_change(move |_, _, _| changed.set(true)),
                )
                .child(Checkbox::new("next", CheckboxState::Unchecked, "次"))
                .into_any_element()
        }
    });
    // GPUI のフォーカス通知はアクティブなウィンドウで発火する。
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
            cx.debug_bounds("md3-switch-handle-wifi")
                .expect("つまみ")
                .size
                .width,
            px(tokens::PRESSED_HANDLE_WIDTH),
            "キーを押している間は押下寸法"
        );
        support::tab(cx);
        assert_eq!(
            cx.debug_bounds("md3-switch-handle-wifi")
                .expect("つまみ")
                .size
                .width,
            px(tokens::UNSELECTED_HANDLE_WIDTH),
            "キー解放を受け取れなくてもフォーカス喪失時に戻る"
        );
        cx.update(|window, cx| {
            window.dispatch_event(KeyUpEvent { keystroke }.to_platform_input(), cx);
        });
        support::draw(cx);
        assert!(!changed.get(), "キャンセルした押下で選択は変わらない");
    }
}

#[gpui::test]
fn disabled_checkbox_does_not_change(cx: &mut TestAppContext) {
    let state = Rc::new(Cell::new(CheckboxState::Unchecked));
    let cx = support::open(cx, {
        let state = state.clone();
        move |_, _| {
            let set = state.clone();
            Checkbox::new("c", state.get(), "無効")
                .disabled(true)
                .on_change(move |next, _, _| set.set(next))
                .into_any_element()
        }
    });
    support::tab(cx);
    support::press(cx, "space");
    assert_eq!(state.get(), CheckboxState::Unchecked);
}

#[test]
fn checkbox_and_switch_expose_toggled_state() {
    let checkbox = Checkbox::new("c", CheckboxState::Checked, "同意する").a11y_node();
    assert_eq!(checkbox.role, Role::CheckBox);
    assert_eq!(checkbox.label.as_deref(), Some("同意する"));
    assert_eq!(checkbox.toggled, Some(Toggled::True));

    let switch = Switch::new("s", false, "通知").a11y_node();
    assert_eq!(switch.role, Role::Switch);
    assert_eq!(switch.toggled, Some(Toggled::False));
}
