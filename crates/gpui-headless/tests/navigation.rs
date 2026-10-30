//! 行き先の状態とキー操作。
mod support;
use gpui::{Role, TestAppContext, prelude::*, px};
use gpui_headless::Navigation;
use std::{cell::Cell, rc::Rc};

#[gpui::test]
fn navigation_bar_moves_focus_and_selects_on_enter(cx: &mut TestAppContext) {
    let selected = Rc::new(Cell::new(1));
    let cx = support::open(cx, {
        let selected = selected.clone();
        move |window, cx| {
            let on_change = selected.clone();
            let nav = Navigation::new("nav", "メイン")
                .item("ホーム", false)
                .item("検索", false)
                .item("設定", false)
                .selected(Some(selected.get()))
                .on_change(move |index, _, _| on_change.set(index))
                .build(window, cx, |item, _, _| {
                    item.base.size(px(40.)).into_any_element()
                });
            nav.base.flex().children(nav.items).into_any_element()
        }
    });
    // Tab は選択中の項目で止まる。
    support::tab(cx);
    support::press(cx, "enter");
    assert_eq!(selected.get(), 1);
    // 矢印キーはフォーカスだけを移し、Enter で選ぶ。
    support::press(cx, "right");
    assert_eq!(selected.get(), 1, "矢印キーでは選ばない");
    support::press(cx, "enter");
    assert_eq!(selected.get(), 2);
    support::press(cx, "home");
    support::press(cx, "space");
    assert_eq!(selected.get(), 0);
}

#[test]
fn landmarks_and_modal_dialogs_expose_roles() {
    let nav = Navigation::new("n", "メイン");
    assert_eq!(nav.a11y_node().role, Role::Navigation);
    assert_eq!(nav.a11y_node().label.as_deref(), Some("メイン"));
    assert!(!nav.a11y_node().modal);
    let modal = Navigation::new("d", "メイン").dialog(true);
    assert_eq!(modal.a11y_node().role, Role::Dialog);
    assert!(modal.a11y_node().modal);
}

#[test]
fn items_expose_position_selection_and_disabled_state() {
    let nav = Navigation::new("n", "メイン")
        .item("受信トレイ", false)
        .item("設定", true)
        .selected(Some(0));
    let first = nav.item_a11y_node(0).expect("item");
    assert_eq!(first.role, Role::Tab);
    assert_eq!(first.label.as_deref(), Some("受信トレイ"));
    assert_eq!(first.selected, Some(true));
    assert_eq!(first.position, Some((1, 2)));
    let second = nav.item_a11y_node(1).expect("item");
    assert!(second.disabled);
    assert_eq!(second.selected, Some(false));
}
