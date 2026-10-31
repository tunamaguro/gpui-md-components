//! Button / IconButton / FAB のキー操作と a11y 情報。

mod support;

use std::{cell::Cell, rc::Rc};

use gpui::{Role, TestAppContext, Toggled, div, prelude::*};
use gpui_headless::Button;

#[gpui::test]
fn enter_and_space_activate_a_button(cx: &mut TestAppContext) {
    let clicks = Rc::new(Cell::new(0));
    let cx = support::open(cx, {
        let clicks = clicks.clone();
        move |window, cx| {
            let clicks = clicks.clone();
            Button::new("save", "保存")
                .on_click(move |_, _, _| clicks.set(clicks.get() + 1))
                .build(window, cx)
                .0
                .size(gpui::px(40.))
                .into_any_element()
        }
    });
    support::tab(cx);
    support::press(cx, "enter");
    support::press(cx, "space");
    assert_eq!(clicks.get(), 2);
}

#[gpui::test]
fn disabled_button_is_skipped_by_tab(cx: &mut TestAppContext) {
    let clicks = Rc::new(Cell::new((0, 0)));
    let cx = support::open(cx, {
        let clicks = clicks.clone();
        move |window, cx| {
            let (a, b) = (clicks.clone(), clicks.clone());
            div()
                .child(
                    Button::new("a", "A")
                        .disabled(true)
                        .on_click(move |_, _, _| a.set((a.get().0 + 1, a.get().1)))
                        .build(window, cx)
                        .0
                        .size(gpui::px(40.)),
                )
                .child(
                    Button::new("b", "B")
                        .on_click(move |_, _, _| b.set((b.get().0, b.get().1 + 1)))
                        .build(window, cx)
                        .0
                        .size(gpui::px(40.)),
                )
                .into_any_element()
        }
    });
    support::tab(cx);
    support::press(cx, "enter");
    assert_eq!(clicks.get(), (0, 1));
}

#[gpui::test]
fn toggle_icon_button_flips_with_space(cx: &mut TestAppContext) {
    let on = Rc::new(Cell::new(false));
    let cx = support::open(cx, {
        let on = on.clone();
        move |window, cx| {
            let toggle = on.clone();
            Button::new("fav", "お気に入り")
                .toggled(Some(on.get()))
                .on_click(move |_, _, _| toggle.set(!toggle.get()))
                .build(window, cx)
                .0
                .size(gpui::px(40.))
                .into_any_element()
        }
    });
    support::tab(cx);
    support::press(cx, "space");
    assert!(on.get());
    support::press(cx, "space");
    assert!(!on.get());
}

#[gpui::test]
fn fab_activates_with_enter(cx: &mut TestAppContext) {
    let clicks = Rc::new(Cell::new(0));
    let cx = support::open(cx, {
        let clicks = clicks.clone();
        move |window, cx| {
            let clicks = clicks.clone();
            Button::new("add", "追加")
                .on_click(move |_, _, _| clicks.set(clicks.get() + 1))
                .build(window, cx)
                .0
                .size(gpui::px(40.))
                .into_any_element()
        }
    });
    support::tab(cx);
    support::press(cx, "enter");
    assert_eq!(clicks.get(), 1);
}

#[test]
fn buttons_expose_role_name_and_state() {
    let button = Button::new("b", "保存").a11y_node();
    assert_eq!(
        (button.role, button.label.as_deref()),
        (Role::Button, Some("保存"))
    );

    let toggle = Button::new("i", "星").toggled(Some(false)).a11y_node();
    assert_eq!(toggle.role, Role::Button);
    assert_eq!(toggle.toggled, Some(Toggled::False));

    let fab = Button::new("f", "作成").a11y_node();
    assert_eq!(fab.label.as_deref(), Some("作成"));
    let extended = Button::new("e", "作成").a11y_node();
    assert_eq!(extended.role, Role::Button);
}

#[test]
fn moved_button_states_are_preserved() {
    let button = Button::new("b", "保存").disabled(true).a11y_node();
    assert_eq!(button.role, Role::Button);
    assert_eq!(button.label.as_deref(), Some("保存"));
    assert!(button.disabled);
    let toggle = Button::new("b", "太字").toggled(Some(true)).a11y_node();
    assert_eq!(toggle.role, Role::Button);
    assert_eq!(toggle.label.as_deref(), Some("太字"));
    assert_eq!(toggle.toggled, Some(Toggled::True));
    assert_eq!(Button::new("i", "お気に入り").a11y_node().toggled, None);
}

#[gpui::test]
fn toggle_button_flips_with_space_and_enter(cx: &mut TestAppContext) {
    let on = Rc::new(Cell::new(false));
    let cx = support::open(cx, {
        let on = on.clone();
        move |window, cx| {
            let toggle = on.clone();
            Button::new("bold", "太字")
                .toggled(Some(on.get()))
                .on_click(move |_, _, _| toggle.set(!toggle.get()))
                .build(window, cx)
                .0
                .size(gpui::px(40.))
                .into_any_element()
        }
    });
    support::tab(cx);
    support::press(cx, "space");
    assert!(on.get());
    support::press(cx, "enter");
    assert!(!on.get());
}

#[test]
fn button_name_override() {
    assert_eq!(
        Button::new("b", "保存")
            .aria_label("保存する")
            .a11y_node()
            .label
            .as_deref(),
        Some("保存する")
    );
    assert_eq!(
        Button::new("b", "保存").aria_label("").a11y_node().label,
        None
    );
    assert_eq!(Button::new("b", "").a11y_node().label, None);
}

#[test]
fn toggle_state_reflects_the_latest_setting() {
    assert_eq!(Button::new("b", "").toggled_state(), None);
    assert_eq!(
        Button::new("b", "")
            .toggled(Some(true))
            .toggled(Some(false))
            .toggled_state(),
        Some(false)
    );
}

#[gpui::test]
fn toggle_handler_and_preprocessing_use_final_configuration(cx: &mut TestAppContext) {
    let events = Rc::new(std::cell::RefCell::new(Vec::new()));
    let cx = support::open(cx, {
        let events = events.clone();
        move |window, cx| {
            let before = events.clone();
            let changed = events.clone();
            Button::new("toggle", "切替")
                .before_click(move |_, _, _| before.borrow_mut().push("before"))
                .on_toggle(move |next, _, _| {
                    changed.borrow_mut().push(if next { "on" } else { "off" });
                })
                .toggled(Some(true))
                .disabled(true)
                .disabled(false)
                .build(window, cx)
                .0
                .size(gpui::px(40.))
                .into_any_element()
        }
    });
    support::tab(cx);
    support::press(cx, "space");
    assert_eq!(*events.borrow(), ["before", "off"]);
}
