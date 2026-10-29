//! Snackbar の置換・順番待ち・結果通知と退場の検証。

mod support;

use gpui::{AppContext, IntoElement, TestAppContext};
use md3_components::overlays::{
    SnackbarDuration, SnackbarHost, SnackbarMessage, SnackbarPolicy, SnackbarResult,
};
use std::{cell::RefCell, rc::Rc, time::Duration};

#[gpui::test]
fn result_handler_can_show_a_follow_up_after_the_host_update(cx: &mut TestAppContext) {
    cx.update(|cx| cx.set_reduce_motion(true));
    let host = cx.new(|_| SnackbarHost::new("閉じる"));
    let log = Rc::new(RefCell::new(Vec::new()));
    let follow_up_host = host.clone();
    let result_log = log.clone();
    host.update(cx, |host, cx| {
        host.show(
            SnackbarMessage::new("最初")
                .duration(SnackbarDuration::Indefinite)
                .on_result(move |result, cx| {
                    result_log.borrow_mut().push(result);
                    follow_up_host.update(cx, |host, cx| {
                        assert!(host.current().is_none(), "元の更新を完了してから通知する");
                        host.show(
                            SnackbarMessage::new("結果からの知らせ")
                                .duration(SnackbarDuration::Indefinite),
                            cx,
                        );
                    });
                }),
            cx,
        );
        host.dismiss_current(cx);
        assert!(log.borrow().is_empty(), "更新中は結果を通知しない");
    });
    cx.run_until_parked();
    assert_eq!(*log.borrow(), [SnackbarResult::Dismissed]);
    assert_eq!(
        host.read_with(cx, |host, _| host.current().cloned())
            .as_deref(),
        Some("結果からの知らせ")
    );
    host.update(cx, |host, cx| host.dismiss_current(cx));
    assert_eq!(log.borrow().len(), 1, "結果通知は一度だけ");
}

#[gpui::test]
fn replaced_pending_result_handler_can_show_a_follow_up(cx: &mut TestAppContext) {
    let host = cx.new(|_| SnackbarHost::new("閉じる").policy(SnackbarPolicy::Queue));
    let view = host.clone();
    let cx = support::open(cx, move |_, _| view.clone().into_any_element());
    let log = Rc::new(RefCell::new(Vec::new()));
    let result_log = log.clone();
    let follow_up_host = host.clone();
    host.update(cx, |host, cx| {
        host.show(
            SnackbarMessage::new("表示中").duration(SnackbarDuration::Indefinite),
            cx,
        );
        host.show(
            SnackbarMessage::new("待機中").on_result(move |result, cx| {
                result_log.borrow_mut().push(result);
                follow_up_host.update(cx, |host, cx| {
                    host.show(
                        SnackbarMessage::new("待機分の結果からの知らせ")
                            .duration(SnackbarDuration::Indefinite),
                        cx,
                    );
                });
            }),
            cx,
        );
        host.set_policy(SnackbarPolicy::Replace, cx);
        host.show(SnackbarMessage::new("置換"), cx);
        assert!(log.borrow().is_empty(), "待機分も更新後に通知する");
    });
    cx.run_until_parked();
    assert_eq!(*log.borrow(), [SnackbarResult::Dismissed]);
    cx.update(|_, cx| cx.set_reduce_motion(true));
    support::draw(cx);
    assert_eq!(
        host.read_with(cx, |host, _| host.current().cloned())
            .as_deref(),
        Some("待機分の結果からの知らせ")
    );
    host.update(cx, |host, cx| host.dismiss_current(cx));
    assert_eq!(log.borrow().len(), 1, "待機分の結果通知も一度だけ");
}

#[gpui::test]
fn replace_keeps_outgoing_until_exit_and_discards_superseded_pending(cx: &mut TestAppContext) {
    let log = Rc::new(RefCell::new(Vec::new()));
    let host = cx.new(|_| SnackbarHost::new("閉じる"));
    let view = host.clone();
    let cx = support::open(cx, move |_, _| view.clone().into_any_element());
    let result = log.clone();
    host.update(cx, |host, cx| {
        host.show(
            SnackbarMessage::new("一番目")
                .on_result(move |result_value, _| result.borrow_mut().push((1, result_value))),
            cx,
        );
    });
    support::draw(cx);
    cx.executor().advance_clock(Duration::from_secs(1));
    support::draw(cx);
    let result = log.clone();
    host.update(cx, |host, cx| {
        host.show(
            SnackbarMessage::new("二番目")
                .on_result(move |value, _| result.borrow_mut().push((2, value))),
            cx,
        );
    });
    assert_eq!(
        host.read_with(cx, |host, _| host.current().cloned())
            .as_deref(),
        Some("一番目")
    );
    host.update(cx, |host, cx| {
        host.show(
            SnackbarMessage::new("最新").duration(SnackbarDuration::Indefinite),
            cx,
        );
    });
    assert_eq!(
        *log.borrow(),
        [
            (1, SnackbarResult::Dismissed),
            (2, SnackbarResult::Dismissed)
        ]
    );
    support::draw(cx);
    // 依存先の scheduler::Instant はこの統合テストでは実時間。描画経路外で落ち着くまで待つ。
    std::thread::sleep(Duration::from_secs(1));
    support::draw(cx);
    assert_eq!(
        host.read_with(cx, |host, _| host.current().cloned())
            .as_deref(),
        Some("最新")
    );
    // 前の表示タイマーが新しい知らせを閉じない。
    cx.executor().advance_clock(Duration::from_secs(10));
    support::draw(cx);
    assert_eq!(
        host.read_with(cx, |host, _| host.current().cloned())
            .as_deref(),
        Some("最新")
    );
}

#[gpui::test]
fn queue_waits_and_reduced_motion_replaces_without_waiting(cx: &mut TestAppContext) {
    cx.update(|cx| cx.set_reduce_motion(true));
    let host = cx.new(|_| SnackbarHost::new("閉じる").policy(SnackbarPolicy::Queue));
    host.update(cx, |host, cx| {
        host.show(SnackbarMessage::new("短い"), cx);
        host.show(
            SnackbarMessage::new("操作付き").action("戻す", |_, _| {}),
            cx,
        );
    });
    cx.run_until_parked();
    cx.executor().advance_clock(Duration::from_secs(4));
    cx.run_until_parked();
    assert_eq!(
        host.read_with(cx, |host, _| host.current().cloned())
            .as_deref(),
        Some("操作付き")
    );
    cx.executor().advance_clock(Duration::from_secs(30));
    cx.run_until_parked();
    assert_eq!(
        host.read_with(cx, |host, _| host.current().cloned())
            .as_deref(),
        Some("操作付き")
    );
    host.update(cx, |host, cx| {
        host.set_policy(SnackbarPolicy::Replace, cx);
        host.show(SnackbarMessage::new("最新"), cx);
        assert_eq!(host.current().map(|s| s.as_ref()), Some("最新"));
    });
}

#[gpui::test]
fn action_reports_action_once_and_moves_to_next(cx: &mut TestAppContext) {
    cx.update(|cx| cx.set_reduce_motion(true));
    let log = Rc::new(RefCell::new(Vec::new()));
    let host = cx.new(|_| SnackbarHost::new("閉じる").policy(SnackbarPolicy::Queue));
    let view = host.clone();
    let cx = support::open(cx, move |_, _| view.clone().into_any_element());
    let result = log.clone();
    host.update(cx, |host, cx| {
        host.show(
            SnackbarMessage::new("操作付き")
                .action("戻す", |_, _| {})
                .on_result(move |value, _| result.borrow_mut().push(value)),
            cx,
        );
        host.show(SnackbarMessage::new("次"), cx);
    });
    support::draw(cx);
    support::tab(cx);
    support::press(cx, "enter");
    assert_eq!(*log.borrow(), [SnackbarResult::ActionPerformed]);
    assert_eq!(
        host.read_with(cx, |host, _| host.current().cloned())
            .as_deref(),
        Some("次")
    );
}

#[gpui::test]
fn enabling_reduced_motion_finishes_an_in_flight_exit(cx: &mut TestAppContext) {
    let host = cx.new(|_| SnackbarHost::new("閉じる"));
    let view = host.clone();
    let cx = support::open(cx, move |_, _| view.clone().into_any_element());
    host.update(cx, |host, cx| {
        host.show(SnackbarMessage::new("古い"), cx);
    });
    support::draw(cx);
    cx.executor().advance_clock(Duration::from_secs(1));
    support::draw(cx);
    host.update(cx, |host, cx| {
        host.show(SnackbarMessage::new("新しい"), cx);
    });
    cx.update(|_, cx| cx.set_reduce_motion(true));
    support::draw(cx);
    assert_eq!(
        host.read_with(cx, |host, _| host.current().cloned())
            .as_deref(),
        Some("新しい")
    );
}
