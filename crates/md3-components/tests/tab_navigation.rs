//! アプリが Tab を割り当てない場合のフォーカス移動。

#![cfg(debug_assertions)]

mod support;

use gpui::{TestAppContext, div, prelude::*};
use md3_components::buttons::Button;
use md3_components::overlays::{Dialog, DialogAction};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[gpui::test]
fn tab_navigates_without_app_bindings(cx: &mut TestAppContext) {
    let pressed = Rc::new(RefCell::new(Vec::new()));
    let cx = support::open(cx, {
        let pressed = pressed.clone();
        move |_, _| {
            let first = pressed.clone();
            let second = pressed.clone();
            div()
                .child(
                    Button::new("first", "First")
                        .on_click(move |_, _, _| first.borrow_mut().push("first")),
                )
                .child(
                    Button::new("second", "Second")
                        .on_click(move |_, _, _| second.borrow_mut().push("second")),
                )
                .into_any_element()
        }
    });
    assert!(cx.update(|window, cx| window.focused(cx).is_none()));
    support::press(cx, "tab");
    support::press(cx, "enter");
    assert_eq!(*pressed.borrow(), ["first"], "Tab で最初のボタンに移る");
    support::press(cx, "tab");
    support::press(cx, "enter");
    assert_eq!(*pressed.borrow(), ["first", "second"]);
    support::press(cx, "shift-tab");
    support::press(cx, "enter");
    assert_eq!(*pressed.borrow(), ["first", "second", "first"]);
}

#[gpui::test]
fn tab_recovers_after_the_focused_button_disappears(cx: &mut TestAppContext) {
    let shown = Rc::new(Cell::new(true));
    let pressed = Rc::new(Cell::new(false));
    let cx = support::open(cx, {
        let shown = shown.clone();
        let pressed = pressed.clone();
        move |_, _| {
            let remove = shown.clone();
            let pressed = pressed.clone();
            div()
                .when(shown.get(), |this| {
                    this.child(div().debug_selector(|| "removable".into()).child(
                        Button::new("first", "Remove").on_click(move |_, _, cx| {
                            remove.set(false);
                            cx.refresh_windows();
                        }),
                    ))
                })
                .child(
                    Button::new("second", "Remaining").on_click(move |_, _, _| pressed.set(true)),
                )
                .into_any_element()
        }
    });
    support::tab(cx);
    assert!(cx.update(|window, cx| window.focused(cx).is_some()));
    support::press(cx, "enter");
    assert!(!shown.get());
    assert!(
        cx.debug_bounds("removable").is_none(),
        "ボタンが描画ツリーから消えた"
    );
    assert!(
        cx.update(|window, cx| window.focused(cx).is_none()),
        "有効なフォーカスが残っていない"
    );
    support::press(cx, "tab");
    support::press(cx, "enter");
    assert!(pressed.get(), "Tab で残ったボタンに移る");
}

#[gpui::test]
fn dialog_traps_tab_without_app_bindings(cx: &mut TestAppContext) {
    let pressed = Rc::new(RefCell::new(Vec::new()));
    let cx = support::open(cx, {
        let pressed = pressed.clone();
        move |_, _| {
            let behind = pressed.clone();
            let inner = pressed.clone();
            let action = pressed.clone();
            div()
                .child(
                    Button::new("behind", "Behind")
                        .on_click(move |_, _, _| behind.borrow_mut().push("behind")),
                )
                .child(
                    Dialog::new("dialog")
                        .headline("Dialog")
                        .child(
                            Button::new("inner", "Inner")
                                .on_click(move |_, _, _| inner.borrow_mut().push("inner")),
                        )
                        .action(DialogAction::new("Action", move |_, _| {
                            action.borrow_mut().push("action")
                        })),
                )
                .into_any_element()
        }
    });
    support::press(cx, "enter");
    support::press(cx, "tab");
    support::press(cx, "enter");
    support::press(cx, "tab");
    support::press(cx, "enter");
    support::press(cx, "shift-tab");
    support::press(cx, "enter");
    assert_eq!(*pressed.borrow(), ["inner", "action", "inner", "action"]);
}
