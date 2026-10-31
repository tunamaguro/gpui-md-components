//! Scaffold の配置と、重ねた領域の操作。
#![cfg(debug_assertions)]
mod support;
use gpui::{AppContext, Modifiers, TestAppContext, div, point, prelude::*, px};
use md3_components::buttons::{Button, ExtendedFab};
use md3_components::containment::{FabPosition, Scaffold};
use md3_components::overlays::{SnackbarHost, SnackbarMessage};
use md3_components::theme::{Md3ThemeExt, SetMd3ThemeExt};
use std::{cell::Cell, rc::Rc};

#[gpui::test]
fn places_snackbar_and_fab(cx: &mut TestAppContext) {
    for scale in [1.0, 2.0] {
        for position in [FabPosition::Start, FabPosition::Center, FabPosition::End] {
            cx.update(|cx| {
                cx.set_reduce_motion(true);
                let theme = cx.md3().to_builder().text_scale(scale).build();
                cx.set_md3_theme(theme);
            });
            let host = cx.new(|_| SnackbarHost::new("閉じる"));
            host.update(cx, |host, cx| {
                host.show(SnackbarMessage::new("保存").action("戻す", |_, _| {}), cx);
            });
            let cx = support::open(cx, move |_, _| {
                div()
                    .w(px(400.))
                    .h(px(400.))
                    .debug_selector(|| "frame".into())
                    .child(
                        Scaffold::new("scaffold")
                            .fab_position(position)
                            .snackbar_host(&host)
                            .bottom_bar(div().h(px(40.)).debug_selector(|| "bottom".into()))
                            .floating_action_button(
                                div()
                                    .debug_selector(|| "fab".into())
                                    .child(ExtendedFab::new("fab", "作成").on_click(|_, _, _| {})),
                            )
                            .child(div().size_full()),
                    )
                    .into_any_element()
            });
            let frame = cx.debug_bounds("frame").expect("frame");
            let fab = cx.debug_bounds("fab").expect("fab");
            let bottom = cx.debug_bounds("bottom").expect("bottom");
            let snack = cx.debug_bounds("scaffold-snackbar").expect("snackbar");
            assert_eq!(bottom.top() - fab.bottom(), px(16.));
            assert_eq!(snack.bottom(), fab.top());
            match position {
                FabPosition::Start => assert_eq!(fab.left() - frame.left(), px(16.)),
                FabPosition::End => assert_eq!(frame.right() - fab.right(), px(16.)),
                FabPosition::Center => assert_eq!(
                    fab.left() + fab.size.width / 2.,
                    frame.left() + frame.size.width / 2.
                ),
            }
        }
    }
}

#[gpui::test]
fn empty_overlay_allows_click_and_does_not_change_content_height(cx: &mut TestAppContext) {
    let host = cx.new(|_| SnackbarHost::new("閉じる"));
    let clicked = Rc::new(Cell::new(false));
    let result = clicked.clone();
    let cx =
        support::open(cx, move |_, _| {
            let result = result.clone();
            div()
                .size(px(300.))
                .child(Scaffold::new("s").snackbar_host(&host).child(
                    div().size_full().debug_selector(|| "content".into()).child(
                        Button::new("button", "押す").on_click(move |_, _, _| result.set(true)),
                    ),
                ))
                .into_any_element()
        });
    let content = cx.debug_bounds("content").expect("content");
    assert_eq!(content.size.height, px(300.));
    cx.simulate_click(
        point(content.left() + px(30.), content.top() + px(20.)),
        Modifiers::none(),
    );
    assert!(clicked.get());
}

#[gpui::test]
fn snackbar_without_fab_sits_above_bottom_bar(cx: &mut TestAppContext) {
    cx.update(|cx| cx.set_reduce_motion(true));
    let host = cx.new(|_| SnackbarHost::new("閉じる"));
    host.update(cx, |host, cx| {
        host.show(SnackbarMessage::new("保存"), cx);
    });
    let cx = support::open(cx, move |_, _| {
        div()
            .size(px(300.))
            .child(
                Scaffold::new("s")
                    .snackbar_host(&host)
                    .bottom_bar(div().h(px(40.)).debug_selector(|| "bottom".into())),
            )
            .into_any_element()
    });
    assert_eq!(
        cx.debug_bounds("scaffold-snackbar")
            .expect("snackbar")
            .bottom(),
        cx.debug_bounds("bottom").expect("bottom").top()
    );
}

#[gpui::test]
fn snackbar_action_is_clickable_above_dialog_scrim(cx: &mut TestAppContext) {
    cx.update(|cx| cx.set_reduce_motion(true));
    let host = cx.new(|_| SnackbarHost::new("閉じる"));
    let clicked = Rc::new(Cell::new(false));
    let action = clicked.clone();
    host.update(cx, |host, cx| {
        host.show(
            SnackbarMessage::new("保存").action("戻す", move |_, _| action.set(true)),
            cx,
        );
    });
    let open = Rc::new(Cell::new(true));
    let dialog_open = open.clone();
    let cx = support::open(cx, move |_, _| {
        let dismiss = dialog_open.clone();
        div()
            .size_full()
            .child(Scaffold::new("s").snackbar_host(&host))
            .when(dialog_open.get(), |root| {
                root.child(
                    md3_components::overlays::Dialog::new("dialog")
                        .headline("確認")
                        .on_dismiss(move |_, _| dismiss.set(false)),
                )
            })
            .into_any_element()
    });
    let action = cx.debug_bounds("md3-snackbar-action").expect("action");
    cx.simulate_click(
        point(
            action.left() + action.size.width / 2.,
            action.top() + action.size.height / 2.,
        ),
        Modifiers::none(),
    );
    support::draw(cx);
    assert!(clicked.get());
    assert!(open.get());
}

#[gpui::test]
fn snackbar_action_comes_last_in_the_tab_order(cx: &mut TestAppContext) {
    use std::cell::RefCell;
    cx.update(|cx| cx.set_reduce_motion(true));
    let host = cx.new(|_| SnackbarHost::new("閉じる"));
    let log = Rc::new(RefCell::new(Vec::new()));
    let action = log.clone();
    host.update(cx, |host, cx| {
        host.show(
            SnackbarMessage::new("保存")
                .action("戻す", move |_, _| action.borrow_mut().push("snackbar")),
            cx,
        );
    });
    let recorded = log.clone();
    let cx = support::open(cx, move |_, _| {
        let content = recorded.clone();
        let fab = recorded.clone();
        let bottom = recorded.clone();
        div()
            .size(px(400.))
            .child(
                Scaffold::new("s")
                    .snackbar_host(&host)
                    .child(
                        Button::new("content", "本文")
                            .on_click(move |_, _, _| content.borrow_mut().push("content")),
                    )
                    .floating_action_button(
                        ExtendedFab::new("fab", "作成")
                            .on_click(move |_, _, _| fab.borrow_mut().push("fab")),
                    )
                    .bottom_bar(
                        Button::new("bottom", "下部")
                            .on_click(move |_, _, _| bottom.borrow_mut().push("bottom")),
                    ),
            )
            .into_any_element()
    });
    for _ in 0..4 {
        support::tab(cx);
        support::press(cx, "enter");
    }
    // Snackbar は遅延描画なので、ウィンドウの層に出すときと同じく最後に回る。
    assert_eq!(*log.borrow(), ["content", "fab", "bottom", "snackbar"]);
}
