//! チェックボックスの状態とキー操作。

mod support;

use gpui::{Role, TestAppContext, Toggled, prelude::*};
use gpui_headless::{Checkbox, CheckboxState};
use std::{cell::Cell, rc::Rc};

#[gpui::test]
fn space_toggles_a_checkbox(cx: &mut TestAppContext) {
    let state = Rc::new(Cell::new(CheckboxState::Indeterminate));
    let cx = support::open(cx, {
        let state = state.clone();
        move |window, cx| {
            let set = state.clone();
            Checkbox::new("all", state.get(), "すべて")
                .on_change(move |next, _, _| set.set(next))
                .build(window, cx)
                .0
                .size(gpui::px(40.))
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
fn disabled_checkbox_does_not_change(cx: &mut TestAppContext) {
    let state = Rc::new(Cell::new(CheckboxState::Unchecked));
    let cx = support::open(cx, {
        let state = state.clone();
        move |window, cx| {
            let set = state.clone();
            Checkbox::new("c", state.get(), "無効")
                .disabled(true)
                .on_change(move |next, _, _| set.set(next))
                .build(window, cx)
                .0
                .size(gpui::px(40.))
                .into_any_element()
        }
    });
    support::tab(cx);
    support::press(cx, "space");
    assert_eq!(state.get(), CheckboxState::Unchecked);
}

#[test]
fn checkbox_exposes_state() {
    let checkbox = Checkbox::new("c", CheckboxState::Checked, "同意する").a11y_node();
    assert_eq!(checkbox.role, Role::CheckBox);
    assert_eq!(checkbox.label.as_deref(), Some("同意する"));
    assert_eq!(checkbox.toggled, Some(Toggled::True));

    let mixed = Checkbox::new("c", CheckboxState::Indeterminate, "すべて").a11y_node();
    assert_eq!(mixed.role, Role::CheckBox);
    assert_eq!(mixed.toggled, Some(Toggled::Mixed));
    assert_eq!(CheckboxState::Unchecked.next(), CheckboxState::Checked);
    assert_eq!(CheckboxState::Checked.next(), CheckboxState::Unchecked);
    assert_eq!(CheckboxState::Indeterminate.next(), CheckboxState::Checked);
}
