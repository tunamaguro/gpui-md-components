//! チップの役割とキー操作。
mod support;
use gpui::{Role, TestAppContext, Toggled, prelude::*};
use gpui_headless::Chip;
use std::{cell::Cell, rc::Rc};

#[gpui::test]
fn filter_chip_toggles_with_space(cx: &mut TestAppContext) {
    let on = Rc::new(Cell::new(false));
    let cx = support::open(cx, {
        let on = on.clone();
        move |window, cx| {
            let set = on.clone();
            Chip::new("unread", "未読")
                .selected(Some(on.get()))
                .on_click(move |_, _, _| set.set(!set.get()))
                .build(window, cx)
                .0
                .size(gpui::px(40.))
                .into_any_element()
        }
    });
    support::tab(cx);
    support::press(cx, "space");
    assert!(on.get());
}

#[gpui::test]
fn input_chip_is_removed_with_backspace(cx: &mut TestAppContext) {
    let removed = Rc::new(Cell::new(false));
    let cx = support::open(cx, {
        let removed = removed.clone();
        move |window, cx| {
            let set = removed.clone();
            Chip::new("tag", "rust")
                .on_remove(move |_, _| set.set(true))
                .build(window, cx)
                .0
                .size(gpui::px(40.))
                .into_any_element()
        }
    });
    support::tab(cx);
    support::press(cx, "backspace");
    assert!(removed.get());
}

#[test]
fn chip_exposes_role_and_state() {
    let filter = Chip::new("c", "未読").selected(Some(true)).a11y_node();
    assert_eq!(filter.role, Role::CheckBox);
    assert_eq!(filter.toggled, Some(Toggled::True));
    assert_eq!(
        Chip::new("f", "未読")
            .selected(Some(false))
            .a11y_node()
            .toggled,
        Some(Toggled::False)
    );
    let assist = Chip::new("a", "予定に追加").a11y_node();
    assert_eq!(assist.role, Role::Button);
}

#[test]
fn remove_control_is_a_named_button() {
    let node = Chip::new("tag", "rust")
        .remove_label("rust を削除")
        .remove_a11y_node();
    assert_eq!(node.role, Role::Button);
    assert_eq!(node.label.as_deref(), Some("rust を削除"));
    assert!(!node.disabled);
}

#[test]
fn remove_control_name_is_optional_and_follows_disabled() {
    let node = Chip::new("tag", "rust").disabled(true).remove_a11y_node();
    assert_eq!(node.role, Role::Button);
    assert_eq!(node.label, None);
    assert!(node.disabled);
    let empty = Chip::new("tag", "rust").remove_label("").remove_a11y_node();
    assert_eq!(empty.label, None);
}

#[test]
fn selection_and_remove_handler_preserve_state_and_disabled_order() {
    use gpui::Role;
    for selected in [false, true] {
        for selectable in [false, true] {
            let chip = gpui_headless::Chip::new("c", "chip").selection(selected, selectable);
            assert_eq!(chip.is_selected(), selected);
            assert_eq!(
                chip.a11y_node().role,
                if selectable {
                    Role::CheckBox
                } else {
                    Role::Button
                }
            );
        }
    }
    let handler = |_: &mut gpui::Window, _: &mut gpui::App| {};
    assert!(
        gpui_headless::Chip::new("c", "")
            .disabled(true)
            .on_remove(handler)
            .remove_handler()
            .is_none()
    );
    assert!(
        gpui_headless::Chip::new("c", "")
            .on_remove(handler)
            .disabled(true)
            .remove_handler()
            .is_none()
    );
    assert!(
        gpui_headless::Chip::new("c", "")
            .disabled(true)
            .on_remove(handler)
            .disabled(false)
            .remove_handler()
            .is_some()
    );
}
