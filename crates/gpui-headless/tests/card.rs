//! カードの役割とキー操作。

mod support;

use gpui::{Role, TestAppContext, div, prelude::*};
use gpui_headless::{Card, CardBase};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[test]
fn clickable_cards_are_buttons() {
    let card = Card::new()
        .on_click("c", |_, _, _| {})
        .aria_label("旅行の計画");
    let node = card.a11y_node().expect("clickable");
    assert_eq!(node.role, Role::Button);
    assert_eq!(node.label.as_deref(), Some("旅行の計画"));
    assert_eq!(Card::new().a11y_node(), None);
    assert!(
        Card::new()
            .on_click("c", |_, _, _| {})
            .aria_label("旅行の計画")
            .disabled(true)
            .a11y_node()
            .expect("clickable")
            .disabled
    );
}

#[gpui::test]
fn clickable_card_responds_to_enter_and_space(cx: &mut TestAppContext) {
    let clicks = Rc::new(Cell::new(0));
    let cx = support::open(cx, {
        let clicks = clicks.clone();
        move |window, cx| {
            let clicks = clicks.clone();
            match Card::new()
                .on_click("c", move |_, _, _| clicks.set(clicks.get() + 1))
                .aria_label("旅行の計画")
                .build(div().size(gpui::px(40.)), window, cx)
            {
                CardBase::Plain(base) => base.into_any_element(),
                CardBase::Clickable(base, _) => base.into_any_element(),
            }
        }
    });
    support::tab(cx);
    support::press(cx, "enter");
    support::press(cx, "space");
    assert_eq!(clicks.get(), 2);
}

#[gpui::test]
fn tab_stops_on_clickable_cards_and_skips_disabled_cards(cx: &mut TestAppContext) {
    let handles = Rc::new(RefCell::new(Vec::new()));
    let stored = handles.clone();
    let cx = support::open(cx, move |window, cx| {
        *stored.borrow_mut() = vec![
            gpui_headless::focus_handle("first", window, cx),
            gpui_headless::focus_handle("last", window, cx),
        ];
        let cards = [("first", false), ("disabled", true), ("last", false)];
        let mut root = div().flex().flex_col();
        for (id, disabled) in cards {
            let card = Card::new()
                .on_click(id, |_, _, _| {})
                .disabled(disabled)
                .build(div().size(gpui::px(40.)), window, cx);
            root = root.child(match card {
                CardBase::Plain(base) => base.into_any_element(),
                CardBase::Clickable(base, _) => base.into_any_element(),
            });
        }
        root.into_any_element()
    });

    support::tab(cx);
    assert_eq!(
        cx.update(|window, cx| window.focused(cx)),
        Some(handles.borrow()[0].clone())
    );
    support::tab(cx);
    assert_eq!(
        cx.update(|window, cx| window.focused(cx)),
        Some(handles.borrow()[1].clone())
    );
}
