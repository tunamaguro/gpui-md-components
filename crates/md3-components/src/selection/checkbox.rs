//! Checkbox。

use gpui::{App, ElementId, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, px};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::checkbox as tokens;
use md3_theme::md3_tokens::layout;
use md3_theme::{CornerRadius, Md3ThemeExt};

use crate::{FocusRing, Icon, IconName, StateLayer, StyledExt};

/// チェックの状態。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CheckboxState {
    /// 未チェック。
    #[default]
    Unchecked,
    /// チェック済み。
    Checked,
    /// 一部だけ選ばれている。
    Indeterminate,
}

impl CheckboxState {
    /// 押したときの次の状態。不定はチェック済みになる。
    pub fn next(self) -> Self {
        gpui_headless::CheckboxState::from(self).next().into()
    }
}

impl From<CheckboxState> for gpui_headless::CheckboxState {
    fn from(state: CheckboxState) -> Self {
        match state {
            CheckboxState::Unchecked => Self::Unchecked,
            CheckboxState::Checked => Self::Checked,
            CheckboxState::Indeterminate => Self::Indeterminate,
        }
    }
}

impl From<gpui_headless::CheckboxState> for CheckboxState {
    fn from(state: gpui_headless::CheckboxState) -> Self {
        match state {
            gpui_headless::CheckboxState::Unchecked => Self::Unchecked,
            gpui_headless::CheckboxState::Checked => Self::Checked,
            gpui_headless::CheckboxState::Indeterminate => Self::Indeterminate,
        }
    }
}

/// 状態ごとの色（トークンの値）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CheckboxColors {
    pub container: Option<ColorRole>,
    pub outline: Option<ColorRole>,
    pub hovered_outline: Option<ColorRole>,
    pub icon: ColorRole,
    pub state_layer: ColorRole,
}

impl CheckboxColors {
    pub(crate) fn of(state: CheckboxState, error: bool) -> Self {
        match (state != CheckboxState::Unchecked, error) {
            (true, false) => Self {
                container: Some(tokens::SELECTED_CONTAINER_COLOR),
                outline: None,
                hovered_outline: None,
                icon: tokens::SELECTED_ICON_COLOR,
                state_layer: tokens::SELECTED_HOVER_CONTAINER_COLOR,
            },
            (true, true) => Self {
                container: Some(tokens::SELECTED_ERROR_CONTAINER_COLOR),
                outline: None,
                hovered_outline: None,
                icon: tokens::SELECTED_ERROR_ICON_COLOR,
                state_layer: tokens::SELECTED_ERROR_HOVER_CONTAINER_COLOR,
            },
            (false, false) => Self {
                container: None,
                outline: Some(tokens::UNSELECTED_OUTLINE_COLOR),
                hovered_outline: Some(tokens::UNSELECTED_HOVER_OUTLINE_COLOR),
                icon: tokens::SELECTED_ICON_COLOR,
                state_layer: tokens::UNSELECTED_HOVER_OUTLINE_COLOR,
            },
            (false, true) => Self {
                container: None,
                outline: Some(tokens::UNSELECTED_ERROR_OUTLINE_COLOR),
                hovered_outline: Some(tokens::UNSELECTED_ERROR_HOVER_OUTLINE_COLOR),
                icon: tokens::SELECTED_ERROR_ICON_COLOR,
                state_layer: tokens::UNSELECTED_ERROR_HOVER_OUTLINE_COLOR,
            },
        }
    }
}

/// チェックボックス。見える文字は呼び出し側で隣に置く。
#[derive(IntoElement)]
pub struct Checkbox {
    inner: gpui_headless::Checkbox,
    error: bool,
}

impl Checkbox {
    /// ID と状態から作る。
    pub fn new(id: impl Into<ElementId>, state: CheckboxState) -> Self {
        let id = id.into();
        Self {
            inner: gpui_headless::Checkbox::new(id.clone(), state.into())
                .group(format!("md3-{id}"))
                .reveal_margin(crate::focus::reveal_margin()),
            error: false,
        }
    }

    delegate_accessible_name!(inner);

    /// エラー状態にする。
    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    /// 無効にする。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.inner = self.inner.disabled(disabled);
        self
    }

    /// 押されたときの処理。引数は次の状態（[`CheckboxState::next`]）。
    pub fn on_change(
        mut self,
        handler: impl Fn(CheckboxState, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.inner = self
            .inner
            .on_change(move |state, window, cx| handler(state.into(), window, cx));
        self
    }
}

impl RenderOnce for Checkbox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let checked_state = CheckboxState::from(self.inner.state());
        let (base, control) = self.inner.build(window, cx);
        let theme = cx.md3();
        let colors = CheckboxColors::of(checked_state, self.error);
        let radius = theme.corner(tokens::STATE_LAYER_SHAPE);
        let box_radius = match tokens::CONTAINER_SHAPE {
            md3_theme::md3_tokens::shape::CornerSize::Fixed(r) => CornerRadius::Fixed(px(r)),
            md3_theme::md3_tokens::shape::CornerSize::Full => CornerRadius::Full,
        };
        let size = px(tokens::CONTAINER_SIZE);

        let mut mark = div()
            .size(size)
            .flex()
            .items_center()
            .justify_center()
            .corner_radius(box_radius);
        let icon_color = if control.disabled {
            theme.color(tokens::SELECTED_DISABLED_ICON_COLOR)
        } else {
            theme.color(colors.icon)
        };
        if let Some(container) = colors.container {
            mark = mark.bg(if control.disabled {
                theme
                    .color(tokens::SELECTED_DISABLED_CONTAINER_COLOR)
                    .opacity(tokens::SELECTED_DISABLED_CONTAINER_OPACITY)
            } else {
                theme.color(container)
            });
        }
        if let Some(outline) = colors.outline {
            let color = if control.disabled {
                theme
                    .color(tokens::UNSELECTED_DISABLED_OUTLINE_COLOR)
                    .opacity(tokens::UNSELECTED_DISABLED_CONTAINER_OPACITY)
            } else {
                theme.color(outline)
            };
            mark = mark
                .border(px(tokens::UNSELECTED_OUTLINE_WIDTH))
                .border_color(color);
            if let (false, Some(hovered)) = (control.disabled, colors.hovered_outline) {
                let hovered = theme.color(hovered);
                mark = mark.group_hover(control.group.clone(), move |s| s.border_color(hovered));
            }
        }
        mark = match checked_state {
            CheckboxState::Checked => mark.child(
                Icon::new(IconName::Check)
                    .size(px(tokens::ICON_SIZE))
                    .tint(icon_color),
            ),
            // 不定の横線。長さは Compose の描画（チェックボックスの幅の 0.4〜0.6 倍を結ぶ線）とは違う簡略形。
            CheckboxState::Indeterminate => mark.child(
                div()
                    .w(size * 0.6)
                    .h(px(layout::CHECKBOX_STROKE_WIDTH))
                    .bg(icon_color),
            ),
            CheckboxState::Unchecked => mark,
        };

        base.size(theme.dense(tokens::STATE_LAYER_SIZE))
            .flex()
            .items_center()
            .justify_center()
            .corner_radius(radius)
            .child(mark)
            .child(
                StateLayer::new(
                    SharedString::from(format!("{}-layer", control.group)),
                    control.group.clone(),
                    colors.state_layer,
                )
                .radius(radius)
                .focused(control.focus_visible)
                .disabled(control.disabled),
            )
            .child(FocusRing::new(radius).visible(control.focus_visible))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_uses_error_roles() {
        let colors = CheckboxColors::of(CheckboxState::Checked, true);
        assert_eq!(colors.container, Some(ColorRole::Error));
        assert_eq!(colors.icon, ColorRole::OnError);
    }
}
