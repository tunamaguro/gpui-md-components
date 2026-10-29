//! 状態レイヤー（ホバー・フォーカス・押下で重ねる半透明の色）。

use gpui::{
    App, Corners, ElementId, IntoElement, Pixels, RenderOnce, SharedString, Window, div, prelude::*,
};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::state;
use md3_theme::{CornerRadius, Md3ThemeExt};

use crate::{ForcedState, StyledExt};

/// 操作要素に重ねる状態レイヤー。
///
/// 親要素に `.group(group)` と `.relative()` を付け、その子として最後に置く。
/// ホバー・押下は親のグループの状態から、フォーカスは `focused` から決まる。
/// 不透明度は MD3 の state layer トークン（ホバー 8%、フォーカス 10%、押下 10%）。
///
/// 状態は重ねず、押下 > フォーカス > ホバー の順に 1 つだけを出す。
#[derive(IntoElement)]
pub struct StateLayer {
    id: ElementId,
    group: SharedString,
    color: ColorRole,
    radius: CornerRadius,
    corners: Option<Corners<Pixels>>,
    focused: bool,
    disabled: bool,
    forced: Option<ForcedState>,
}

impl StateLayer {
    /// 状態レイヤーを作る。`group` は親要素に付けたグループ名、`color` は前景（内容）のロール。
    pub fn new(id: impl Into<ElementId>, group: impl Into<SharedString>, color: ColorRole) -> Self {
        Self {
            id: id.into(),
            group: group.into(),
            color,
            radius: CornerRadius::Fixed(gpui::px(0.0)),
            corners: None,
            focused: false,
            disabled: false,
            forced: None,
        }
    }

    /// 親要素と同じ角丸にする。
    pub fn radius(mut self, radius: CornerRadius) -> Self {
        self.radius = radius;
        self
    }

    /// 角ごとに違う角丸にする（[`StateLayer::radius`] より優先）。
    pub fn corners(mut self, corners: Corners<Pixels>) -> Self {
        self.corners = Some(corners);
        self
    }

    /// フォーカスがあるか（キーボード操作でフォーカスしたときだけ `true` にする）。
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// 見本用に状態を固定する。
    #[doc(hidden)]
    pub fn forced(mut self, forced: Option<ForcedState>) -> Self {
        self.forced = forced;
        self
    }

    /// 無効状態か。無効なら何も重ねない。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl RenderOnce for StateLayer {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let color = cx.md3().color(self.color);
        let layer = div()
            .id(self.id)
            .absolute()
            .inset_0()
            .corner_radius(self.radius)
            .when_some(self.corners, |this, corners| this.corners(corners));
        if self.disabled {
            return layer;
        }
        if let Some(forced) = self.forced {
            let opacity = match forced {
                ForcedState::Hovered => state::HOVER_STATE_LAYER_OPACITY,
                ForcedState::Focused => state::FOCUS_STATE_LAYER_OPACITY,
                ForcedState::Pressed => state::PRESSED_STATE_LAYER_OPACITY,
            };
            return layer.bg(color.opacity(opacity));
        }
        let layer = if self.focused {
            layer.bg(color.opacity(state::FOCUS_STATE_LAYER_OPACITY))
        } else {
            layer.group_hover(self.group.clone(), |style| {
                style.bg(color.opacity(state::HOVER_STATE_LAYER_OPACITY))
            })
        };
        layer.group_active(self.group, |style| {
            style.bg(color.opacity(state::PRESSED_STATE_LAYER_OPACITY))
        })
    }
}
