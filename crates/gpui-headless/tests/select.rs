//! Select の役割と開閉。
mod support;
use gpui::{Role, TestAppContext, prelude::*};
use gpui_headless::Select;
use std::{cell::Cell, rc::Rc};

#[test]
fn select_is_a_combobox() {
    let select = Select::new("s", "色");
    assert_eq!(select.a11y_node().role, Role::ComboBox);
    assert!(!select.a11y_node().invalid);
    assert!(select.error(true).a11y_node().invalid);
}

#[gpui::test]
fn down_opens_and_the_dismiss_handler_closes(cx: &mut TestAppContext) {
    let open = Rc::new(Cell::new(false));
    let dismiss = Rc::new(std::cell::RefCell::new(None));
    let cx = support::open(cx, {
        let (open, dismiss) = (open.clone(), dismiss.clone());
        move |window, cx| {
            let (base, state) = Select::new("s", "色").build(window, cx);
            open.set(state.open);
            *dismiss.borrow_mut() = Some(state.dismiss_handler());
            base.into_any_element()
        }
    });
    support::tab(cx);
    support::press(cx, "down");
    support::draw(cx);
    assert!(open.get(), "↓ で開く");
    let handler = dismiss.borrow().clone().expect("handler");
    cx.update(|window, cx| handler(window, cx));
    support::draw(cx);
    assert!(!open.get(), "閉じる処理で閉じる");
}

#[gpui::test]
fn disabled_select_does_not_open(cx: &mut TestAppContext) {
    let open = Rc::new(Cell::new(false));
    let cx = support::open(cx, {
        let open = open.clone();
        move |window, cx| {
            let (base, state) = Select::new("s", "色").disabled(true).build(window, cx);
            open.set(state.open);
            base.into_any_element()
        }
    });
    support::tab(cx);
    support::press(cx, "down");
    support::draw(cx);
    assert!(!open.get());
}
