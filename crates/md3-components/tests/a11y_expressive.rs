//! ToggleButton / ButtonGroup（M3 Expressive）のキー操作。

mod support;

use std::{cell::Cell, rc::Rc};

use gpui::TestAppContext;
use gpui::prelude::*;
use md3_components::expressive::{ButtonGroup, ToggleButton};
use md3_components::theme::{MotionScheme, SetMd3ThemeExt, Theme};

fn use_expressive(cx: &mut TestAppContext, reduce_motion: bool) {
    cx.update(|cx| {
        cx.set_md3_theme(
            Theme::default()
                .to_builder()
                .motion_scheme(MotionScheme::Expressive)
                .build(),
        );
        cx.set_reduce_motion(reduce_motion);
    });
}

/// 3 つの項目の選択の状態を持つ ButtonGroup を描く。`single` なら選択を 1 つに限る。
fn group_harness(
    cx: &mut TestAppContext,
    connected: bool,
    single: bool,
) -> (&mut gpui::VisualTestContext, Rc<Cell<[bool; 3]>>) {
    let checked = Rc::new(Cell::new([false; 3]));
    let cx = support::open(cx, {
        let checked = checked.clone();
        move |_, _| {
            let items = ["日", "週", "月"]
                .into_iter()
                .enumerate()
                .map(|(index, label)| {
                    let checked = checked.clone();
                    ToggleButton::new(("item", index), label)
                        .checked(checked.get()[index])
                        .disabled(index == 1 && !single)
                        .on_change(move |on, _, _| {
                            let mut next = if single { [false; 3] } else { checked.get() };
                            next[index] = on;
                            checked.set(next);
                        })
                });
            ButtonGroup::new("range", "期間")
                .connected(connected)
                .children(items)
                .into_any_element()
        }
    });
    (cx, checked)
}

#[gpui::test]
fn button_group_items_are_tab_stops_and_skip_disabled(cx: &mut TestAppContext) {
    use_expressive(cx, false);
    let (cx, checked) = group_harness(cx, false, false);
    support::tab(cx);
    support::press(cx, "space");
    support::tab(cx);
    support::press(cx, "space");
    // 2 つ目は無効なので、2 回目の Tab は 3 つ目に移る。
    assert_eq!(checked.get(), [true, false, true]);
}

#[gpui::test]
fn connected_group_can_be_used_for_a_single_choice(cx: &mut TestAppContext) {
    use_expressive(cx, false);
    let (cx, checked) = group_harness(cx, true, true);
    support::tab(cx);
    support::press(cx, "enter");
    assert_eq!(checked.get(), [true, false, false]);
    support::tab(cx);
    support::press(cx, "enter");
    assert_eq!(checked.get(), [false, true, false]);
}

#[gpui::test]
fn group_works_with_reduced_motion(cx: &mut TestAppContext) {
    use_expressive(cx, true);
    let (cx, checked) = group_harness(cx, true, false);
    support::tab(cx);
    support::press(cx, "space");
    assert_eq!(checked.get(), [true, false, false]);
}
