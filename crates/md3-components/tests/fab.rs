//! FAB は内容の上に浮くので、下の要素にクリックを渡さない。

mod support;

use gpui::{Modifiers, TestAppContext, div, point, prelude::*, px};
use md3_components::IconName;
use md3_components::buttons::{ExtendedFab, Fab};
use std::{cell::Cell, rc::Rc};

fn click_over_content(
    cx: &mut TestAppContext,
    fab: impl Fn(Rc<Cell<u32>>) -> gpui::AnyElement + 'static,
) -> (u32, u32) {
    let behind = Rc::new(Cell::new(0));
    let pressed = Rc::new(Cell::new(0));
    let (count, fab_count) = (behind.clone(), pressed.clone());
    let cx = support::open(cx, move |_, _| {
        let count = count.clone();
        div()
            .id("content")
            .relative()
            .size(px(200.0))
            .on_click(move |_, _, _| count.set(count.get() + 1))
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .child(fab(fab_count.clone())),
            )
            .into_any_element()
    });
    cx.simulate_click(point(px(44.0), px(44.0)), Modifiers::none());
    support::draw(cx);
    (pressed.get(), behind.get())
}

#[gpui::test]
fn fab_does_not_click_what_is_behind_it(cx: &mut TestAppContext) {
    let clicks = click_over_content(cx, |pressed| {
        Fab::new("fab", IconName::Add)
            .aria_label("追加")
            .on_click(move |_, _, _| pressed.set(pressed.get() + 1))
            .into_any_element()
    });
    assert_eq!(clicks, (1, 0));
}

#[gpui::test]
fn extended_fab_does_not_click_what_is_behind_it(cx: &mut TestAppContext) {
    let clicks = click_over_content(cx, |pressed| {
        ExtendedFab::new("fab", "作成")
            .on_click(move |_, _, _| pressed.set(pressed.get() + 1))
            .into_any_element()
    });
    assert_eq!(clicks, (1, 0));
}
