//! リストのアクセシビリティとキー操作。
mod support;
use gpui::{Role, SharedString, TestAppContext, UniformListScrollHandle, div, prelude::*, px};
use gpui_headless::{LazyList, List, ListItem};
use std::{cell::RefCell, rc::Rc};

#[test]
fn list_and_items_expose_their_names_and_states() {
    let list = List::new("list", "受信トレイ");
    assert_eq!(list.a11y_node().role, Role::List);
    assert_eq!(list.a11y_node().label.as_deref(), Some("受信トレイ"));
    let node = ListItem::new("受信トレイ")
        .selected(true)
        .disabled(true)
        .a11y_node(1, 3);
    assert_eq!(node.role, Role::ListItem);
    assert_eq!(node.label.as_deref(), Some("受信トレイ"));
    assert_eq!(node.selected, Some(true));
    assert_eq!(node.position, Some((2, 3)));
    assert!(node.disabled);
    let scroll = UniformListScrollHandle::new();
    let lazy = LazyList::new("lazy", "連絡先", 0, &scroll, |_, _| {
        ("key".into(), ListItem::new("項目"), ())
    });
    assert_eq!(lazy.a11y_node().role, Role::List);
    assert_eq!(lazy.a11y_node().label.as_deref(), Some("連絡先"));
}
fn item(index: usize, log: Rc<RefCell<Vec<usize>>>) -> ListItem {
    ListItem::new(format!("項目 {index}"))
        .disabled(index == 1)
        .on_select(move |_, _| log.borrow_mut().push(index))
}
#[gpui::test]
fn list_keys_skip_disabled_and_stop_at_the_ends(cx: &mut TestAppContext) {
    let log = Rc::new(RefCell::new(Vec::new()));
    let cx = support::open(cx, {
        let log = log.clone();
        move |window, cx| {
            let list = (0..4).fold(List::new("list", "一覧"), |list, i| {
                list.item(format!("k{i}"), item(i, log.clone()))
            });
            list.build(window, cx, |row, _, _| {
                row.base.size(px(40.)).into_any_element()
            })
            .finish(|base, _| base)
        }
    });
    support::tab(cx);
    for key in [
        "enter", "down", "enter", "down", "down", "enter", "up", "up", "enter", "up", "enter",
        "end", "enter", "home", "enter", "left", "enter",
    ] {
        support::press(cx, key);
    }
    assert_eq!(*log.borrow(), [0, 2, 3, 0, 0, 3, 0, 0]);
}
#[gpui::test]
fn lazy_list_keys_reveal_offscreen_items(cx: &mut TestAppContext) {
    let log = Rc::new(RefCell::new(Vec::new()));
    let scroll = UniformListScrollHandle::new();
    let cx = support::open(cx, {
        let log = log.clone();
        move |window, cx| {
            let log = log.clone();
            let list = LazyList::new("lazy", "連絡先", 1000, &scroll, move |index, _| {
                (
                    SharedString::from(format!("k{index}")),
                    item(index, log.clone()),
                    (),
                )
            })
            .build(window, cx, |row, (), _, _| {
                row.base.w_full().h(px(40.)).into_any_element()
            });
            div()
                .h(px(150.))
                .w(px(300.))
                .child(list.size_full())
                .into_any_element()
        }
    });
    support::tab(cx);
    for key in [
        "enter", "end", "enter", "up", "enter", "home", "down", "enter",
    ] {
        support::press(cx, key);
    }
    assert_eq!(*log.borrow(), [0, 999, 998, 2]);
}

#[gpui::test]
fn selected_item_is_tab_stop_and_key_survives_reordering(cx: &mut TestAppContext) {
    use std::cell::Cell;
    let log = Rc::new(RefCell::new(Vec::new()));
    let reversed = Rc::new(Cell::new(false));
    let cx = support::open(cx, {
        let log = log.clone();
        let reversed = reversed.clone();
        move |window, cx| {
            let order = if reversed.get() { [2, 1, 0] } else { [0, 1, 2] };
            let list = order
                .into_iter()
                .fold(List::new("list", "一覧"), |list, i| {
                    list.item(format!("key-{i}"), item(i, log.clone()).selected(i == 2))
                });
            list.build(window, cx, |row, _, _| {
                row.base.size(px(40.)).into_any_element()
            })
            .finish(|base, _| base)
        }
    });
    support::tab(cx);
    support::press(cx, "enter");
    reversed.set(true);
    support::draw(cx);
    support::press(cx, "enter");
    support::press(cx, "down");
    support::press(cx, "enter");
    assert_eq!(*log.borrow(), [2, 2, 0]);
}
