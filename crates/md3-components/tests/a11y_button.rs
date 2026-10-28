//! Button / IconButton / FAB のキー操作と a11y 情報。

mod support;

use std::{cell::Cell, rc::Rc};

use gpui::{Role, TestAppContext, Toggled, div, prelude::*};
use md3_components::IconName;
use md3_components::buttons::{Button, ExtendedFab, Fab, IconButton};

#[gpui::test]
fn enter_and_space_activate_a_button(cx: &mut TestAppContext) {
    let clicks = Rc::new(Cell::new(0));
    let cx = support::open(cx, {
        let clicks = clicks.clone();
        move |_, _| {
            let clicks = clicks.clone();
            Button::new("save", "保存")
                .on_click(move |_, _, _| clicks.set(clicks.get() + 1))
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
        move |_, _| {
            let (a, b) = (clicks.clone(), clicks.clone());
            div()
                .child(
                    Button::new("a", "A")
                        .disabled(true)
                        .on_click(move |_, _, _| a.set((a.get().0 + 1, a.get().1))),
                )
                .child(
                    Button::new("b", "B")
                        .on_click(move |_, _, _| b.set((b.get().0, b.get().1 + 1))),
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
        move |_, _| {
            let toggle = on.clone();
            IconButton::new("fav", IconName::Favorite, "お気に入り")
                .selected(on.get())
                .on_click(move |_, _, _| toggle.set(!toggle.get()))
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
        move |_, _| {
            let clicks = clicks.clone();
            Fab::new("add", IconName::Add, "追加")
                .on_click(move |_, _, _| clicks.set(clicks.get() + 1))
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

    let toggle = IconButton::new("i", IconName::Star, "星")
        .selected(false)
        .a11y_node();
    assert_eq!(toggle.role, Role::Button);
    assert_eq!(toggle.toggled, Some(Toggled::False));

    let fab = Fab::new("f", IconName::Edit, "作成").a11y_node();
    assert_eq!(fab.label.as_deref(), Some("作成"));
    let extended = ExtendedFab::new("e", "作成").a11y_node();
    assert_eq!(extended.role, Role::Button);
}
