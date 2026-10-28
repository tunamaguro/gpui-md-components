//! Dialog・Menu のフォーカス（閉じ込め、Esc、閉じた後の復帰）とキー操作。
//!
//! どの要素にフォーカスがあるかは、Enter で押されたボタンの記録で確かめる。

mod support;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gpui::{TestAppContext, div, prelude::*};
use md3_components::buttons::Button;
use md3_components::overlays::{Dialog, DialogAction, Menu, MenuItem};

/// 押されたボタンの記録と、オーバーレイの開閉の状態。
#[derive(Clone, Default)]
struct Log {
    open: Rc<Cell<bool>>,
    pressed: Rc<RefCell<Vec<&'static str>>>,
}

impl Log {
    fn record(&self, name: &'static str) -> impl Fn(&mut gpui::Window, &mut gpui::App) + use<> {
        let pressed = self.pressed.clone();
        move |_, _| pressed.borrow_mut().push(name)
    }

    fn close(&self) -> impl Fn(&mut gpui::Window, &mut gpui::App) + use<> {
        let open = self.open.clone();
        move |_, _| open.set(false)
    }

    fn take(&self) -> Vec<&'static str> {
        std::mem::take(&mut self.pressed.borrow_mut())
    }
}

fn opener(log: &Log) -> Button {
    let open = log.open.clone();
    let pressed = log.pressed.clone();
    Button::new("opener", "開く").on_click(move |_, _, _| {
        pressed.borrow_mut().push("opener");
        open.set(!open.get());
    })
}

fn dialog(
    log: &Log,
) -> impl Fn(&mut gpui::Window, &mut gpui::Context<support::Harness>) -> gpui::AnyElement + use<> {
    let log = log.clone();
    move |_, _| {
        let record = log.record("inner");
        div()
            .child(opener(&log))
            .child(Button::new("behind", "後ろ").on_click({
                let record = log.record("behind");
                move |_, window, cx| record(window, cx)
            }))
            .when(log.open.get(), |this| {
                let close = log.close();
                let cancel = log.record("cancel");
                let close_cancel = log.close();
                this.child(
                    Dialog::new("confirm", "確認")
                        .child(
                            Button::new("inner", "中")
                                .on_click(move |_, window, cx| record(window, cx)),
                        )
                        .action(DialogAction::new("キャンセル", move |window, cx| {
                            cancel(window, cx);
                            close_cancel(window, cx);
                        }))
                        .on_dismiss(close),
                )
            })
            .into_any_element()
    }
}

#[gpui::test]
fn dialog_traps_focus_and_restores_it(cx: &mut TestAppContext) {
    let log = Log::default();
    let cx = support::open(cx, dialog(&log));
    support::tab(cx);
    support::press(cx, "enter");
    assert!(log.open.get());
    assert_eq!(log.take(), ["opener"]);

    // 開くとダイアログの中の最初の要素にフォーカスする。
    support::press(cx, "enter");
    assert_eq!(log.take(), ["inner"]);

    // Tab はダイアログの中で回り、後ろのボタンには行かない。
    support::press(cx, "tab");
    support::press(cx, "tab");
    support::press(cx, "enter");
    assert_eq!(log.take(), ["inner"], "末尾から先頭に回る");
    support::press(cx, "shift-tab");
    support::press(cx, "enter");
    assert_eq!(log.take(), ["cancel"], "先頭から末尾に回る");

    // 操作のボタンで閉じると、開いたボタンにフォーカスが戻る。
    assert!(!log.open.get());
    support::draw(cx);
    support::press(cx, "enter");
    assert_eq!(log.take(), ["opener"]);
    assert!(log.open.get());
}

#[gpui::test]
fn escape_closes_the_dialog_and_restores_focus(cx: &mut TestAppContext) {
    let log = Log::default();
    let cx = support::open(cx, dialog(&log));
    support::tab(cx);
    support::press(cx, "enter");
    log.take();

    support::press(cx, "escape");
    assert!(!log.open.get());
    support::press(cx, "enter");
    assert_eq!(log.take(), ["opener"], "フォーカスは開いたボタンに戻る");
}

fn menu(
    log: &Log,
) -> impl Fn(&mut gpui::Window, &mut gpui::Context<support::Harness>) -> gpui::AnyElement + use<> {
    let log = log.clone();
    move |_, _| {
        div()
            .relative()
            .child(opener(&log))
            .when(log.open.get(), |this| {
                this.child(
                    Menu::new("actions", "操作")
                        .item(MenuItem::new("切り取り").on_select(log.record("cut")))
                        .item(MenuItem::new("コピー").disabled(true))
                        .item(MenuItem::new("貼り付け").on_select(log.record("paste")))
                        .on_dismiss(log.close()),
                )
            })
            .into_any_element()
    }
}

#[gpui::test]
fn menu_arrows_skip_disabled_items_and_select(cx: &mut TestAppContext) {
    let log = Log::default();
    let cx = support::open(cx, menu(&log));
    support::tab(cx);
    support::press(cx, "enter");
    log.take();

    support::press(cx, "down");
    support::press(cx, "enter");
    assert_eq!(log.take(), ["paste"], "無効な項目を飛ばす");
    assert!(!log.open.get(), "選ぶと閉じる");

    support::draw(cx);
    support::press(cx, "enter");
    assert_eq!(
        log.take(),
        ["opener"],
        "閉じた後は開いたボタンにフォーカスが戻る"
    );
    support::press(cx, "up");
    support::press(cx, "enter");
    assert_eq!(log.take(), ["paste"], "先頭から上で末尾に回る");
}

#[gpui::test]
fn escape_and_tab_close_the_menu(cx: &mut TestAppContext) {
    let log = Log::default();
    let cx = support::open(cx, menu(&log));
    support::tab(cx);
    support::press(cx, "enter");
    support::press(cx, "escape");
    assert!(!log.open.get());
    assert_eq!(log.take(), ["opener"], "Esc では何も選ばない");

    support::press(cx, "enter");
    assert!(log.open.get(), "Esc の後は開いたボタンにフォーカスが戻る");
    support::press(cx, "tab");
    assert!(!log.open.get(), "Tab でも閉じる");
}
