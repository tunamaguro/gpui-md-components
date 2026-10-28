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
