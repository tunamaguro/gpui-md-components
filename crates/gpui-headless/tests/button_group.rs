//! ボタングループの状態。
use gpui::Role;
use gpui_headless::ButtonGroup;
#[test]
fn button_group_is_a_labelled_group() {
    let node = ButtonGroup::new("g", "配置").a11y_node();
    assert_eq!(node.role, Role::Group);
    assert_eq!(node.label.as_deref(), Some("配置"));
}

mod support;
use gpui::{TestAppContext, div, prelude::*, px};
use gpui_headless::Button;
use std::{cell::Cell, rc::Rc};

#[gpui::test]
fn items_are_tab_stops_and_skip_disabled(cx: &mut TestAppContext) {
    let checked = Rc::new(Cell::new([false; 3]));
    let cx = support::open(cx, {
        let checked = checked.clone();
        move |window, cx| {
            let mut group = ButtonGroup::new("range", "期間");
            for (index, label) in ["日", "週", "月"].into_iter().enumerate() {
                let checked = checked.clone();
                let selected = checked.get()[index];
                group = group.item(
                    Button::new(("item", index), label)
                        .toggled(Some(selected))
                        .disabled(index == 1)
                        .on_click(move |_, _, _| {
                            let mut next = checked.get();
                            next[index] = !next[index];
                            checked.set(next);
                        }),
                );
            }
            group
                .build(window, cx, |item, _, _| {
                    item.base.size(px(40.0)).into_any_element()
                })
                .flex()
                .into_any_element()
        }
    });
    support::tab(cx);
    support::press(cx, "space");
    support::tab(cx);
    support::press(cx, "space");
    assert_eq!(checked.get(), [true, false, true]);
}

#[gpui::test]
fn independent_buttons_can_implement_single_selection(cx: &mut TestAppContext) {
    let checked = Rc::new(Cell::new([false; 3]));
    let cx = support::open(cx, {
        let checked = checked.clone();
        move |window, cx| {
            let mut group = ButtonGroup::new("range", "期間");
            for (index, label) in ["日", "週", "月"].into_iter().enumerate() {
                let checked = checked.clone();
                group = group.item(
                    Button::new(("item", index), label)
                        .toggled(Some(checked.get()[index]))
                        .on_click(move |_, _, _| {
                            let mut next = [false; 3];
                            next[index] = true;
                            checked.set(next);
                        }),
                );
            }
            group
                .build(window, cx, |item, _, _| {
                    item.base.size(px(40.0)).into_any_element()
                })
                .flex()
                .into_any_element()
        }
    });
    support::tab(cx);
    support::press(cx, "enter");
    assert_eq!(checked.get(), [true, false, false]);
    support::tab(cx);
    support::press(cx, "enter");
    assert_eq!(checked.get(), [false, true, false]);
}

#[gpui::test]
fn groups_with_reused_child_ids_have_independent_focus_and_callbacks(cx: &mut TestAppContext) {
    let clicked = Rc::new(Cell::new([false; 2]));
    let cx = support::open(cx, {
        let clicked = clicked.clone();
        move |window, cx| {
            let mut root = div().flex();
            for (group_index, group_id) in ["first-group", "second-group"].into_iter().enumerate() {
                let mut group = ButtonGroup::new(group_id, group_id);
                for child_id in ["first-button", "second-button"] {
                    let clicked = clicked.clone();
                    group = group.item(Button::new(child_id, child_id).on_click(move |_, _, _| {
                        let mut next = clicked.get();
                        next[group_index] = true;
                        clicked.set(next);
                    }));
                }
                root = root.child(group.build(window, cx, |item, _, _| {
                    item.base.size(px(40.0)).into_any_element()
                }));
            }
            root.into_any_element()
        }
    });

    support::tab(cx);
    let mut stops = vec![cx.update(|window, cx| window.focused(cx).expect("フォーカス"))];
    support::press(cx, "enter");
    let first_click = clicked.get();
    for _ in 0..3 {
        support::tab(cx);
        stops.push(cx.update(|window, cx| window.focused(cx).expect("フォーカス")));
    }
    let mut distinct = Vec::new();
    for stop in stops {
        if !distinct.contains(&stop) {
            distinct.push(stop);
        }
    }
    assert_eq!(
        (first_click, distinct.len()),
        ([true, false], 4),
        "最初のグループだけが呼ばれ、Tab は 4 つの異なる要素で止まる"
    );
}
