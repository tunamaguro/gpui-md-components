//! 押下の状態を追跡する。
use gpui::{App, Context, ElementId, Entity, MouseButton, SharedString, Task, Window, prelude::*};
use std::time::Duration;

/// 押下の状態。
///
/// 短いクリックやキーボードでの押下でも形の変化が見えるように、押下から指定された時間の間は
/// 離しても押下のままにする。
#[derive(Default)]
pub struct Press {
    pressed: bool,
    held: bool,
    release_pending: bool,
    timer: Option<Task<()>>,
}

impl Press {
    /// 押下中か。
    pub fn is_pressed(&self) -> bool {
        self.pressed
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
pub fn press_state(id: &ElementId, window: &mut Window, cx: &mut App) -> Entity<Press> {
    window.use_keyed_state(
        ElementId::NamedChild(std::sync::Arc::new(id.clone()), SharedString::from("press")),
        cx,
        |_, _| Press::default(),
    )
}

/// ポインタの押下と離しを `press` に伝える。
/// `hold` は押下時に呼び、モーションを減らす設定なら `Duration::ZERO` を返す。
pub fn track_press<E: InteractiveElement>(
    element: E,
    press: &Entity<Press>,
    hold: impl Fn(&App) -> Duration + 'static,
) -> E {
    let down = press.clone();
    let up = press.clone();
    let up_out = press.clone();
    element
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            let hold = hold(cx);
            down.update(cx, |press, cx| press.press(hold, cx));
        })
        .on_mouse_up(MouseButton::Left, move |_, _, cx| {
            up.update(cx, |press, cx| press.release(cx));
        })
        .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
            up_out.update(cx, |press, cx| press.release(cx));
        })
}
