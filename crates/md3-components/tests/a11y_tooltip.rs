//! キーボードフォーカスで出すツールチップ（APG の Tooltip パターン）。
//!
//! 表示は `debug_selector` で確かめるため、デバッグビルドでだけ走る（gpui はリリースでは記録しない）。
#![cfg(debug_assertions)]

mod support;

use gpui::{Modifiers, TestAppContext, VisualTestContext, div, prelude::*};
use md3_components::IconName;
use md3_components::buttons::{Button, IconButton};
use md3_components::overlays::Dialog;
use std::{cell::Cell, rc::Rc};

const TOOLTIP: &str = "md3-focus-tooltip-share";

fn buttons(cx: &mut TestAppContext) -> &mut VisualTestContext {
    support::open(cx, |_, _| {
        div()
            .child(
                IconButton::new("share", IconName::Share)
                    .aria_label("共有")
                    .tooltip("リンクをコピー")
                    .on_click(|_, _, _| {}),
            )
            .child(Button::new("next", "次へ").on_click(|_, _, _| {}))
            .into_any_element()
    })
}

/// キーボードで次へ移す。gpui は実際のキー入力がないとキーボード操作と判定しない。
fn key_tab(cx: &mut VisualTestContext) {
    support::press(cx, "tab");
    support::tab(cx);
}

fn shown(cx: &mut VisualTestContext) -> bool {
    cx.debug_bounds(TOOLTIP).is_some()
}

#[gpui::test]
fn keyboard_focus_shows_the_tooltip(cx: &mut TestAppContext) {
    let cx = buttons(cx);
    assert!(!shown(cx));
    key_tab(cx);
    assert!(shown(cx), "フォーカスで出る");
    key_tab(cx);
    assert!(!shown(cx), "フォーカスが離れると消える");
}

#[gpui::test]
fn escape_hides_the_tooltip_until_focus_returns(cx: &mut TestAppContext) {
    let cx = buttons(cx);
    key_tab(cx);
    assert!(shown(cx), "Esc の前は出ている");
    support::press(cx, "escape");
    support::draw(cx);
    assert!(!shown(cx), "Esc で閉じる");
    support::draw(cx);
    assert!(!shown(cx), "フォーカスがある間は閉じたまま");
    key_tab(cx);
    cx.update(|window, cx| window.focus_prev(cx));
    support::draw(cx);
    assert!(shown(cx), "フォーカスが戻るとまた出る");
}

#[gpui::test]
fn pointer_focus_does_not_show_the_tooltip(cx: &mut TestAppContext) {
    let cx = buttons(cx);
    let button = cx
        .debug_bounds("md3-icon-button-share")
        .expect("ボタンの位置");
    cx.simulate_click(button.center(), Modifiers::none());
    support::draw(cx);
    assert!(!shown(cx));
}

#[gpui::test]
fn escape_closes_the_tooltip_before_the_dialog(cx: &mut TestAppContext) {
    let dismissed = Rc::new(Cell::new(0));
    let cx = support::open(cx, {
        let dismissed = dismissed.clone();
        move |_, _| {
            let dismissed = dismissed.clone();
            Dialog::new("dialog")
                .headline("確認")
                .on_dismiss(move |_, _| dismissed.set(dismissed.get() + 1))
                .child(
                    IconButton::new("share", IconName::Share)
                        .aria_label("共有")
                        .tooltip("リンクをコピー")
                        .on_click(|_, _, _| {}),
                )
                .into_any_element()
        }
    });
    key_tab(cx);
    if !shown(cx) {
        key_tab(cx);
    }
    assert!(shown(cx), "ダイアログの中でも出る");
    support::press(cx, "escape");
    support::draw(cx);
    assert!(!shown(cx));
    assert_eq!(dismissed.get(), 0, "最初の Esc はツールチップだけを閉じる");
    support::press(cx, "escape");
    assert_eq!(dismissed.get(), 1, "次の Esc でダイアログを閉じる");
}
