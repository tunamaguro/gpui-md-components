//! タブの状態。
mod support;
use gpui::{Role, TestAppContext, prelude::*, px};
use gpui_headless::Tabs;
use std::{cell::Cell, rc::Rc};
#[test]
fn tabs_are_a_tab_list() {
    let tabs = Tabs::new("t", "表示", 1)
        .tab("写真", false)
        .tab("アルバム", false);
    assert_eq!(tabs.a11y_node().role, Role::TabList);
    let tab = tabs.tab_a11y_node(1).expect("tab");
    assert_eq!(tab.role, Role::Tab);
    assert_eq!(tab.selected, Some(true));
    assert_eq!(tab.position, Some((2, 2)));
    assert_eq!(tabs.tab_a11y_node(0).and_then(|t| t.selected), Some(false));
}

#[gpui::test]
fn arrows_select_the_next_tab(cx: &mut TestAppContext) {
    let selected = Rc::new(Cell::new(0));
    let cx = support::open(cx, {
        let selected = selected.clone();
        move |window, cx| {
            let on_change = selected.clone();
            Tabs::new("tabs", "表示", selected.get())
                .tab("写真", false)
                .tab("アルバム", true)
                .tab("共有", false)
                .on_change(move |index, _, _| on_change.set(index))
                .build(window, cx, |item, _, _| {
                    item.base.size(px(40.)).into_any_element()
                })
                .into_any_element()
        }
    });
    support::tab(cx);
    support::press(cx, "right");
    assert_eq!(selected.get(), 2, "無効なタブを飛ばす");
    support::press(cx, "right");
    assert_eq!(selected.get(), 0, "末尾から先頭に回る");
    support::press(cx, "end");
    assert_eq!(selected.get(), 2);
    support::press(cx, "home");
    assert_eq!(selected.get(), 0);
}
