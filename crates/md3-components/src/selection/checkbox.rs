//! Checkbox。

use gpui::{
    App, ElementId, IntoElement, RenderOnce, Role, SharedString, Toggled, Window, div, prelude::*,
    px,
};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::checkbox as tokens;
use md3_theme::md3_tokens::layout;
use md3_theme::{CornerRadius, Md3ThemeExt};

use crate::a11y::{A11yExt, A11yNode};
use crate::control::control;
use crate::{FocusRing, Icon, IconName, StateLayer, StyledExt};

/// チェックの状態。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CheckboxState {
    /// 未チェック。
    #[default]
    Unchecked,
    /// チェック済み。
    Checked,
    /// 一部だけ選ばれている（子の項目の一部がチェックされている親など）。
    Indeterminate,
}

impl CheckboxState {
    /// 押したときの次の状態。不定はチェック済みになる。
    pub fn next(self) -> Self {
        match self {
            CheckboxState::Checked => CheckboxState::Unchecked,
            CheckboxState::Unchecked | CheckboxState::Indeterminate => CheckboxState::Checked,
        }
    }

    fn is_selected(self) -> bool {
        self != CheckboxState::Unchecked
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
        match (state.is_selected(), error) {
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

type ChangeHandler = Box<dyn Fn(CheckboxState, &mut Window, &mut App) + 'static>;

/// チェックボックス。ラベル（読み上げ用の名前）が必須。見える文字は呼び出し側で隣に置く。
#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    state: CheckboxState,
    label: SharedString,
    error: bool,
    disabled: bool,
    on_change: Option<ChangeHandler>,
}

impl Checkbox {
    /// ID、状態、ラベルから作る。
    pub fn new(
        id: impl Into<ElementId>,
        state: CheckboxState,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            state,
            label: label.into(),
            error: false,
            disabled: false,
            on_change: None,
        }
    }

    /// エラー状態にする。
    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    /// 無効にする。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 押されたときの処理。引数は次の状態（[`CheckboxState::next`]）。
    pub fn on_change(
        mut self,
        handler: impl Fn(CheckboxState, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// アクセシビリティツリーに出す情報。
    pub fn a11y_node(&self) -> A11yNode {
        let toggled = match self.state {
            CheckboxState::Unchecked => Toggled::False,
            CheckboxState::Checked => Toggled::True,
            CheckboxState::Indeterminate => Toggled::Mixed,
        };
        A11yNode::new(Role::CheckBox)
            .label(self.label.clone())
            .toggled(toggled)
            .disabled(self.disabled)
    }
}

impl RenderOnce for Checkbox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let node = self.a11y_node();
        let (base, control) = control(&self.id, self.disabled, true, window, cx);
        let theme = cx.md3();
        let colors = CheckboxColors::of(self.state, self.error);
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
        let icon_color = if self.disabled {
            theme.color(tokens::SELECTED_DISABLED_ICON_COLOR)
        } else {
            theme.color(colors.icon)
        };
        if let Some(container) = colors.container {
            mark = mark.bg(if self.disabled {
                theme
                    .color(tokens::SELECTED_DISABLED_CONTAINER_COLOR)
                    .opacity(tokens::SELECTED_DISABLED_CONTAINER_OPACITY)
            } else {
                theme.color(container)
            });
        }
        if let Some(outline) = colors.outline {
            let color = if self.disabled {
                theme
                    .color(tokens::UNSELECTED_DISABLED_OUTLINE_COLOR)
                    .opacity(tokens::UNSELECTED_DISABLED_CONTAINER_OPACITY)
            } else {
                theme.color(outline)
            };
            mark = mark
                .border(px(tokens::UNSELECTED_OUTLINE_WIDTH))
                .border_color(color);
            if let (false, Some(hovered)) = (self.disabled, colors.hovered_outline) {
                let hovered = theme.color(hovered);
                mark = mark.group_hover(control.group.clone(), move |s| s.border_color(hovered));
            }
        }
        mark = match self.state {
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

        base.a11y(node)
            .size(theme.dense(tokens::STATE_LAYER_SIZE))
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
                .disabled(self.disabled),
            )
            .child(FocusRing::new(radius).visible(control.focus_visible))
            .when_some(
                self.on_change.filter(|_| !self.disabled),
                |this, handler| {
                    let next = self.state.next();
                    this.on_click(move |_, window, cx| handler(next, window, cx))
                },
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_state_cycles() {
        assert_eq!(CheckboxState::Unchecked.next(), CheckboxState::Checked);
        assert_eq!(CheckboxState::Checked.next(), CheckboxState::Unchecked);
        assert_eq!(CheckboxState::Indeterminate.next(), CheckboxState::Checked);
    }

    #[test]
    fn error_uses_error_roles() {
        let colors = CheckboxColors::of(CheckboxState::Checked, true);
        assert_eq!(colors.container, Some(ColorRole::Error));
        assert_eq!(colors.icon, ColorRole::OnError);
    }

    #[test]
    fn indeterminate_is_mixed() {
        let node = Checkbox::new("c", CheckboxState::Indeterminate, "すべて").a11y_node();
        assert_eq!(node.role, Role::CheckBox);
        assert_eq!(node.toggled, Some(Toggled::Mixed));
    }
}
