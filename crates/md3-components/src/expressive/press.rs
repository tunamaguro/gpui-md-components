// Ported from Jetpack Compose (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ButtonGroup.kt @ b8256a52
// Copyright 2024 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
//! 押下の追跡に使うテーマの時間。
use gpui::{App, Entity, prelude::*};
pub(crate) use gpui_headless::press::{Press, press_state};
use md3_theme::{Md3ThemeExt, MotionToken, time_to_reach};
use std::time::Duration;

/// Compose ButtonGroup の押下アニメーションの到達率。
const RELEASE_FRACTION: f32 = 0.75;

pub(crate) fn hold(cx: &App) -> Duration {
    if cx.reduce_motion() {
        Duration::ZERO
    } else {
        time_to_reach(cx.md3().spring(MotionToken::FastSpatial), RELEASE_FRACTION)
    }
}

pub(crate) fn track_press<E: InteractiveElement>(element: E, press: &Entity<Press>) -> E {
    gpui_headless::press::track_press(element, press, hold)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;

    #[gpui::test]
    fn short_press_is_held_until_the_shape_has_moved(cx: &mut TestAppContext) {
        let press = cx.new(|_| Press::default());
        let hold = cx.update(|cx| hold(cx));
        assert!(hold > Duration::ZERO);
        press.update(cx, |press, cx| press.pulse(hold, cx));
        assert!(press.read_with(cx, |press, _| press.is_pressed()));
        cx.executor().advance_clock(hold);
        cx.run_until_parked();
        assert!(!press.read_with(cx, |press, _| press.is_pressed()));
    }

    #[gpui::test]
    fn long_press_ends_on_release(cx: &mut TestAppContext) {
        let press = cx.new(|_| Press::default());
        let hold = cx.update(|cx| hold(cx));
        press.update(cx, |press, cx| press.press(hold, cx));
        cx.executor().advance_clock(hold * 2);
        cx.run_until_parked();
        assert!(press.read_with(cx, |press, _| press.is_pressed()));
        press.update(cx, |press, cx| press.release(cx));
        assert!(!press.read_with(cx, |press, _| press.is_pressed()));
    }

    #[gpui::test]
    fn reduced_motion_does_not_hold_the_press(cx: &mut TestAppContext) {
        cx.update(|cx| cx.set_reduce_motion(true));
        let hold = cx.update(|cx| hold(cx));
        assert_eq!(hold, Duration::ZERO);
        let press = cx.new(|_| Press::default());
        press.update(cx, |press, cx| press.pulse(hold, cx));
        assert!(!press.read_with(cx, |press, _| press.is_pressed()));
    }
}
