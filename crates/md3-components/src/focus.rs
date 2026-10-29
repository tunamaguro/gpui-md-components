//! キーボードフォーカスの表示。

use gpui::{
    App, Corners, ElementId, FocusHandle, IntoElement, Pixels, RenderOnce, Window, div, prelude::*,
    px,
};
use md3_theme::md3_tokens::state;
use md3_theme::{CornerRadius, Md3ThemeExt};

use crate::StyledExt;

/// キーボード操作でフォーカスしているか（フォーカスリングを出すべきか）。
pub fn is_focus_visible(handle: &FocusHandle, window: &Window) -> bool {
    handle.is_focused(window) && window.last_input_was_keyboard()
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
