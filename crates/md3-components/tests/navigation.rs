//! Tabs・List・NavigationBar のキー操作（矢印キーでの移動、Tab で止まる場所）。
//!
//! どの項目にフォーカスがあるかは、Enter で選ばれた項目の記録で確かめる。

use md3_components::inputs::TextFieldState;
mod support;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gpui::{SharedString, TestAppContext, UniformListScrollHandle, div, prelude::*, px};
use md3_components::IconName;
use md3_components::containment::{LazyList, List, ListItem};
use md3_components::navigation::NavigationItem;

/// 選ばれた項目の記録。
fn recorder() -> (
    Rc<RefCell<Vec<usize>>>,
    impl Fn(usize) -> ListItem + Clone + 'static,
) {
    let log = Rc::new(RefCell::new(Vec::new()));
    let item = {
        let log = log.clone();
        move |index: usize| {
            let log = log.clone();
            ListItem::new(format!("項目 {index}"))
                .disabled(index == 1)
                .on_select(move |_, _| log.borrow_mut().push(index))
        }
    };
    (log, item)
}

#[gpui::test]
fn list_arrows_stop_at_the_ends(cx: &mut TestAppContext) {
    let (log, item) = recorder();
    let cx = support::open(cx, move |_, _| {
        (0..4)
            .fold(
                List::new("list").aria_label("受信トレイ"),
                |list, index| list.item(SharedString::from(format!("k{index}")), item(index)),
            )
            .into_any_element()
    });
    support::tab(cx);
    support::press(cx, "enter");
    support::press(cx, "down");
    support::press(cx, "enter");
    support::press(cx, "down");
    support::press(cx, "down");
    support::press(cx, "enter");
    support::press(cx, "up");
    support::press(cx, "up");
    support::press(cx, "enter");
    support::press(cx, "up");
    support::press(cx, "enter");
    assert_eq!(
        *log.borrow(),
        [0, 2, 3, 0, 0],
        "無効な項目を飛ばし、端では止まる"
    );
}

#[gpui::test]
fn lazy_list_scrolls_to_items_that_are_not_drawn(cx: &mut TestAppContext) {
    let (log, item) = recorder();
    let scroll = UniformListScrollHandle::new();
    let cx = support::open(cx, move |_, _| {
        let item = item.clone();
        div()
            .h(px(150.0))
            .w(px(300.0))
            .child(
                LazyList::new("lazy", 1000, &scroll, move |index, _| {
                    (SharedString::from(format!("k{index}")), item(index))
                })
                .aria_label("連絡先")
                .size_full(),
            )
            .into_any_element()
    });
    support::tab(cx);
    support::press(cx, "enter");
    support::press(cx, "end");
    support::press(cx, "enter");
    support::press(cx, "up");
    support::press(cx, "enter");
    support::press(cx, "home");
    support::press(cx, "down");
    support::press(cx, "enter");
    assert_eq!(*log.borrow(), [0, 999, 998, 2]);
}

#[gpui::test]
fn modal_drawer_takes_focus_and_escape_restores_it(cx: &mut TestAppContext) {
    use md3_components::buttons::Button;
    use md3_components::navigation::{NavigationDrawer, NavigationDrawerKind};

    let open = Rc::new(Cell::new(false));
    let selected = Rc::new(Cell::new(0));
    let cx = support::open(cx, {
        let open = open.clone();
        let selected = selected.clone();
        move |_, _| {
            let toggle = open.clone();
            let close = open.clone();
            let on_change = selected.clone();
            div()
                .child(Button::new("opener", "メニュー").on_click(move |_, _, _| toggle.set(true)))
                .when(open.get(), |this| {
                    this.child(
                        NavigationDrawer::new("drawer")
                            .aria_label("メイン")
                            .kind(NavigationDrawerKind::Modal)
                            .item(NavigationItem::new(IconName::Inbox, "受信トレイ"))
                            .item(NavigationItem::new(IconName::Send, "送信済み"))
                            .selected(Some(0))
                            .on_change(move |index, _, _| on_change.set(index))
                            .on_dismiss(move |_, _| close.set(false)),
                    )
                })
                .into_any_element()
        }
    });
    support::tab(cx);
    support::press(cx, "enter");
    assert!(open.get());

    // 開くと選択中の項目にフォーカスし、↓ と Enter で選べる。
    support::press(cx, "down");
    support::press(cx, "enter");
    assert_eq!(selected.get(), 1);

    // Esc で閉じ、開いたボタンにフォーカスが戻る。
    support::press(cx, "escape");
    assert!(!open.get());
    support::press(cx, "enter");
    assert!(open.get(), "フォーカスは開いたボタンに戻る");
}

#[gpui::test]
fn nested_lazy_list_consumes_wheel_until_its_edge(cx: &mut TestAppContext) {
    let outer = gpui::ScrollHandle::new();
    let inner = UniformListScrollHandle::new();
    let cx = support::open(cx, {
        let outer = outer.clone();
        let inner = inner.clone();
        move |_, _| {
            div()
                .id("page")
                .w(px(300.0))
                .h(px(300.0))
                .overflow_y_scroll()
                .track_scroll(&outer)
                .child(
                    LazyList::new("nested-list", 100, &inner, |index, _| {
                        (
                            format!("{index}").into(),
                            ListItem::new(format!("項目 {index}")),
                        )
                    })
                    .aria_label("項目")
                    .h(px(150.0))
                    .w_full(),
                )
                .child(div().h(px(1000.0)))
                .into_any_element()
        }
    });
    let wheel = |cx: &mut gpui::VisualTestContext, delta, phase| {
        cx.update(|window, cx| {
            window.dispatch_event(
                gpui::PlatformInput::Scroll(gpui::ScrollEvent {
                    // バー上でもリスト本体と同じホイール伝播になる。
                    position: {
                        let bounds = inner.0.borrow().base_handle.bounds();
                        gpui::point(bounds.right() - px(2.0), bounds.top() + px(30.0))
                    },
                    delta: gpui::ScrollDelta::Pixels(gpui::point(px(0.0), px(delta))),
                    touch_phase: phase,
                    ..Default::default()
                }),
                cx,
            );
        });
        support::draw(cx);
    };
    wheel(cx, -30.0, gpui::TouchPhase::Started);
    assert_eq!(outer.offset().y, px(0.0));
    assert_eq!(inner.0.borrow().base_handle.offset().y, px(-30.0));
    wheel(cx, -10000.0, gpui::TouchPhase::Moved);
    assert_eq!(outer.offset().y, px(0.0));
    wheel(cx, -30.0, gpui::TouchPhase::Moved);
    assert_eq!(
        outer.offset().y,
        px(0.0),
        "同じジェスチャーは端でも消費する"
    );
    wheel(cx, 0.0, gpui::TouchPhase::Ended);
    wheel(cx, -30.0, gpui::TouchPhase::Started);
    assert_eq!(
        outer.offset().y,
        px(-30.0),
        "端から始まる次の操作は祖先に渡す"
    );
    wheel(cx, 30.0, gpui::TouchPhase::Started);
    assert_eq!(
        outer.offset().y,
        px(-30.0),
        "逆方向へ動けるときは内側だけ動く"
    );
}

#[gpui::test]
fn nested_horizontal_scroll_consumes_wheel_until_its_edge(cx: &mut TestAppContext) {
    use gpui_headless::NestedScroll;
    let outer = gpui::ScrollHandle::new();
    let inner = gpui::ScrollHandle::new();
    let cx = support::open(cx, {
        let outer = outer.clone();
        let inner = inner.clone();
        move |window, cx| {
            let id = gpui::ElementId::from("row");
            let wheel = NestedScroll::new(&id, inner.clone(), window, cx);
            div()
                .id("page")
                .w(px(300.0))
                .h(px(300.0))
                .overflow_y_scroll()
                .track_scroll(&outer)
                .child(
                    div()
                        .id(id)
                        .flex()
                        .w(px(200.0))
                        .h(px(100.0))
                        .overflow_x_scroll()
                        .track_scroll(&inner)
                        .on_scroll(wheel.on_scroll())
                        .child(div().flex_none().w(px(500.0)).h(px(50.0)))
                        .child(wheel.capture()),
                )
                .child(div().h(px(1000.0)))
                .into_any_element()
        }
    });
    // 縦ホイールは横だけの領域で x に振り替えられる。
    let wheel = |cx: &mut gpui::VisualTestContext, delta| {
        cx.update(|window, cx| {
            window.dispatch_event(
                gpui::PlatformInput::Scroll(gpui::ScrollEvent {
                    position: gpui::point(px(50.0), px(50.0)),
                    delta: gpui::ScrollDelta::Pixels(gpui::point(px(0.0), px(delta))),
                    ..Default::default()
                }),
                cx,
            );
        });
        support::draw(cx);
    };
    wheel(cx, -100.0);
    assert_eq!(inner.offset().x, px(-100.0));
    assert_eq!(outer.offset().y, px(0.0), "内側が動いたら祖先は動かない");
    wheel(cx, -1000.0);
    assert_eq!(inner.offset().x, px(-300.0), "内側は端で止まる");
    assert_eq!(outer.offset().y, px(0.0));
    wheel(cx, -30.0);
    assert_eq!(outer.offset().y, px(-30.0), "端からの操作は祖先に渡す");
}

#[gpui::test]
fn modal_drawer_falls_back_from_disabled_selection(cx: &mut TestAppContext) {
    use md3_components::navigation::{NavigationDrawer, NavigationDrawerKind};
    let selected = Rc::new(Cell::new(None));
    let cx = support::open(cx, {
        let selected = selected.clone();
        move |_, _| {
            let selected = selected.clone();
            NavigationDrawer::new("drawer")
                .aria_label("移動")
                .kind(NavigationDrawerKind::Modal)
                .item(NavigationItem::new(IconName::Home, "無効").disabled(true))
                .item(NavigationItem::new(IconName::Search, "検索"))
                .item(NavigationItem::new(IconName::Settings, "設定"))
                .selected(Some(0))
                .on_change(move |index, _, _| selected.set(Some(index)))
                .into_any_element()
        }
    });
    support::press(cx, "enter");
    assert_eq!(selected.get(), Some(1));
    support::press(cx, "down");
    support::press(cx, "enter");
    assert_eq!(selected.get(), Some(2));
}

/// 高さを限ったリストで、キーボードで移った項目が見える範囲に入る（WCAG 2.2 の 2.4.11）。
/// 位置は `debug_selector` で測るため、デバッグビルドでだけ走る（gpui はリリースでは記録しない）。
#[cfg(debug_assertions)]
#[gpui::test]
fn focused_item_is_scrolled_into_view(cx: &mut TestAppContext) {
    let cx = support::open(cx, |_, _| {
        (0..20)
            .fold(
                List::new("mail")
                    .aria_label("受信トレイ")
                    .max_height(px(120.0)),
                |list, index| {
                    list.item(
                        format!("k{index}"),
                        ListItem::new(format!("メール {index}")).on_select(|_, _| {}),
                    )
                },
            )
            .into_any_element()
    });
    let item_selector = |index: usize| -> &'static str {
        let id = gpui::ElementId::NamedChild(
            std::sync::Arc::new("mail".into()),
            SharedString::from(format!("k{index}")),
        );
        Box::leak(format!("md3-list-item-{id}").into_boxed_str())
    };
    let visible = |cx: &mut gpui::VisualTestContext, index: usize| {
        let list = cx.debug_bounds("md3-list-mail").expect("list bounds");
        let item = cx.debug_bounds(item_selector(index)).expect("item bounds");
        item.top() >= list.top() && item.bottom() <= list.bottom()
    };
    assert!(!visible(cx, 19), "最初は最後の項目が見えていない");
    reveal_key(cx, "tab");
    reveal_key(cx, "end");
    assert!(visible(cx, 19), "End で移った最後の項目が見える");
    reveal_key(cx, "home");
    assert!(visible(cx, 0), "Home で先頭に戻ると先頭が見える");
    for _ in 0..5 {
        reveal_key(cx, "down");
    }
    assert!(visible(cx, 5), "下矢印で移った項目が見える");
}

/// テスト用プラットフォームにはフレーム源がないので、次フレームの配信だけを予約する。
/// refresh / draw は呼ばない。入力後は run_until_parked だけで位置が反映されることを検証する。
fn queue_reveal_frames(cx: &mut gpui::VisualTestContext) {
    cx.update(|_, cx| cx.set_reduce_motion(true));
    let mut frame_cx = cx.clone();
    cx.foreground_executor()
        .spawn(async move {
            // 既存の Scrollbar はフェード待ちにも次フレームを予約する。
            // 時計を進めずにそれを待ち切ろうとせず、フォーカスと位置の反映分を配信する。
            for _ in 0..3 {
                if frame_cx.update(|window, cx| window.simulate_next_frame(cx)) == 0 {
                    return;
                }
            }
        })
        .detach();
}

/// gallery と同じく、実際のキー入力と focus_next / focus_prev を組み合わせる。
fn reveal_key(cx: &mut gpui::VisualTestContext, key: &'static str) {
    use gpui::{InputEvent, KeyDownEvent, KeyUpEvent, Keystroke};
    queue_reveal_frames(cx);
    let keystroke = Keystroke::parse(key).expect("キー");
    cx.update(|window, cx| {
        window.dispatch_event(
            KeyDownEvent {
                keystroke: keystroke.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        match key {
            "tab" => window.focus_next(cx),
            "shift-tab" => window.focus_prev(cx),
            _ => {}
        }
        window.dispatch_event(KeyUpEvent { keystroke }.to_platform_input(), cx);
    });
    cx.run_until_parked();
}

#[cfg(debug_assertions)]
fn assert_inside(
    cx: &mut gpui::VisualTestContext,
    selector: &'static str,
    viewport: gpui::Bounds<gpui::Pixels>,
    ring: bool,
) {
    let bounds = cx.debug_bounds(selector).expect("項目の位置");
    let bounds = if ring {
        let state = md3_components::theme::md3_tokens::state::FOCUS_INDICATOR_THICKNESS
            + md3_components::theme::md3_tokens::state::FOCUS_INDICATOR_OUTER_OFFSET;
        bounds.dilate(px(state))
    } else {
        bounds
    };
    assert!(
        bounds.top() >= viewport.top()
            && bounds.bottom() <= viewport.bottom()
            && bounds.left() >= viewport.left()
            && bounds.right() <= viewport.right(),
        "{selector}: {bounds:?} が {viewport:?} に収まる"
    );
}

/// Tab・Shift+Tab・循環で、見えなくなったボタンを再描画も含めて追従する。
#[cfg(debug_assertions)]
#[gpui::test]
fn page_reveals_tab_shift_tab_and_wrapping(cx: &mut TestAppContext) {
    use gpui_headless::ScrollScope;
    use md3_components::buttons::Button;
    let scroll = gpui::ScrollHandle::new();
    let cx = support::open(cx, {
        let scroll = scroll.clone();
        move |_, _| {
            ScrollScope::new(
                scroll.clone(),
                div()
                    .id("page")
                    .w(px(240.0))
                    .h(px(120.0))
                    .overflow_y_scroll()
                    .track_scroll(&scroll)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .p(px(10.0))
                            .gap(px(150.0))
                            .child(
                                div()
                                    .debug_selector(|| "first".into())
                                    .child(Button::new("first", "先頭")),
                            )
                            .child(
                                div()
                                    .debug_selector(|| "middle".into())
                                    .child(Button::new("middle", "中央")),
                            )
                            .child(
                                div()
                                    .debug_selector(|| "last".into())
                                    .child(Button::new("last", "末尾")),
                            ),
                    ),
            )
            .into_any_element()
        }
    });
    assert!(cx.debug_bounds("last").expect("末尾").top() > scroll.bounds().bottom());
    reveal_key(cx, "tab");
    assert_inside(cx, "first", scroll.bounds(), true);
    reveal_key(cx, "tab");
    assert_inside(cx, "middle", scroll.bounds(), true);
    reveal_key(cx, "tab");
    assert_inside(cx, "last", scroll.bounds(), true);
    assert!(scroll.offset().y < px(0.0));
    reveal_key(cx, "shift-tab");
    assert_inside(cx, "middle", scroll.bounds(), true);
    reveal_key(cx, "tab");
    reveal_key(cx, "tab");
    assert_inside(cx, "first", scroll.bounds(), true);
    assert!(scroll.offset().y >= px(-10.0), "先頭の余白まで戻る");
    reveal_key(cx, "shift-tab");
    assert_inside(cx, "last", scroll.bounds(), true);
}

/// control を使わない Card と入力欄も同じ領域へ追従する。
#[cfg(debug_assertions)]
#[gpui::test]
fn page_reveals_card_and_text_field(cx: &mut TestAppContext) {
    use gpui_headless::ScrollScope;
    use md3_components::{
        buttons::Button,
        containment::{Card, CardVariant},
        inputs::TextField,
    };
    let scroll = gpui::ScrollHandle::new();
    let state = cx.update(TextFieldState::new);
    let cx = support::open(cx, {
        let scroll = scroll.clone();
        move |_, _| {
            ScrollScope::new(
                scroll.clone(),
                div()
                    .id("page")
                    .w(px(300.0))
                    .h(px(160.0))
                    .overflow_y_scroll()
                    .track_scroll(&scroll)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .p(px(10.0))
                            .gap(px(200.0))
                            .child(Button::new("start", "開始"))
                            .child(
                                div().debug_selector(|| "card".into()).child(
                                    Card::new(CardVariant::Filled)
                                        .on_click("card", |_, _, _| {})
                                        .aria_label("カード")
                                        .h(px(64.0))
                                        .child("カード"),
                                ),
                            )
                            .child(
                                div()
                                    .debug_selector(|| "field".into())
                                    .child(TextField::new("field", &state).label("名前")),
                            ),
                    ),
            )
            .into_any_element()
        }
    });
    reveal_key(cx, "tab");
    reveal_key(cx, "tab");
    assert_inside(cx, "card", scroll.bounds(), true);
    let card_offset = scroll.offset().y;
    reveal_key(cx, "tab");
    assert_inside(cx, "field", scroll.bounds(), true);
    assert!(scroll.offset().y < card_offset);
}

/// 内側で制限された移動量を外側に反映し、両方の表示領域に項目を収める。
#[cfg(debug_assertions)]
#[gpui::test]
fn nested_list_reveals_item_in_both_scopes(cx: &mut TestAppContext) {
    use gpui_headless::ScrollScope;
    use md3_components::buttons::Button;
    let scroll = gpui::ScrollHandle::new();
    let cx = support::open(cx, {
        let scroll = scroll.clone();
        move |_, _| {
            ScrollScope::new(
                scroll.clone(),
                div()
                    .id("page")
                    .w(px(320.0))
                    .h(px(200.0))
                    .overflow_y_scroll()
                    .track_scroll(&scroll)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .p(px(10.0))
                            .gap(px(300.0))
                            .child(Button::new("start", "開始"))
                            .child((0..20).fold(
                                List::new("nested").aria_label("一覧").max_height(px(120.0)),
                                |list, i| {
                                    list.item(
                                        format!("k{i}"),
                                        ListItem::new(format!("項目 {i}")).on_select(|_, _| {}),
                                    )
                                },
                            ))
                            .child(div().h(px(200.0))),
                    ),
            )
            .into_any_element()
        }
    });
    reveal_key(cx, "tab");
    reveal_key(cx, "tab");
    reveal_key(cx, "end");
    let id = gpui::ElementId::NamedChild(std::sync::Arc::new("nested".into()), "k19".into());
    let selector = Box::leak(format!("md3-list-item-{id}").into_boxed_str());
    let list = cx.debug_bounds("md3-list-nested").expect("リスト");
    assert_inside(cx, selector, list, false);
    assert_inside(cx, selector, scroll.bounds(), true);
    assert!(scroll.offset().y < px(0.0));
    reveal_key(cx, "home");
    let id = gpui::ElementId::NamedChild(std::sync::Arc::new("nested".into()), "k0".into());
    let selector = Box::leak(format!("md3-list-item-{id}").into_boxed_str());
    let list = cx.debug_bounds("md3-list-nested").expect("リスト");
    assert_inside(cx, selector, list, false);
    assert_inside(cx, selector, scroll.bounds(), false);
}

#[gpui::test]
fn pointer_focus_does_not_reveal_a_partly_hidden_button(cx: &mut TestAppContext) {
    use gpui_headless::ScrollScope;
    use md3_components::buttons::Button;
    let scroll = gpui::ScrollHandle::new();
    let clicked = Rc::new(Cell::new(false));
    let cx = support::open(cx, {
        let scroll = scroll.clone();
        let clicked = clicked.clone();
        move |_, _| {
            let clicked = clicked.clone();
            ScrollScope::new(
                scroll.clone(),
                div()
                    .id("page")
                    .w(px(240.0))
                    .h(px(120.0))
                    .overflow_y_scroll()
                    .track_scroll(&scroll)
                    .child(div().h(px(100.0)))
                    .child(
                        Button::new("partial", "一部が見える")
                            .on_click(move |_, _, _| clicked.set(true)),
                    )
                    .child(div().h(px(200.0))),
            )
            .into_any_element()
        }
    });
    queue_reveal_frames(cx);
    let viewport = scroll.bounds();
    cx.simulate_click(
        gpui::point(viewport.left() + px(30.0), viewport.bottom() - px(10.0)),
        gpui::Modifiers::none(),
    );
    cx.run_until_parked();
    assert!(clicked.get(), "一部が見えるボタンをクリックした");
    assert!(cx.update(|window, cx| { window.focused(cx).is_some() }));
    assert_eq!(scroll.offset().y, px(0.0), "マウスではスクロールしない");
}

#[gpui::test]
fn wheel_does_not_pull_back_the_focused_element(cx: &mut TestAppContext) {
    use gpui_headless::ScrollScope;
    use md3_components::buttons::Button;
    let scroll = gpui::ScrollHandle::new();
    let cx = support::open(cx, {
        let scroll = scroll.clone();
        move |_, _| {
            ScrollScope::new(
                scroll.clone(),
                div()
                    .id("page")
                    .w(px(240.0))
                    .h(px(120.0))
                    .overflow_y_scroll()
                    .track_scroll(&scroll)
                    .child(div().h(px(300.0)))
                    .child(Button::new("target", "対象"))
                    .child(div().h(px(300.0))),
            )
            .into_any_element()
        }
    });
    reveal_key(cx, "tab");
    let revealed = scroll.offset().y;
    assert!(revealed < px(0.0));
    queue_reveal_frames(cx);
    let viewport = scroll.bounds();
    cx.simulate_event(gpui::ScrollEvent {
        position: gpui::point(viewport.left() + px(30.0), viewport.top() + px(30.0)),
        delta: gpui::ScrollDelta::Pixels(gpui::point(px(0.0), px(100.0))),
        ..Default::default()
    });
    cx.run_until_parked();
    let after_wheel = scroll.offset();
    assert!(after_wheel.y > revealed, "ホイールで表示を離した");
    reveal_key(cx, "shift");
    assert_eq!(
        scroll.offset(),
        after_wheel,
        "フォーカスが同じならキーを押しても引き戻さない"
    );
}

/// メニューの項目は tab_stop=false でも矢印で見える。遅延描画時に背後の領域は使わない。
#[cfg(debug_assertions)]
#[gpui::test]
fn deferred_menu_reveals_arrow_focus_without_scrolling_the_page(cx: &mut TestAppContext) {
    use gpui_headless::ScrollScope;
    use md3_components::{
        buttons::Button,
        overlays::{Menu, MenuItem},
    };
    let scroll = gpui::ScrollHandle::new();
    let open = Rc::new(Cell::new(false));
    let cx = support::open(cx, {
        let scroll = scroll.clone();
        let open = open.clone();
        move |_, _| {
            let toggle = open.clone();
            ScrollScope::new(
                scroll.clone(),
                div()
                    .id("page")
                    .w(px(320.0))
                    .h(px(160.0))
                    .overflow_y_scroll()
                    .track_scroll(&scroll)
                    .child(
                        div()
                            .relative()
                            .child(Button::new("open", "開く").on_click(move |_, window, _| {
                                toggle.set(true);
                                window.refresh();
                            }))
                            .when(open.get(), |this| {
                                this.child((0..40).fold(
                                    Menu::new("long-menu").aria_label("操作"),
                                    |menu, i| menu.item(MenuItem::new(format!("操作 {i}"))),
                                ))
                            }),
                    )
                    .child(div().h(px(1000.0))),
            )
            .into_any_element()
        }
    });
    reveal_key(cx, "tab");
    let before_menu = scroll.offset();
    reveal_key(cx, "enter");
    reveal_key(cx, "end");
    let selector = |index| -> &'static str {
        let id = gpui::ElementId::NamedChild(
            std::sync::Arc::new("long-menu".into()),
            format!("item-{index}").into(),
        );
        Box::leak(format!("md3-menu-item-{id}").into_boxed_str())
    };
    let viewport = cx.debug_bounds("md3-menu-long-menu").expect("メニュー");
    assert_inside(cx, selector(39), viewport, false);
    assert_eq!(scroll.offset(), before_menu, "背後のページは動かない");
    reveal_key(cx, "home");
    assert_inside(cx, selector(0), viewport, false);
    reveal_key(cx, "up");
    assert_inside(cx, selector(39), viewport, false);
    assert_eq!(scroll.offset(), before_menu);
}
