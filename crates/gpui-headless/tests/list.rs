//! リストのアクセシビリティとキー操作。
mod support;
use gpui::{Role, SharedString, TestAppContext, UniformListScrollHandle, div, prelude::*, px};
use gpui_headless::{LazyList, List, ListItem};
use std::{cell::RefCell, rc::Rc};

#[test]
fn list_and_items_expose_their_names_and_states() {
    let list = List::new("list").aria_label("受信トレイ");
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
    let lazy = LazyList::new("lazy", 0, &scroll, |_, _| {
        ("key".into(), ListItem::new("項目"), ())
    })
    .aria_label("連絡先");
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
            let list = (0..4).fold(List::new("list").aria_label("一覧"), |list, i| {
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
            let list = LazyList::new("lazy", 1000, &scroll, move |index, _| {
                (
                    SharedString::from(format!("k{index}")),
                    item(index, log.clone()),
                    (),
                )
            })
            .aria_label("連絡先")
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
                .fold(List::new("list").aria_label("一覧"), |list, i| {
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

#[gpui::test]
fn button_tab_stop_false_skips_tab_but_allows_explicit_focus(cx: &mut TestAppContext) {
    use gpui_headless::{Button, focus_handle};
    let log = Rc::new(RefCell::new(Vec::new()));
    let skipped = Rc::new(RefCell::new(None));
    let cx = support::open(cx, {
        let log = log.clone();
        let skipped = skipped.clone();
        move |window, cx| {
            *skipped.borrow_mut() = Some(focus_handle("button-1", window, cx));
            div()
                .children((0..3).map(|index| {
                    let log = log.clone();
                    Button::new(format!("button-{index}"), format!("操作 {index}"))
                        .tab_stop(index != 1)
                        .on_click(move |_, _, _| log.borrow_mut().push(index))
                        .build(window, cx)
                        .0
                        .size(px(40.))
                }))
                .into_any_element()
        }
    });
    support::tab(cx);
    support::press(cx, "enter");
    support::tab(cx);
    support::press(cx, "enter");
    assert_eq!(*log.borrow(), [0, 2]);
    cx.update(|window, cx| {
        window.focus(skipped.borrow().as_ref().expect("描画したボタン"), cx);
    });
    support::press(cx, "enter");
    assert_eq!(*log.borrow(), [0, 2, 1]);
}

#[gpui::test]
fn action_tab_stop_follows_the_focused_row_and_static_rows(cx: &mut TestAppContext) {
    let stops = Rc::new(RefCell::new(Vec::new()));
    let cx = support::open(cx, {
        let stops = stops.clone();
        move |window, cx| {
            stops.borrow_mut().clear();
            List::new("list")
                .item("a", ListItem::new("先頭").on_select(|_, _| {}))
                .item("b", ListItem::new("次").on_select(|_, _| {}))
                .item("static", ListItem::new("静的"))
                .item(
                    "disabled",
                    ListItem::new("無効").disabled(true).on_select(|_, _| {}),
                )
                .build(window, cx, |row, _, _| {
                    stops.borrow_mut().push(row.action_tab_stop);
                    row.base.size(px(40.)).into_any_element()
                })
                .finish(|base, _| base)
        }
    });
    assert_eq!(*stops.borrow(), [true, false, true, true]);
    support::tab(cx);
    support::press(cx, "down");
    assert_eq!(*stops.borrow(), [false, true, true, true]);
}
