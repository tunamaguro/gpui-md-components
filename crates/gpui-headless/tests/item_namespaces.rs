//! グループと項目の描画で使う ID の名前空間。
mod support;

use gpui::{App, FocusHandle, TestAppContext, Window, div, prelude::*, px};
use gpui_headless::{Button, ButtonGroup, Navigation, RadioGroup, SegmentedButton, Tabs};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Copy)]
enum GroupKind {
    Buttons,
    Navigation,
    Radio,
    Segments,
    Tabs,
}

fn check_item_namespaces(cx: &mut TestAppContext, kind: GroupKind) {
    let handles = Rc::new(RefCell::new(Vec::<FocusHandle>::new()));
    let cx = support::open(cx, {
        let handles = handles.clone();
        move |window, cx| {
            handles.borrow_mut().clear();
            let mut root = div().flex();
            for id in ["first", "second"] {
                let record = |window: &mut Window, cx: &mut App| {
                    handles.borrow_mut().push(gpui_headless::focus_handle(
                        "reused-child",
                        window,
                        cx,
                    ));
                };
                let group = match kind {
                    GroupKind::Buttons => ButtonGroup::new(id)
                        .aria_label(id)
                        .item(Button::new("a", "a"))
                        .item(Button::new("b", "b"))
                        .build(window, cx, |item, window, cx| {
                            record(window, cx);
                            item.base.size(px(40.)).into_any_element()
                        })
                        .into_any_element(),
                    GroupKind::Navigation => {
                        let nav = Navigation::new(id)
                            .aria_label(id)
                            .item("a", false)
                            .item("b", false)
                            .build(window, cx, |item, window, cx| {
                                record(window, cx);
                                item.base.size(px(40.)).into_any_element()
                            });
                        nav.base.children(nav.items).into_any_element()
                    }
                    GroupKind::Radio => RadioGroup::new(id)
                        .aria_label(id)
                        .option("a")
                        .option("b")
                        .build(window, cx, |item, window, cx| {
                            record(window, cx);
                            item.base.size(px(40.)).into_any_element()
                        })
                        .into_any_element(),
                    GroupKind::Segments => SegmentedButton::new(id)
                        .aria_label(id)
                        .segment("a", false)
                        .segment("b", false)
                        .build(window, cx, |item, window, cx| {
                            record(window, cx);
                            item.base.size(px(40.)).into_any_element()
                        })
                        .into_any_element(),
                    GroupKind::Tabs => Tabs::new(id, 0)
                        .aria_label(id)
                        .tab("a", false)
                        .tab("b", false)
                        .build(window, cx, |item, window, cx| {
                            record(window, cx);
                            item.base.size(px(40.)).into_any_element()
                        })
                        .into_any_element(),
                };
                root = root.child(group);
            }
            root.into_any_element()
        }
    });
    let first = handles.borrow().clone();
    assert_eq!(first.len(), 4);
    for (index, handle) in first.iter().enumerate() {
        assert!(first[..index].iter().all(|other| other != handle));
    }
    support::draw(cx);
    assert_eq!(*handles.borrow(), first, "状態は次の描画でも維持される");
}

#[gpui::test]
fn button_group_item_namespaces(cx: &mut TestAppContext) {
    check_item_namespaces(cx, GroupKind::Buttons);
}

#[gpui::test]
fn navigation_item_namespaces(cx: &mut TestAppContext) {
    check_item_namespaces(cx, GroupKind::Navigation);
}

#[gpui::test]
fn radio_group_item_namespaces(cx: &mut TestAppContext) {
    check_item_namespaces(cx, GroupKind::Radio);
}

#[gpui::test]
fn segmented_button_item_namespaces(cx: &mut TestAppContext) {
    check_item_namespaces(cx, GroupKind::Segments);
}

#[gpui::test]
fn tab_item_namespaces(cx: &mut TestAppContext) {
    check_item_namespaces(cx, GroupKind::Tabs);
}
