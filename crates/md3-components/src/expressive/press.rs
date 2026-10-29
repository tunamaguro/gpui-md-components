//! 押下の追跡（形と幅のバネの目標を決める）。

use std::time::Duration;

use gpui::{App, Context, ElementId, Entity, MouseButton, SharedString, Task, Window, prelude::*};
use md3_theme::{Md3ThemeExt, MotionToken, time_to_reach};

/// 押下を離した後、形の変化をここまで進めてから戻す（Compose の ButtonGroup の `pressedAnimatable.value > 0.75f`）。
const RELEASE_FRACTION: f32 = 0.75;

/// 押下の状態。
///
/// 短いクリックやキーボードでの押下でも形の変化が見えるように、押下から [`Press::hold`] の間は
/// 離しても押下のままにする。
#[derive(Default)]
pub(crate) struct Press {
    pressed: bool,
    held: bool,
    release_pending: bool,
    timer: Option<Task<()>>,
}

impl Press {
    /// 押下中か（バネの目標を押下の形にするか）。
    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    /// 押下から離すまでに最低限待つ時間。モーションを減らす設定なら待たない。
    pub fn hold(cx: &App) -> Duration {
        if cx.reduce_motion() {
            Duration::ZERO
        } else {
            time_to_reach(cx.md3().spring(MotionToken::FastSpatial), RELEASE_FRACTION)
        }
    }

    /// 押下を始める。
    pub fn press(&mut self, hold: Duration, cx: &mut Context<Self>) {
        self.pressed = true;
        self.release_pending = false;
        self.held = hold.is_zero();
        self.timer = (!self.held).then(|| {
            cx.spawn(async move |this, cx| {
                cx.background_executor().timer(hold).await;
                this.update(cx, |press, cx| {
                    press.held = true;
                    if press.release_pending {
                        press.release_pending = false;
                        press.pressed = false;
                    }
                    cx.notify();
                })
                .ok();
            })
        });
        cx.notify();
    }

    /// 押下を終える。最低限の時間が経っていなければ、経ってから終える。
    pub fn release(&mut self, cx: &mut Context<Self>) {
        if !self.pressed {
            return;
        }
        if self.held {
            self.pressed = false;
            cx.notify();
        } else {
            self.release_pending = true;
        }
    }

    /// 押して離す（キーボードでの押下）。
    pub fn pulse(&mut self, hold: Duration, cx: &mut Context<Self>) {
        self.press(hold, cx);
        self.release(cx);
    }
}

/// 要素の ID に結びついた押下の状態。
pub(crate) fn press_state(id: &ElementId, window: &mut Window, cx: &mut App) -> Entity<Press> {
    window.use_keyed_state(
        ElementId::NamedChild(std::sync::Arc::new(id.clone()), SharedString::from("press")),
        cx,
        |_, _| Press::default(),
    )
}

/// ポインタの押下と離しを `press` に伝える。
pub(crate) fn track_press<E: InteractiveElement>(element: E, press: &Entity<Press>) -> E {
    let down = press.clone();
    let up = press.clone();
    let up_out = press.clone();
    element
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            let hold = Press::hold(cx);
            down.update(cx, |press, cx| press.press(hold, cx));
        })
        .on_mouse_up(MouseButton::Left, move |_, _, cx| {
            up.update(cx, |press, cx| press.release(cx));
        })
        .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
            up_out.update(cx, |press, cx| press.release(cx));
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;

    #[gpui::test]
    fn short_press_is_held_until_the_shape_has_moved(cx: &mut TestAppContext) {
        let press = cx.new(|_| Press::default());
        let hold = cx.update(|cx| Press::hold(cx));
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
        let hold = cx.update(|cx| Press::hold(cx));
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
        let hold = cx.update(|cx| Press::hold(cx));
        assert_eq!(hold, Duration::ZERO);
        let press = cx.new(|_| Press::default());
        press.update(cx, |press, cx| press.pulse(hold, cx));
        assert!(!press.read_with(cx, |press, _| press.is_pressed()));
    }
}
