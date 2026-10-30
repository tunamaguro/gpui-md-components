//! 選択ボタンの状態とキー操作。
mod support;
use gpui::{Role, TestAppContext, Toggled, prelude::*};
use gpui_headless::SegmentedButton;
use std::{cell::Cell, rc::Rc};

#[gpui::test]
fn segmented_button_moves_with_arrows(cx: &mut TestAppContext) {
    let selected = Rc::new(Cell::new(0));
    let cx = support::open(cx, {
        let selected = selected.clone();
        move |window, cx| {
            let set = selected.clone();
            SegmentedButton::new("view", "表示")
                .segment("日", false)
                .segment("週", false)
                .segment("月", false)
                .selected([selected.get()])
                .on_change(move |index, _, _| set.set(index))
                .build(window, cx, |item, _, _| {
                    item.base.size(gpui::px(40.)).into_any_element()
                })
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
fn single_select_is_a_radio_group() {
    let button = SegmentedButton::new("s", "表示")
        .segment("日", false)
        .segment("週", false)
        .selected([1]);
    assert_eq!(button.a11y_node().role, Role::RadioGroup);
    let node = button.segment_a11y_node(1).expect("segment");
    assert_eq!(node.role, Role::RadioButton);
    assert_eq!(node.toggled, Some(Toggled::True));
    assert_eq!(node.selected, None);
    assert_eq!(node.position, Some((2, 2)));
}
