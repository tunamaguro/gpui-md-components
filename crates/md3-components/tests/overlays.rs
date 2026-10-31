//! Dialog・Menu のフォーカス（閉じ込め、Esc、閉じた後の復帰）とキー操作。
//!
//! どの要素にフォーカスがあるかは、Enter で押されたボタンの記録で確かめる。

mod support;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gpui::{Focusable, TestAppContext, div, prelude::*};
use md3_components::IconName;
use md3_components::buttons::{Button, IconButton};
use md3_components::inputs::{TextField, TextFieldState};
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
                    Dialog::new("confirm")
                        .headline("確認")
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

fn validating_dialog<'a>(
    cx: &'a mut TestAppContext,
    log: &Log,
    move_focus: bool,
    closes: bool,
) -> (&'a mut gpui::VisualTestContext, gpui::FocusHandle) {
    let state = cx.update(TextFieldState::new);
    let focus = cx.update(|cx| state.focus_handle(cx));
    let field_focus = focus.clone();
    let log = log.clone();
    let cx = support::open(cx, move |_, _| {
        let action_log = log.clone();
        let field_focus = field_focus.clone();
        div()
            .child(opener(&log))
            .when(log.open.get(), |this| {
                this.child(
                    Dialog::new("validating")
                        .headline("入力")
                        .child(TextField::new("required", &state).label("必須"))
                        .action(DialogAction::validating("送信", move |window, cx| {
                            action_log.pressed.borrow_mut().push("send");
                            if move_focus {
                                window.focus(&field_focus, cx);
                            }
                            if closes {
                                action_log.open.set(false);
                            }
                            closes
                        })),
                )
            })
            .into_any_element()
    });
    (cx, focus)
}

#[gpui::test]
fn validation_failure_keeps_handler_focus_and_traps_tab(cx: &mut TestAppContext) {
    let log = Log::default();
    let (cx, focus) = validating_dialog(cx, &log, true, false);
    support::tab(cx);
    support::press(cx, "enter");
    assert_eq!(log.take(), ["opener"]);
    support::press(cx, "tab");
    support::press(cx, "enter");
    assert_eq!(log.take(), ["send"]);
    assert!(log.open.get());
    cx.update(|window, _| assert!(focus.is_focused(window), "入力欄にフォーカスが残る"));
    support::press(cx, "tab");
    support::press(cx, "enter");
    assert_eq!(log.take(), ["send"], "Tab はダイアログの操作に移る");
    assert!(log.open.get());
}

#[gpui::test]
fn validation_failure_keeps_action_focus(cx: &mut TestAppContext) {
    let log = Log::default();
    let (cx, focus) = validating_dialog(cx, &log, false, false);
    support::tab(cx);
    support::press(cx, "enter");
    assert_eq!(log.take(), ["opener"]);
    support::press(cx, "tab");
    support::press(cx, "enter");
    assert_eq!(log.take(), ["send"]);
    assert!(log.open.get());
    cx.update(|window, _| assert!(!focus.is_focused(window)));
    support::press(cx, "enter");
    assert_eq!(log.take(), ["send"], "操作ボタンにフォーカスが残る");
}

#[gpui::test]
fn validation_success_restores_opener_focus(cx: &mut TestAppContext) {
    let log = Log::default();
    let (cx, _) = validating_dialog(cx, &log, false, true);
    support::tab(cx);
    support::press(cx, "enter");
    assert_eq!(log.take(), ["opener"]);
    support::press(cx, "tab");
    support::press(cx, "enter");
    assert_eq!(log.take(), ["send"]);
    assert!(!log.open.get());
    support::press(cx, "enter");
    assert_eq!(log.take(), ["opener"]);
    assert!(log.open.get());
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
                    Menu::new("actions")
                        .aria_label("操作")
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

gpui::actions!(
    overlays_test,
    [
        /// アプリの Tab。
        AppTab,
        /// アプリの Shift-Tab。
        AppShiftTab,
        /// アプリの Esc。
        AppEscape
    ]
);

/// init より前のアプリの Tab はライブラリの割り当てに上書きされる。Esc はルートで受ける。
fn dialog_with_inner<'a>(
    cx: &'a mut TestAppContext,
    log: &Log,
    action: bool,
    inner: impl Fn() -> gpui::AnyElement + 'static,
) -> &'a mut gpui::VisualTestContext {
    cx.update(|cx| {
        cx.bind_keys([
            gpui::KeyBinding::new("tab", AppTab, None),
            gpui::KeyBinding::new("shift-tab", AppShiftTab, None),
            gpui::KeyBinding::new("escape", AppEscape, None),
        ]);
    });
    let log = log.clone();
    support::open(cx, move |window, cx| {
        let focus = window.use_keyed_state("root", cx, |_, cx| cx.focus_handle());
        let cancel = log.record("cancel");
        div()
            .track_focus(focus.read(cx))
            .on_action(|_: &AppTab, window, cx| window.focus_next(cx))
            .on_action(|_: &AppShiftTab, window, cx| window.focus_prev(cx))
            .on_action(|_: &AppEscape, _, _| {})
            .child(opener(&log))
            .when(log.open.get(), |this| {
                let close = log.close();
                let dialog = Dialog::new("form")
                    .headline("入力")
                    .child(inner())
                    .on_dismiss(close);
                this.child(if action {
                    dialog.action(DialogAction::new("キャンセル", move |window, cx| {
                        cancel(window, cx)
                    }))
                } else {
                    dialog
                })
            })
            .into_any_element()
    })
}

fn assert_keys_stay_in_the_dialog(cx: &mut gpui::VisualTestContext, log: &Log, escapes: usize) {
    support::tab(cx);
    support::press(cx, "enter");
    assert_eq!(log.take(), ["opener"]);
    support::draw(cx);

    // 最初の要素から Shift-Tab で末尾の操作へ回り、後ろの「開く」には行かない。
    support::press(cx, "shift-tab");
    support::press(cx, "enter");
    assert_eq!(log.take(), ["cancel"]);
    assert!(log.open.get());

    // 操作のあとはフォーカスが後ろへ戻るので、Tab で開き直さずに中の要素へ進めることを確かめる。
    support::press(cx, "tab");
    support::press(cx, "tab");
    support::press(cx, "enter");
    assert_eq!(log.take(), ["cancel"]);

    // 内側の要素からでも Esc で閉じる。
    support::press(cx, "tab");
    for _ in 0..escapes {
        assert!(log.open.get());
        support::press(cx, "escape");
    }
    assert!(!log.open.get());
}

#[gpui::test]
fn shift_tab_from_a_text_field_stays_in_the_dialog(cx: &mut TestAppContext) {
    let log = Log::default();
    let state = cx.update(TextFieldState::new);
    let cx = dialog_with_inner(cx, &log, true, move || {
        TextField::new("to", &state)
            .label("宛先")
            .into_any_element()
    });
    assert_keys_stay_in_the_dialog(cx, &log, 1);
}

#[gpui::test]
fn shift_tab_from_a_tooltip_button_stays_in_the_dialog(cx: &mut TestAppContext) {
    let log = Log::default();
    let cx = dialog_with_inner(cx, &log, true, || {
        IconButton::new("share", IconName::Share)
            .aria_label("共有")
            .tooltip("共有")
            .into_any_element()
    });
    // 1 回目の Esc はツールチップを閉じる。
    assert_keys_stay_in_the_dialog(cx, &log, 2);
}

#[gpui::test]
fn keys_from_an_app_context_stay_in_the_dialog(cx: &mut TestAppContext) {
    let log = Log::default();
    let inner = log.clone();
    let cx = dialog_with_inner(cx, &log, true, move || {
        let record = inner.record("inner");
        div()
            .key_context("AppEditor")
            .child(Button::new("inner", "中").on_click(move |_, window, cx| record(window, cx)))
            .into_any_element()
    });
    assert_keys_stay_in_the_dialog(cx, &log, 1);
}

#[gpui::test]
fn tab_from_the_last_text_field_stays_in_the_dialog(cx: &mut TestAppContext) {
    let log = Log::default();
    let state = cx.update(TextFieldState::new);
    let cx = dialog_with_inner(cx, &log, false, move || {
        TextField::new("to", &state)
            .label("宛先")
            .into_any_element()
    });
    support::tab(cx);
    support::press(cx, "enter");
    assert_eq!(log.take(), ["opener"]);
    support::draw(cx);

    support::press(cx, "tab");
    support::press(cx, "enter");
    assert_eq!(log.take(), Vec::<&str>::new(), "後ろのボタンに行かない");
    assert!(log.open.get());
}
