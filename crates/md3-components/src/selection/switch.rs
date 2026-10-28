//! Switch。

use gpui::{
    App, ElementId, IntoElement, RenderOnce, Role, SharedString, Window, div, prelude::*, px,
};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::switch as tokens;
use md3_theme::{CornerRadius, Md3ThemeExt};

use crate::a11y::{A11yExt, A11yNode, toggled};
use crate::control::control;
use crate::{FocusRing, ForcedState, Icon, IconName, StateLayer, StyledExt};

/// 選択状態ごとの色（トークンの値）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SwitchColors {
    pub track: ColorRole,
    pub track_outline: Option<ColorRole>,
    pub handle: ColorRole,
    pub hovered_handle: ColorRole,
    pub icon: ColorRole,
}

impl SwitchColors {
    pub(crate) fn of(selected: bool) -> Self {
        if selected {
            Self {
                track: tokens::SELECTED_TRACK_COLOR,
                track_outline: None,
                handle: tokens::SELECTED_HANDLE_COLOR,
                hovered_handle: tokens::SELECTED_HOVER_HANDLE_COLOR,
                icon: tokens::SELECTED_ICON_COLOR,
            }
        } else {
            Self {
                track: tokens::UNSELECTED_TRACK_COLOR,
                track_outline: Some(tokens::UNSELECTED_TRACK_OUTLINE_COLOR),
                handle: tokens::UNSELECTED_HANDLE_COLOR,
                hovered_handle: tokens::UNSELECTED_HOVER_HANDLE_COLOR,
                icon: tokens::UNSELECTED_ICON_COLOR,
            }
        }
    }
}

type ChangeHandler = Box<dyn Fn(bool, &mut Window, &mut App) + 'static>;

/// スイッチ。ラベル（読み上げ用の名前）が必須。
#[derive(IntoElement)]
pub struct Switch {
    id: ElementId,
    selected: bool,
    label: SharedString,
    icons: bool,
    disabled: bool,
    on_change: Option<ChangeHandler>,
    forced: Option<ForcedState>,
}

impl Switch {
    /// ID、状態、ラベルから作る。
    pub fn new(id: impl Into<ElementId>, selected: bool, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            selected,
            label: label.into(),
            icons: false,
            disabled: false,
            on_change: None,
            forced: None,
        }
    }

    /// つまみにアイコン（オンはチェック、オフはバツ）を出す。
    pub fn icons(mut self, icons: bool) -> Self {
        self.icons = icons;
        self
    }

    /// 無効にする。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 切り替えたときの処理。引数は次の状態。
    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// 見本用に状態を固定する。
    #[doc(hidden)]
    pub fn forced(mut self, forced: ForcedState) -> Self {
        self.forced = Some(forced);
        self
    }

    /// アクセシビリティツリーに出す情報。
    pub fn a11y_node(&self) -> A11yNode {
        A11yNode::new(Role::Switch)
            .label(self.label.clone())
            .toggled(toggled(self.selected))
            .disabled(self.disabled)
    }
}

impl RenderOnce for Switch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let node = self.a11y_node();
        let (base, control) = control(&self.id, self.disabled, true, self.forced, window, cx);
        let theme = cx.md3();
        let colors = SwitchColors::of(self.selected);
        let track_radius = theme.corner(tokens::TRACK_SHAPE);
        let disabled = self.disabled;

        let track_color = if disabled {
            let role = if self.selected {
                tokens::DISABLED_SELECTED_TRACK_COLOR
            } else {
                tokens::DISABLED_UNSELECTED_TRACK_COLOR
            };
            theme.color(role).opacity(tokens::DISABLED_TRACK_OPACITY)
        } else {
            theme.color(colors.track)
        };
        let handle_color = match (disabled, self.selected) {
            (true, true) => theme
                .color(tokens::DISABLED_SELECTED_HANDLE_COLOR)
                .opacity(tokens::DISABLED_SELECTED_HANDLE_OPACITY),
            (true, false) => theme
                .color(tokens::DISABLED_UNSELECTED_HANDLE_COLOR)
                .opacity(tokens::DISABLED_UNSELECTED_HANDLE_OPACITY),
            (false, _) => theme.color(colors.handle),
        };
        let handle_size = if self.selected || self.icons {
            tokens::SELECTED_HANDLE_WIDTH
        } else {
            tokens::UNSELECTED_HANDLE_WIDTH
        };
        let pressed_size = px(tokens::PRESSED_HANDLE_WIDTH);
        let hovered_handle = theme.color(colors.hovered_handle);

        let mut handle = div()
            .id("handle")
            .size(px(handle_size))
            .rounded_full()
            .bg(handle_color)
            .flex()
            .items_center()
            .justify_center();
        if !disabled {
            handle = match self.forced {
                Some(ForcedState::Pressed) => handle.size(pressed_size).bg(hovered_handle),
                Some(ForcedState::Hovered | ForcedState::Focused) => handle.bg(hovered_handle),
                None => handle
                    .group_hover(control.group.clone(), move |s| s.bg(hovered_handle))
                    .group_active(control.group.clone(), move |s| s.size(pressed_size)),
            };
        }
        if self.icons {
            let (icon, size) = if self.selected {
                (IconName::Check, tokens::SELECTED_ICON_SIZE)
            } else {
                (IconName::Close, tokens::UNSELECTED_ICON_SIZE)
            };
            let color = if disabled {
                let role = if self.selected {
                    tokens::DISABLED_SELECTED_ICON_COLOR
                } else {
                    tokens::DISABLED_UNSELECTED_ICON_COLOR
                };
                let opacity = if self.selected {
                    tokens::DISABLED_SELECTED_ICON_OPACITY
                } else {
                    tokens::DISABLED_UNSELECTED_ICON_OPACITY
                };
                theme.color(role).opacity(opacity)
            } else {
                theme.color(colors.icon)
            };
            handle = handle.child(Icon::new(icon).size(px(size)).tint(color));
        }

        // つまみは track の高さの正方形の中央に置き、その正方形を左右の端に寄せる。
        let slot_size = px(tokens::TRACK_HEIGHT);
        let layer_size = px(tokens::STATE_LAYER_SIZE);
        let layer_offset = (slot_size - layer_size) / 2.0;
        let layer_radius = CornerRadius::Full;
        let slot = div()
            .relative()
            .size(slot_size)
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .absolute()
                    .top(layer_offset)
                    .left(layer_offset)
                    .size(layer_size)
                    .child(
                        StateLayer::new(
                            SharedString::from(format!("{}-layer", control.group)),
                            control.group.clone(),
                            colors.hovered_handle,
                        )
                        .radius(layer_radius)
                        .focused(control.focus_visible)
                        .disabled(disabled)
                        .forced(self.forced),
                    ),
            )
            .child(handle);

        let mut track = div()
            .size_full()
            .flex()
            .items_center()
            .when(self.selected, |this| this.justify_end())
            .corner_radius(track_radius)
            .bg(track_color);
        if let Some(outline) = colors.track_outline {
            let color = if disabled {
                theme
                    .color(tokens::DISABLED_UNSELECTED_TRACK_OUTLINE_COLOR)
                    .opacity(tokens::DISABLED_TRACK_OPACITY)
            } else {
                theme.color(outline)
            };
            track = track
                .border(px(tokens::TRACK_OUTLINE_WIDTH))
                .border_color(color);
        }

        base.a11y(node)
            .w(px(tokens::TRACK_WIDTH))
            .h(px(tokens::TRACK_HEIGHT))
            .child(track.child(slot))
            .child(FocusRing::new(track_radius).visible(control.focus_visible))
            .when_some(self.on_change.filter(|_| !disabled), |this, handler| {
                let next = !self.selected;
                this.on_click(move |_, window, cx| handler(next, window, cx))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::Toggled;

    #[test]
    fn selected_track_is_primary() {
        assert_eq!(SwitchColors::of(true).track, ColorRole::Primary);
        assert!(SwitchColors::of(false).track_outline.is_some());
    }

    #[test]
    fn switch_reports_toggled() {
        let node = Switch::new("s", true, "Wi-Fi").a11y_node();
        assert_eq!(node.role, Role::Switch);
        assert_eq!(node.toggled, Some(Toggled::True));
    }
}
