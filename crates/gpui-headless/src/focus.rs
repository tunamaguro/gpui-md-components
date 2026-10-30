//! キーボードフォーカスの表示。

use gpui::{App, ElementId, FocusHandle, Global, WeakFocusHandle, Window, WindowId};
use std::collections::HashMap;

/// ウィンドウごとのフォーカス表示の保持。
#[derive(Default)]
// gpui は FocusHandle の FocusId を公開しないため、寿命を延ばさない弱いハンドルで識別する。
pub struct VisibleFocus(HashMap<WindowId, WeakFocusHandle>);

impl Global for VisibleFocus {}

/// キーボードで得たフォーカスの間、マウス移動後もリングを表示するか。
///
/// フォーカスを失うと表示を解除する。共通の操作要素と Card はクリックでも解除する。
pub fn is_focus_visible(handle: &FocusHandle, window: &Window, cx: &mut App) -> bool {
    if !cx.has_global::<VisibleFocus>() {
        cx.set_global(VisibleFocus::default());
    }
    let window_id = window.window_handle().window_id();
    let latched = cx.global_mut::<VisibleFocus>();
    let matches = latched
        .0
        .get(&window_id)
        .is_some_and(|focus| focus == handle);
    if !handle.is_focused(window) {
        if matches {
            latched.0.remove(&window_id);
        }
        false
    } else if matches {
        true
    } else if window.last_input_was_keyboard() {
        latched.0.insert(window_id, handle.downgrade());
        true
    } else {
        false
    }
}

/// 保持したフォーカス表示を解除する。
pub fn clear_focus_visible(handle: &FocusHandle, window: &mut Window, cx: &mut App) {
    if cx.has_global::<VisibleFocus>() {
        let latched = cx.global_mut::<VisibleFocus>();
        let id = window.window_handle().window_id();
        if latched.0.get(&id).is_some_and(|focus| focus == handle) {
            latched.0.remove(&id);
            window.refresh();
        }
    }
}

/// 要素の ID に結び付いたフォーカスハンドル。描画をまたいで同じものを返す。
///
/// Tab で移動できる（tab stop）。`RenderOnce` の部品がフォーカスを持つときに使う。
pub fn focus_handle(id: impl Into<ElementId>, window: &mut Window, cx: &mut App) -> FocusHandle {
    window
        .use_keyed_state(id, cx, |_, cx| cx.focus_handle().tab_stop(true))
        .read(cx)
        .clone()
}
