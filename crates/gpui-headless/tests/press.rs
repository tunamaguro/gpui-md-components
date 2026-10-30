//! 押下の最短時間と解除を検証する。

use std::time::Duration;

use gpui::{AppContext, TestAppContext};
use gpui_headless::press::Press;

#[gpui::test]
fn short_press_waits_for_the_supplied_duration(cx: &mut TestAppContext) {
    let hold = Duration::from_millis(100);
    let press = cx.new(|_| Press::default());
    press.update(cx, |press, cx| press.pulse(hold, cx));
    assert!(press.read_with(cx, |press, _| press.is_pressed()));
    cx.executor().advance_clock(hold);
    cx.run_until_parked();
    assert!(!press.read_with(cx, |press, _| press.is_pressed()));
}

#[gpui::test]
fn long_press_waits_for_release(cx: &mut TestAppContext) {
    let hold = Duration::from_millis(100);
    let press = cx.new(|_| Press::default());
    press.update(cx, |press, cx| press.press(hold, cx));
    cx.executor().advance_clock(hold * 2);
    cx.run_until_parked();
    assert!(press.read_with(cx, |press, _| press.is_pressed()));
    press.update(cx, |press, cx| press.release(cx));
    assert!(!press.read_with(cx, |press, _| press.is_pressed()));
}

#[gpui::test]
fn zero_duration_releases_immediately(cx: &mut TestAppContext) {
    let press = cx.new(|_| Press::default());
    press.update(cx, |press, cx| press.pulse(Duration::ZERO, cx));
    assert!(!press.read_with(cx, |press, _| press.is_pressed()));
}
