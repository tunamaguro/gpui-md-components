//! RadioGroup / SegmentedButton のキー操作（矢印での移動と選択）と a11y 情報。

mod support;

use std::{cell::Cell, rc::Rc};

use gpui::{Role, TestAppContext, prelude::*};
use md3_components::buttons::{Segment, SegmentedButton};
use md3_components::selection::RadioGroup;

fn radio(selected: &Rc<Cell<Option<usize>>>) -> RadioGroup {
    let set = selected.clone();
    RadioGroup::new("size", "サイズ")
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
        move |_, _| radio(&selected).into_any_element()
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
        move |_, _| radio(&selected).into_any_element()
    });
    support::tab(cx);
    // フォーカスが選択中の「大」にあれば、上矢印で「小」に移る（「中」は無効）。
    support::press(cx, "up");
    assert_eq!(selected.get(), Some(0));
}

#[gpui::test]
fn segmented_button_moves_with_arrows(cx: &mut TestAppContext) {
    let selected = Rc::new(Cell::new(0));
    let cx = support::open(cx, {
        let selected = selected.clone();
        move |_, _| {
            let set = selected.clone();
            SegmentedButton::new("view", "表示")
                .segment(Segment::new("日"))
                .segment(Segment::new("週"))
                .segment(Segment::new("月"))
                .selected([selected.get()])
                .on_change(move |index, _, _| set.set(index))
                .into_any_element()
        }
    });
    support::tab(cx);
    support::press(cx, "right");
    assert_eq!(selected.get(), 1);
    support::press(cx, "left");
    support::press(cx, "left");
    assert_eq!(selected.get(), 2);
}

#[test]
fn radio_group_exposes_roles_and_selection() {
    let selected = Rc::new(Cell::new(Some(0)));
    let group = radio(&selected);
    assert_eq!(group.a11y_node().role, Role::RadioGroup);
    let first = group.option_a11y_node(0).expect("option");
    assert_eq!(first.role, Role::RadioButton);
    assert_eq!(first.selected, Some(true));
    assert_eq!(first.position, Some((1, 3)));
}
