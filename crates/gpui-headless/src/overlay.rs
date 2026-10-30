//! オーバーレイの共通部分（ウィンドウ全体への重ね描き、幕、フォーカスの閉じ込めと復帰）。

use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    AnchoredPositionMode, App, Deferred, Div, ElementId, Entity, FocusHandle, Hsla, MouseButton,
    Stateful, Window, anchored, deferred, div, point, prelude::*, px,
};

use crate::keys::{Dismiss, FocusNextWithin, FocusPrevWithin, MODAL_CONTEXT};

/// 閉じるときの処理。
pub type DismissHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// 重ね描きの優先度。メニューはダイアログより手前に出す。
/// モーダルの重ね描きの優先度。
pub const MODAL_PRIORITY: usize = 1;
/// メニューの重ね描きの優先度。
pub const MENU_PRIORITY: usize = 2;
/// 通知の重ね描きの優先度。
pub const SNACKBAR_PRIORITY: usize = 3;

/// 開いたときに最初にフォーカスする先。
pub enum InitialFocus {
    /// オーバーレイの中の最初の要素（なければオーバーレイ自体）。
    Within,
    /// 指定した要素。
    Handle(FocusHandle),
}

/// オーバーレイのフォーカスの状態。開いている間だけ存在する。
pub struct OverlayFocus {
    /// オーバーレイ全体のフォーカスハンドル（Tab では止まらない）。
    pub handle: FocusHandle,
    /// 開く前にフォーカスしていた要素（閉じたら戻す）。
    opener: Option<FocusHandle>,
}

/// 開く前にフォーカスしていた要素へ戻す。
pub fn restore_focus(focus: &Entity<OverlayFocus>, window: &mut Window, cx: &mut App) {
    if let Some(opener) = focus.read(cx).opener.clone() {
        window.focus(&opener, cx);
    }
}

/// オーバーレイのフォーカスの状態を取り出す。最初に描かれたときに、その時点のフォーカスを覚え、
/// 次のフレームで `initial` にフォーカスを移す。
pub fn overlay_focus(
    id: &ElementId,
    initial: impl FnOnce() -> InitialFocus,
    window: &mut Window,
    cx: &mut App,
) -> Entity<OverlayFocus> {
    let key = ElementId::NamedChild(Arc::new(id.clone()), "overlay-focus".into());
    window.use_keyed_state(key, cx, move |window, cx| {
        let handle = cx.focus_handle().tab_stop(false);
        let opener = window.focused(cx);
        let scope = handle.clone();
        let initial = initial();
        window.on_next_frame(move |window, cx| match initial {
            InitialFocus::Handle(target) => window.focus(&target, cx),
            InitialFocus::Within => {
                window.focus(&scope, cx);
                cycle_focus(&scope, true, window, cx);
            }
        });
        OverlayFocus { handle, opener }
    })
}

/// フォーカスを `scope` の中で次（前）の要素へ移す。外に出たら中に戻るまで同じ向きに進める。
///
/// gpui の Tab 順はウィンドウ全体で 1 本なので、`scope` の中の要素はその中で連続している。
/// 末尾から進むと先頭に、先頭から戻ると末尾に回る。中に要素がなければ `scope` 自体にフォーカスする。
pub fn cycle_focus(scope: &FocusHandle, forward: bool, window: &mut Window, cx: &mut App) {
    // 無限に回らないための上限（gpui は Tab で止まる要素の数を公開していない）。
    const MAX_STEPS: usize = 4096;
    let start = window.focused(cx);
    for _ in 0..MAX_STEPS {
        if forward {
            window.focus_next(cx);
        } else {
            window.focus_prev(cx);
        }
        let focused = window.focused(cx);
        if scope.contains_focused(window, cx) && !scope.is_focused(window) {
            return;
        }
        if focused.is_none() || focused == start {
            break;
        }
    }
    window.focus(scope, cx);
}

/// ウィンドウ全体を覆う層に `child` を置く。`child` はウィンドウの大きさになる。
pub fn window_layer(child: impl IntoElement, priority: usize, window: &Window) -> Deferred {
    let viewport = window.viewport_size();
    deferred(
        anchored()
            .position_mode(AnchoredPositionMode::Window)
            .position(point(px(0.0), px(0.0)))
            .child(div().w(viewport.width).h(viewport.height).child(child)),
    )
    .with_priority(priority)
}

/// モーダルの後ろを覆う幕。押すと `dismiss` を呼ぶ。後ろの要素への操作は遮る。
pub fn scrim(id: ElementId, color: Hsla, dismiss: Option<DismissHandler>) -> Stateful<Div> {
    div()
        .id(id)
        .size_full()
        .occlude()
        .bg(color)
        .when_some(dismiss, |this, dismiss| {
            this.on_mouse_down(MouseButton::Left, move |_, window, cx| dismiss(window, cx))
        })
}

/// モーダルな入れ物の共通部分（Esc で閉じる、Tab を中に閉じ込める、押下をスクリムに渡さない）。
pub fn modal_container(
    base: Stateful<Div>,
    focus: &Entity<OverlayFocus>,
    dismiss: Option<DismissHandler>,
    cx: &App,
) -> Stateful<Div> {
    let handle = focus.read(cx).handle.clone();
    let next = handle.clone();
    let prev = handle.clone();
    base.key_context(MODAL_CONTEXT)
        .track_focus(&handle)
        .occlude()
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_action(move |_: &FocusNextWithin, window, cx| cycle_focus(&next, true, window, cx))
        .on_action(move |_: &FocusPrevWithin, window, cx| cycle_focus(&prev, false, window, cx))
        .when_some(dismiss, |this, dismiss| {
            this.on_action(move |_: &Dismiss, window, cx| dismiss(window, cx))
        })
}

/// 呼び出し側の閉じる処理に、フォーカスを開く前の要素へ戻す処理を足す。
pub fn dismiss_and_restore(
    on_dismiss: Option<DismissHandler>,
    focus: &Entity<OverlayFocus>,
) -> Option<DismissHandler> {
    let on_dismiss = on_dismiss?;
    let focus = focus.clone();
    Some(Rc::new(move |window: &mut Window, cx: &mut App| {
        on_dismiss(window, cx);
        restore_focus(&focus, window, cx);
    }))
}
