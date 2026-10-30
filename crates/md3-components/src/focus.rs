//! キーボードフォーカスの表示。

use gpui::{
    App, Corners, ElementId, FocusHandle, Global, IntoElement, Pixels, RenderOnce, WeakFocusHandle,
    Window, WindowId, div, prelude::*, px,
};
use md3_theme::md3_tokens::state;
use md3_theme::{CornerRadius, Md3ThemeExt};
use std::collections::HashMap;

use crate::StyledExt;

#[derive(Default)]
// gpui は FocusHandle の FocusId を公開しないため、寿命を延ばさない弱いハンドルで識別する。
struct VisibleFocus(HashMap<WindowId, WeakFocusHandle>);

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

pub(crate) fn clear_focus_visible(handle: &FocusHandle, window: &mut Window, cx: &mut App) {
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

/// フォーカスリング。MD3 の focus indicator（太さ 3dp、外側に 2dp 離す、`Secondary`）。
///
/// 親要素に `.relative()` を付け、その子として置く。`visible` が `false` なら何も描かない。
#[derive(IntoElement)]
pub struct FocusRing {
    radius: CornerRadius,
    corners: Option<Corners<Pixels>>,
    visible: bool,
    inward: bool,
}

impl FocusRing {
    /// 親要素の角丸を指定して作る。
    pub fn new(radius: CornerRadius) -> Self {
        Self {
            radius,
            corners: None,
            visible: true,
            inward: false,
        }
    }

    /// 親要素が角ごとに違う角丸のときに使う（[`FocusRing::new`] の角丸より優先）。
    pub fn corners(mut self, corners: Corners<Pixels>) -> Self {
        self.corners = Some(corners);
        self
    }

    /// 要素の内側に描く（リストやメニューの項目のように、外側が切り取られる要素で使う）。
    pub fn inward(mut self) -> Self {
        self.inward = true;
        self
    }

    /// 表示するか。通常は [`is_focus_visible`] の結果を渡す。
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}

impl RenderOnce for FocusRing {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let thickness = px(state::FOCUS_INDICATOR_THICKNESS);
        let outset: Pixels = if self.inward {
            -px(state::FOCUS_INDICATOR_INWARD_OFFSET)
        } else {
            thickness + px(state::FOCUS_INDICATOR_OUTER_OFFSET)
        };
        let ring = div()
            .absolute()
            .top(-outset)
            .left(-outset)
            .right(-outset)
            .bottom(-outset)
            .corner_radius(self.radius.grow(outset))
            .when_some(self.corners, |this, corners| {
                this.corners(corners.map(|radius| (*radius + outset).max(px(0.0))))
            });
        if self.visible {
            ring.border(thickness)
                .border_color(cx.md3().color(state::FOCUS_INDICATOR_COLOR))
        } else {
            ring
        }
    }
}
