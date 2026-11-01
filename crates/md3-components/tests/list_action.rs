//! リストの行と行内の操作を組み合わせた入力・フォーカスの検証。

#![cfg(debug_assertions)]

mod support;

use gpui::{
    AnyElement, ElementId, InputEvent, KeyDownEvent, KeyUpEvent, Keystroke, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, ScrollDelta, ScrollEvent,
    TestAppContext, UniformListScrollHandle, VisualTestContext, div, point, prelude::*, px,
};
use md3_components::{
    IconName,
    buttons::Button,
    containment::{LazyList, List, ListAction, ListItem, ListTrailing},
};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Event {
    Before,
    Row(usize),
    Action(usize),
    After,
}
type Log = Rc<RefCell<Vec<Event>>>;

#[derive(Clone, Copy)]
enum Kind {
    Enabled,
    Static,
    Disabled,
    ActionDisabled,
}

fn item(index: usize, kind: Kind, log: &Log) -> ListItem {
    let action_log = log.clone();
    let mut item = ListItem::new(format!("項目 {index}"))
        .trailing(ListTrailing::Text("10:24".into()))
        .action(
            ListAction::new(IconName::Star)
                .aria_label(format!("項目 {index} のスター"))
                .selected(false)
                .disabled(matches!(kind, Kind::ActionDisabled))
                .on_click(move |_, _| action_log.borrow_mut().push(Event::Action(index))),
        )
        .disabled(matches!(kind, Kind::Disabled));
    if !matches!(kind, Kind::Static) {
        let log = log.clone();
        item = item.on_select(move |_, _| log.borrow_mut().push(Event::Row(index)));
    }
    item
}

fn sentinels(content: impl IntoElement, log: &Log) -> AnyElement {
    let before = log.clone();
    let after = log.clone();
    div()
        .flex()
        .flex_col()
        .w(px(360.))
        .child(
            Button::new("before", "前")
                .on_click(move |_, _, _| before.borrow_mut().push(Event::Before)),
        )
        .child(content)
        .child(
            Button::new("after", "後")
                .on_click(move |_, _, _| after.borrow_mut().push(Event::After)),
        )
        .into_any_element()
}

fn open_list<'a>(
    cx: &'a mut TestAppContext,
    log: &Log,
    kinds: &'static [Kind],
    max_height: Option<Pixels>,
) -> &'a mut VisualTestContext {
    let log = log.clone();
    support::open(cx, move |_, _| {
        let mut list = List::new("list").aria_label("一覧");
        if let Some(height) = max_height {
            list = list.max_height(height);
        }
        for (index, &kind) in kinds.iter().enumerate() {
            list = list.item(format!("k{index}"), item(index, kind, &log));
        }
        sentinels(list, &log)
    })
}

fn activate(cx: &mut VisualTestContext, log: &Log, expected: Event) {
    log.borrow_mut().clear();
    support::press(cx, "enter");
    assert_eq!(
        *log.borrow(),
        [expected],
        "フォーカス中の対象だけを実行する"
    );
}

fn selector(list: &str, index: usize, action: bool) -> &'static str {
    let row = ElementId::NamedChild(
        ElementId::from(list.to_owned()).into(),
        format!("k{index}").into(),
    );
    Box::leak(
        if action {
            let id = ElementId::NamedChild(row.into(), "action".into());
            format!("md3-icon-button-{id}")
        } else {
            format!("md3-list-item-{row}")
        }
        .into_boxed_str(),
    )
}

/// キーを押して離す。`support::press` と違い、入力が起こす描画のほかに描き直さない。
fn press_without_redraw(cx: &mut VisualTestContext, key: &'static str) {
    let keystroke = Keystroke::parse(key)
        .map_err(|error| error.to_string())
        .expect("valid keystroke");
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
        window.dispatch_event(KeyUpEvent { keystroke }.to_platform_input(), cx);
    });
    cx.run_until_parked();
}

fn send(cx: &mut VisualTestContext, event: impl InputEvent) {
    cx.update(|window, cx| window.dispatch_event(event.to_platform_input(), cx));
    support::draw(cx);
}

fn click(cx: &mut VisualTestContext, position: Point<Pixels>) {
    send(
        cx,
        MouseMoveEvent {
            position,
            ..Default::default()
        },
    );
    send(
        cx,
        MouseDownEvent {
            position,
            button: MouseButton::Left,
            modifiers: Default::default(),
            click_count: 1,
            first_mouse: false,
        },
    );
    send(
        cx,
        MouseUpEvent {
            position,
            button: MouseButton::Left,
            modifiers: Default::default(),
            click_count: 1,
        },
    );
}

#[gpui::test]
fn pointer_action_does_not_select_the_row(cx: &mut TestAppContext) {
    let log = Log::default();
    let cx = open_list(cx, &log, &[Kind::Enabled], None);
    let row = cx
        .debug_bounds(selector("list", 0, false))
        .expect("行の位置");
    click(cx, point(row.left() + px(24.), row.center().y));
    assert_eq!(*log.borrow(), [Event::Row(0)]);
    log.borrow_mut().clear();
    let button = cx
        .debug_bounds(selector("list", 0, true))
        .expect("操作の位置");
    click(cx, button.center());
    assert_eq!(
        *log.borrow(),
        [Event::Action(0)],
        "行にはクリックを渡さない"
    );
}

#[gpui::test]
fn enter_and_space_only_activate_the_focused_action(cx: &mut TestAppContext) {
    let log = Log::default();
    let cx = open_list(cx, &log, &[Kind::Enabled], None);
    support::tab(cx);
    support::tab(cx);
    activate(cx, &log, Event::Row(0));
    support::tab(cx);
    for key in ["enter", "space"] {
        log.borrow_mut().clear();
        support::press(cx, key);
        assert_eq!(*log.borrow(), [Event::Action(0)]);
    }
}

#[gpui::test]
fn tab_and_shift_tab_visit_row_action_and_outside(cx: &mut TestAppContext) {
    let log = Log::default();
    let cx = open_list(
        cx,
        &log,
        &[Kind::Enabled, Kind::Enabled, Kind::Enabled],
        None,
    );
    for expected in [Event::Before, Event::Row(0), Event::Action(0), Event::After] {
        support::tab(cx);
        activate(cx, &log, expected);
    }
    for expected in [Event::Action(0), Event::Row(0), Event::Before] {
        support::press(cx, "shift-tab");
        activate(cx, &log, expected);
    }
}

#[gpui::test]
fn arrows_update_the_tab_row_and_shift_tab_can_leave(cx: &mut TestAppContext) {
    let log = Log::default();
    let cx = open_list(
        cx,
        &log,
        &[Kind::Enabled, Kind::Enabled, Kind::Enabled],
        None,
    );
    support::tab(cx);
    support::tab(cx);
    support::press(cx, "down");
    support::press(cx, "down");
    activate(cx, &log, Event::Row(2));
    support::tab(cx);
    activate(cx, &log, Event::Action(2));
    support::tab(cx);
    activate(cx, &log, Event::After);
    support::press(cx, "shift-tab");
    activate(cx, &log, Event::Action(0));
    support::press(cx, "down");
    support::press(cx, "down");
    activate(cx, &log, Event::Row(2));
    support::press(cx, "shift-tab");
    activate(cx, &log, Event::Before);
}

#[gpui::test]
fn action_arrows_home_and_end_move_to_rows(cx: &mut TestAppContext) {
    let log = Log::default();
    let cx = open_list(
        cx,
        &log,
        &[Kind::Enabled, Kind::Enabled, Kind::Enabled],
        None,
    );
    support::tab(cx);
    support::tab(cx);
    support::tab(cx);
    activate(cx, &log, Event::Action(0));
    for (key, row) in [("down", 1), ("end", 2), ("up", 1), ("home", 0)] {
        support::press(cx, key);
        activate(cx, &log, Event::Row(row));
        support::tab(cx);
        activate(cx, &log, Event::Action(row));
    }
}

#[gpui::test]
fn static_action_is_a_tab_stop_and_disabled_actions_are_inert(cx: &mut TestAppContext) {
    let log = Log::default();
    let cx = open_list(
        cx,
        &log,
        &[Kind::Static, Kind::Disabled, Kind::ActionDisabled],
        None,
    );
    for expected in [Event::Before, Event::Action(0), Event::Row(2), Event::After] {
        support::tab(cx);
        activate(cx, &log, expected);
    }
    let enabled = cx
        .debug_bounds(selector("list", 0, true))
        .expect("有効な操作");
    log.borrow_mut().clear();
    click(cx, enabled.center());
    assert_eq!(*log.borrow(), [Event::Action(0)]);
    log.borrow_mut().clear();
    for index in [1, 2] {
        let button = cx
            .debug_bounds(selector("list", index, true))
            .expect("無効な操作");
        click(cx, button.center());
        assert!(log.borrow().is_empty(), "無効な操作は行も実行しない");
    }
}

#[gpui::test]
fn wheel_over_action_scrolls_the_list(cx: &mut TestAppContext) {
    let log = Log::default();
    let cx = open_list(cx, &log, &[Kind::Enabled; 12], Some(px(150.)));
    let before = cx
        .debug_bounds(selector("list", 0, true))
        .expect("操作の位置");
    send(
        cx,
        ScrollEvent {
            position: before.center(),
            delta: ScrollDelta::Pixels(point(px(0.), px(-60.))),
            ..Default::default()
        },
    );
    let after = cx
        .debug_bounds(selector("list", 0, true))
        .expect("スクロール後の位置");
    assert!(
        after.top() < before.top(),
        "ボタンの上でもリストをスクロールする"
    );
    assert!(log.borrow().is_empty());
}

fn open_lazy<'a>(cx: &'a mut TestAppContext, log: &Log) -> &'a mut VisualTestContext {
    support::open(cx, {
        let log = log.clone();
        let scroll = UniformListScrollHandle::new();
        move |_, _| {
            let item_log = log.clone();
            sentinels(
                LazyList::new("lazy", 20, &scroll, move |index, _| {
                    (
                        format!("k{index}").into(),
                        item(index, Kind::Enabled, &item_log),
                    )
                })
                .aria_label("仮想一覧")
                .h(px(200.)),
                &log,
            )
        }
    })
}

#[gpui::test]
fn lazy_tab_right_after_an_arrow_uses_the_new_row(cx: &mut TestAppContext) {
    let log = Log::default();
    let cx = open_lazy(cx, &log);
    support::tab(cx);
    support::tab(cx);
    press_without_redraw(cx, "down");
    press_without_redraw(cx, "tab");
    activate(cx, &log, Event::Action(1));
}

#[gpui::test]
fn lazy_shift_tab_right_after_an_arrow_leaves_the_list(cx: &mut TestAppContext) {
    let log = Log::default();
    let cx = open_lazy(cx, &log);
    support::tab(cx);
    support::tab(cx);
    press_without_redraw(cx, "down");
    press_without_redraw(cx, "shift-tab");
    activate(cx, &log, Event::Before);
}

#[gpui::test]
fn lazy_action_click_updates_the_tab_target(cx: &mut TestAppContext) {
    let log = Log::default();
    let cx = open_lazy(cx, &log);
    support::tab(cx);
    support::tab(cx);
    activate(cx, &log, Event::Row(0));
    support::tab(cx);
    activate(cx, &log, Event::Action(0));
    let third = cx
        .debug_bounds(selector("lazy", 2, true))
        .expect("3 行目の操作");
    log.borrow_mut().clear();
    click(cx, third.center());
    assert_eq!(*log.borrow(), [Event::Action(2)]);
    support::tab(cx);
    activate(cx, &log, Event::After);
    support::press(cx, "shift-tab");
    activate(cx, &log, Event::Action(2));
    support::press(cx, "shift-tab");
    activate(cx, &log, Event::Row(2));
}
