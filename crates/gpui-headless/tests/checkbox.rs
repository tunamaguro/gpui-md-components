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
            Checkbox::new("all", state.get())
                .aria_label("すべて")
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
            Checkbox::new("c", state.get())
                .aria_label("無効")
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
    let checkbox = Checkbox::new("c", CheckboxState::Checked)
        .aria_label("同意する")
        .a11y_node();
    assert_eq!(checkbox.role, Role::CheckBox);
    assert_eq!(checkbox.label.as_deref(), Some("同意する"));
    assert_eq!(checkbox.toggled, Some(Toggled::True));

    let mixed = Checkbox::new("c", CheckboxState::Indeterminate)
        .aria_label("すべて")
        .a11y_node();
    assert_eq!(mixed.role, Role::CheckBox);
    assert_eq!(mixed.toggled, Some(Toggled::Mixed));
    assert_eq!(CheckboxState::Unchecked.next(), CheckboxState::Checked);
    assert_eq!(CheckboxState::Checked.next(), CheckboxState::Unchecked);
    assert_eq!(CheckboxState::Indeterminate.next(), CheckboxState::Checked);
}

#[test]
fn checkbox_name_is_optional() {
    assert_eq!(
        Checkbox::new("c", CheckboxState::Unchecked)
            .a11y_node()
            .label,
        None
    );
    assert_eq!(
        Checkbox::new("c", CheckboxState::Unchecked)
            .aria_label("")
            .a11y_node()
            .label,
        None
    );
    assert_eq!(
        Checkbox::new("c", CheckboxState::Unchecked)
            .aria_label("同意")
            .a11y_node()
            .label
            .as_deref(),
        Some("同意")
    );
}

#[test]
fn state_accessor_returns_each_checkbox_state() {
    for state in [
        gpui_headless::CheckboxState::Unchecked,
        gpui_headless::CheckboxState::Checked,
        gpui_headless::CheckboxState::Indeterminate,
    ] {
        assert_eq!(gpui_headless::Checkbox::new("c", state).state(), state);
    }
}
