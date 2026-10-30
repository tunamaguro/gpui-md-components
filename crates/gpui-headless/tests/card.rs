//! カードの役割とキー操作。

mod support;

use gpui::{Role, TestAppContext, div, prelude::*};
use gpui_headless::{Card, CardBase};
use std::{cell::Cell, rc::Rc};

#[test]
fn clickable_cards_are_buttons() {
    let card = Card::new().on_click("c", "旅行の計画", |_, _, _| {});
    let node = card.a11y_node().expect("clickable");
    assert_eq!(node.role, Role::Button);
    assert_eq!(node.label.as_deref(), Some("旅行の計画"));
    assert_eq!(Card::new().a11y_node(), None);
    assert!(
        Card::new()
            .on_click("c", "旅行の計画", |_, _, _| {})
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
                .on_click("c", "旅行の計画", move |_, _, _| {
                    clicks.set(clicks.get() + 1)
                })
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
