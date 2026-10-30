//! Snackbar のキュー（見た目なし）。

use gpui::{AppContext, Context, TestAppContext};
use gpui_headless::{SnackbarEntry, SnackbarOwner, SnackbarPolicy, SnackbarQueue};
use std::{cell::RefCell, rc::Rc, time::Duration};

struct Owner(SnackbarQueue);

impl SnackbarOwner for Owner {
    fn snackbar_queue(&mut self) -> &mut SnackbarQueue {
        &mut self.0
    }
}

fn show(owner: &mut Owner, text: &'static str, cx: &mut Context<Owner>) {
    owner.0.show(SnackbarEntry::new(text), cx);
}

#[gpui::test]
fn queued_entries_wait_their_turn(cx: &mut TestAppContext) {
    let owner = cx.new(|_| Owner(SnackbarQueue::new().policy(SnackbarPolicy::Queue)));
    owner.update(cx, |owner, cx| {
        show(owner, "一", cx);
        show(owner, "二", cx);
    });
    owner.read_with(cx, |owner, _| {
        assert_eq!(owner.0.current().map(|s| s.as_ref()), Some("一"));
    });
    let weak = owner.downgrade();
    owner.update(cx, |owner, cx| owner.0.dismiss_current(cx));
    let front = owner.read_with(cx, |owner, _| owner.0.front(weak.clone()).expect("front"));
    assert!(front.exiting, "演出が終わるまで残る");
    cx.update(|cx| SnackbarQueue::finish_exit(&weak, front.id, cx));
    owner.read_with(cx, |owner, _| {
        assert_eq!(owner.0.current().map(|s| s.as_ref()), Some("二"));
    });
}

#[gpui::test]
fn the_timer_dismisses_and_reports_once(cx: &mut TestAppContext) {
    let results = Rc::new(RefCell::new(Vec::new()));
    let owner = cx.new(|_| Owner(SnackbarQueue::new()));
    let log = results.clone();
    owner.update(cx, |owner, cx| {
        let entry = SnackbarEntry::new("一")
            .duration(Some(Duration::from_secs(4)))
            .on_result(move |result, _| log.borrow_mut().push(result));
        owner.0.show(entry, cx);
    });
    cx.executor().advance_clock(Duration::from_secs(3));
    assert!(results.borrow().is_empty());
    cx.executor().advance_clock(Duration::from_secs(2));
    cx.run_until_parked();
    assert_eq!(results.borrow().len(), 1);
}
