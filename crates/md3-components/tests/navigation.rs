//! Tabs・List・NavigationBar のキー操作（矢印キーでの移動、Tab で止まる場所）。
//!
//! どの項目にフォーカスがあるかは、Enter で選ばれた項目の記録で確かめる。

mod support;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gpui::{SharedString, TestAppContext, UniformListScrollHandle, div, prelude::*, px};
use md3_components::IconName;
use md3_components::containment::{LazyList, List, ListItem};
use md3_components::navigation::{NavigationBar, NavigationItem, Tab, Tabs};

#[gpui::test]
fn arrows_select_the_next_tab(cx: &mut TestAppContext) {
    let selected = Rc::new(Cell::new(0));
    let cx = support::open(cx, {
        let selected = selected.clone();
        move |_, _| {
            let on_change = selected.clone();
            Tabs::new("tabs", "表示", selected.get())
                .tab(Tab::new("写真"))
                .tab(Tab::new("アルバム").disabled(true))
                .tab(Tab::new("共有"))
                .on_change(move |index, _, _| on_change.set(index))
                .into_any_element()
        }
    });
    support::tab(cx);
    support::press(cx, "right");
    assert_eq!(selected.get(), 2, "無効なタブを飛ばす");
    support::press(cx, "right");
    assert_eq!(selected.get(), 0, "末尾から先頭に回る");
    support::press(cx, "end");
    assert_eq!(selected.get(), 2);
    support::press(cx, "home");
    assert_eq!(selected.get(), 0);
}

#[gpui::test]
fn navigation_bar_moves_focus_and_selects_on_enter(cx: &mut TestAppContext) {
    let selected = Rc::new(Cell::new(1));
    let cx = support::open(cx, {
        let selected = selected.clone();
        move |_, _| {
            let on_change = selected.clone();
            NavigationBar::new("nav", "メイン")
                .item(NavigationItem::new(IconName::Home, "ホーム"))
                .item(NavigationItem::new(IconName::Search, "検索"))
                .item(NavigationItem::new(IconName::Settings, "設定"))
                .selected(Some(selected.get()))
                .on_change(move |index, _, _| on_change.set(index))
                .into_any_element()
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

/// 選ばれた項目の記録。
fn recorder() -> (
    Rc<RefCell<Vec<usize>>>,
    impl Fn(usize) -> ListItem + Clone + 'static,
) {
    let log = Rc::new(RefCell::new(Vec::new()));
    let item = {
        let log = log.clone();
        move |index: usize| {
            let log = log.clone();
            ListItem::new(format!("項目 {index}"))
                .disabled(index == 1)
                .on_select(move |_, _| log.borrow_mut().push(index))
        }
    };
    (log, item)
}

#[gpui::test]
fn list_arrows_stop_at_the_ends(cx: &mut TestAppContext) {
    let (log, item) = recorder();
    let cx = support::open(cx, move |_, _| {
        (0..4)
            .fold(List::new("list", "受信トレイ"), |list, index| {
                list.item(SharedString::from(format!("k{index}")), item(index))
            })
            .into_any_element()
    });
    support::tab(cx);
    support::press(cx, "enter");
    support::press(cx, "down");
    support::press(cx, "enter");
    support::press(cx, "down");
    support::press(cx, "down");
    support::press(cx, "enter");
    support::press(cx, "up");
    support::press(cx, "up");
    support::press(cx, "enter");
    support::press(cx, "up");
    support::press(cx, "enter");
    assert_eq!(
        *log.borrow(),
        [0, 2, 3, 0, 0],
        "無効な項目を飛ばし、端では止まる"
    );
}

#[gpui::test]
fn lazy_list_scrolls_to_items_that_are_not_drawn(cx: &mut TestAppContext) {
    let (log, item) = recorder();
    let scroll = UniformListScrollHandle::new();
    let cx = support::open(cx, move |_, _| {
        let item = item.clone();
        div()
            .h(px(150.0))
            .w(px(300.0))
            .child(
                LazyList::new("lazy", "連絡先", 1000, &scroll, move |index, _| {
                    (SharedString::from(format!("k{index}")), item(index))
                })
                .size_full(),
            )
            .into_any_element()
    });
    support::tab(cx);
    support::press(cx, "enter");
    support::press(cx, "end");
    support::press(cx, "enter");
    support::press(cx, "up");
    support::press(cx, "enter");
    support::press(cx, "home");
    support::press(cx, "down");
    support::press(cx, "enter");
    assert_eq!(*log.borrow(), [0, 999, 998, 2]);
}

#[gpui::test]
fn modal_drawer_takes_focus_and_escape_restores_it(cx: &mut TestAppContext) {
    use md3_components::buttons::Button;
    use md3_components::navigation::{NavigationDrawer, NavigationDrawerKind};

    let open = Rc::new(Cell::new(false));
    let selected = Rc::new(Cell::new(0));
    let cx = support::open(cx, {
        let open = open.clone();
        let selected = selected.clone();
        move |_, _| {
            let toggle = open.clone();
            let close = open.clone();
            let on_change = selected.clone();
            div()
                .child(Button::new("opener", "メニュー").on_click(move |_, _, _| toggle.set(true)))
                .when(open.get(), |this| {
                    this.child(
                        NavigationDrawer::new("drawer", "メイン")
                            .kind(NavigationDrawerKind::Modal)
                            .item(NavigationItem::new(IconName::Inbox, "受信トレイ"))
                            .item(NavigationItem::new(IconName::Send, "送信済み"))
                            .selected(Some(0))
                            .on_change(move |index, _, _| on_change.set(index))
                            .on_dismiss(move |_, _| close.set(false)),
                    )
                })
                .into_any_element()
        }
    });
    support::tab(cx);
    support::press(cx, "enter");
    assert!(open.get());

    // 開くと選択中の項目にフォーカスし、↓ と Enter で選べる。
    support::press(cx, "down");
    support::press(cx, "enter");
    assert_eq!(selected.get(), 1);

    // Esc で閉じ、開いたボタンにフォーカスが戻る。
    support::press(cx, "escape");
    assert!(!open.get());
    support::press(cx, "enter");
    assert!(open.get(), "フォーカスは開いたボタンに戻る");
}
