//! ラジオグループの状態とキー操作。
mod support;
use gpui::{Role, TestAppContext, Toggled, prelude::*};
use gpui_headless::RadioGroup;
use std::{cell::Cell, rc::Rc};

fn radio(selected: &Rc<Cell<Option<usize>>>) -> RadioGroup {
    let set = selected.clone();
    RadioGroup::new("size")
        .aria_label("サイズ")
        .option("小")
        .disabled_option("中")
        .option("大")
        .selected(selected.get())
        .on_change(move |index, _, _| set.set(Some(index)))
}

#[gpui::test]
fn arrows_move_selection_and_skip_disabled(cx: &mut TestAppContext) {
    let selected = Rc::new(Cell::new(Some(0)));
    let cx = support::open(cx, {
        let selected = selected.clone();
        move |window, cx| {
            radio(&selected)
                .build(window, cx, |item, _, _| {
                    item.base.size(gpui::px(40.)).into_any_element()
                })
                .into_any_element()
        }
    });
    support::tab(cx);
    support::press(cx, "down");
    assert_eq!(selected.get(), Some(2));
    support::press(cx, "down");
    assert_eq!(selected.get(), Some(0), "wraps around");
    support::press(cx, "up");
    assert_eq!(selected.get(), Some(2));
}

#[gpui::test]
fn tab_lands_on_the_selected_radio(cx: &mut TestAppContext) {
    let selected = Rc::new(Cell::new(Some(2)));
    let cx = support::open(cx, {
        let selected = selected.clone();
        move |window, cx| {
            radio(&selected)
                .build(window, cx, |item, _, _| {
                    item.base.size(gpui::px(40.)).into_any_element()
                })
                .into_any_element()
        }
    });
    support::tab(cx);
    // フォーカスが選択中の「大」にあれば、上矢印で「小」に移る（「中」は無効）。
    support::press(cx, "up");
    assert_eq!(selected.get(), Some(0));
}

#[test]
fn radio_group_exposes_roles_and_selection() {
    let selected = Rc::new(Cell::new(Some(0)));
    let group = radio(&selected);
    assert_eq!(group.a11y_node().role, Role::RadioGroup);
    let first = group.option_a11y_node(0).expect("option");
    assert_eq!(first.role, Role::RadioButton);
    assert_eq!(first.toggled, Some(Toggled::True));
    assert_eq!(
        first.selected, None,
        "ラジオの選択は checked（toggled）で伝える"
    );
    assert_eq!(first.position, Some((1, 3)));
    let third = group.option_a11y_node(2).expect("option");
    assert_eq!(third.toggled, Some(Toggled::False));
}

#[test]
fn disabled_option_is_reported() {
    let group = RadioGroup::new("g")
        .aria_label("サイズ")
        .disabled_option("大");
    assert!(group.option_a11y_node(0).expect("option").disabled);
}

#[test]
fn group_name_is_optional_and_can_be_cleared() {
    assert_eq!(RadioGroup::new("g").a11y_node().label, None);
    assert_eq!(
        RadioGroup::new("g")
            .aria_label("サイズ")
            .a11y_node()
            .label
            .as_deref(),
        Some("サイズ")
    );
    assert_eq!(RadioGroup::new("g").aria_label("").a11y_node().label, None);
}
