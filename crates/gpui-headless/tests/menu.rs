//! メニューの役割とキー操作。
mod support;

use gpui::{Role, TestAppContext, accesskit::HasPopup, div, prelude::*};
use gpui_headless::{Menu, MenuItem, MenuKind};
use std::{cell::Cell, rc::Rc};

#[test]
fn menu_kind_maps_roles_and_popup() {
    for (kind, role, item_role, popup) in [
        (
            MenuKind::Actions,
            Role::Menu,
            Role::MenuItem,
            HasPopup::Menu,
        ),
        (
            MenuKind::Options,
            Role::ListBox,
            Role::ListBoxOption,
            HasPopup::Listbox,
        ),
    ] {
        assert_eq!(kind.role(), role);
        assert_eq!(kind.item_role(), item_role);
        assert_eq!(kind.popup(), popup);
    }
}

#[test]
fn menu_items_are_menuitems_in_a_set() {
    let menu = Menu::new("m")
        .aria_label("編集")
        .item(MenuItem::new("切り取り"))
        .item(MenuItem::new("コピー").disabled(true));
    assert_eq!(menu.a11y_node().role, Role::Menu);
    let item = menu.item_a11y_node(1).expect("item");
    assert_eq!(item.role, Role::MenuItem);
    assert_eq!(item.position, Some((2, 2)));
    assert!(item.disabled);
    assert!(menu.item_a11y_node(2).is_none());
}

#[test]
fn option_menus_report_selection() {
    let menu = Menu::new("m")
        .aria_label("色")
        .kind(MenuKind::Options)
        .selected(Some(0))
        .item(MenuItem::new("赤"))
        .item(MenuItem::new("青"));
    assert_eq!(menu.a11y_node().role, Role::ListBox);
    assert_eq!(menu.item_a11y_node(0).and_then(|n| n.selected), Some(true));
    assert_eq!(menu.item_a11y_node(1).and_then(|n| n.selected), Some(false));
}

#[gpui::test]
fn arrows_skip_disabled_items_and_enter_selects(cx: &mut TestAppContext) {
    let chosen = Rc::new(Cell::new(None));
    let dismissed = Rc::new(Cell::new(false));
    let cx = support::open(cx, {
        let (chosen, dismissed) = (chosen.clone(), dismissed.clone());
        move |window, cx| {
            let mut menu = Menu::new("m")
                .aria_label("編集")
                .item(MenuItem::new("切り取り"));
            menu = menu.item(MenuItem::new("コピー").disabled(true));
            let on_select = chosen.clone();
            menu = menu.item(
                MenuItem::new("貼り付け").on_select(move |_, _| on_select.set(Some("貼り付け"))),
            );
            let on_dismiss = dismissed.clone();
            let popup =
                menu.on_dismiss(move |_, _| on_dismiss.set(true))
                    .build(window, cx, |row, _, _| row.base.h_8().into_any_element());
            div()
                .child(popup.place(|container| container))
                .into_any_element()
        }
    });
    support::press(cx, "down");
    support::press(cx, "enter");
    assert_eq!(chosen.get(), Some("貼り付け"));
    assert!(dismissed.get(), "選ぶと閉じる");
}
