//! IconButton（Standard / Filled / Filled tonal / Outlined、トグル対応）。

use gpui::{
    App, ClickEvent, ElementId, IntoElement, RenderOnce, Role, SharedString, Window, prelude::*, px,
};
use md3_theme::Md3ThemeExt;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::{
    filled_icon_button, filled_tonal_icon_button, outlined_icon_button, small_icon_button,
    standard_icon_button,
};

use crate::control::control;
use crate::{FocusRing, Icon, IconName, StateLayer, StyledExt};
use gpui_headless::{A11yExt, A11yNode, toggled};

/// アイコンボタンの種類。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum IconButtonVariant {
    /// 背景なし。
    #[default]
    Standard,
    /// 塗りつぶし。
    Filled,
    /// 落ち着いた色の塗りつぶし。
    FilledTonal,
    /// 枠線。
    Outlined,
}

impl IconButtonVariant {
    /// すべての種類。
    pub const ALL: [IconButtonVariant; 4] = [
        IconButtonVariant::Standard,
        IconButtonVariant::Filled,
        IconButtonVariant::FilledTonal,
        IconButtonVariant::Outlined,
    ];
}

/// 種類と選択状態ごとの色（トークンの値）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct IconButtonColors {
    pub container: Option<ColorRole>,
    pub content: ColorRole,
    pub outline: Option<ColorRole>,
}

impl IconButtonColors {
    /// `selected` は トグルでなければ `None`。
    pub(crate) fn of(variant: IconButtonVariant, selected: Option<bool>) -> Self {
        use IconButtonVariant::*;
        match (variant, selected) {
            (Standard, None) => Self::content(standard_icon_button::COLOR),
            (Standard, Some(true)) => Self::content(standard_icon_button::SELECTED_COLOR),
            (Standard, Some(false)) => Self::content(standard_icon_button::UNSELECTED_COLOR),
            (Filled, None) => Self::filled(
                filled_icon_button::CONTAINER_COLOR,
                filled_icon_button::COLOR,
            ),
            (Filled, Some(true)) => Self::filled(
                filled_icon_button::SELECTED_CONTAINER_COLOR,
                filled_icon_button::SELECTED_COLOR,
            ),
            (Filled, Some(false)) => Self::filled(
                filled_icon_button::UNSELECTED_CONTAINER_COLOR,
                filled_icon_button::UNSELECTED_COLOR,
            ),
            (FilledTonal, None) => Self::filled(
                filled_tonal_icon_button::CONTAINER_COLOR,
                filled_tonal_icon_button::COLOR,
            ),
            (FilledTonal, Some(true)) => Self::filled(
                filled_tonal_icon_button::SELECTED_CONTAINER_COLOR,
                filled_tonal_icon_button::SELECTED_COLOR,
            ),
            (FilledTonal, Some(false)) => Self::filled(
                filled_tonal_icon_button::UNSELECTED_CONTAINER_COLOR,
                filled_tonal_icon_button::UNSELECTED_COLOR,
            ),
            (Outlined, None) => Self {
                outline: Some(outlined_icon_button::OUTLINE_COLOR),
                ..Self::content(outlined_icon_button::COLOR)
            },
            (Outlined, Some(true)) => Self::filled(
                outlined_icon_button::SELECTED_CONTAINER_COLOR,
                outlined_icon_button::SELECTED_COLOR,
            ),
            (Outlined, Some(false)) => Self {
                outline: Some(outlined_icon_button::UNSELECTED_OUTLINE_COLOR),
                ..Self::content(outlined_icon_button::UNSELECTED_COLOR)
            },
        }
    }

    fn content(content: ColorRole) -> Self {
        Self {
            container: None,
            content,
            outline: None,
        }
    }

    fn filled(container: ColorRole, content: ColorRole) -> Self {
        Self {
            container: Some(container),
            content,
            outline: None,
        }
    }
}

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// アイコンだけのボタン。アイコンだけでは意味が伝わらないため、ラベル（読み上げ用の名前）が必須。
#[derive(IntoElement)]
pub struct IconButton {
    id: ElementId,
    icon: IconName,
    label: SharedString,
    variant: IconButtonVariant,
    selected: Option<bool>,
    disabled: bool,
    on_click: Option<ClickHandler>,
    content_role: Option<ColorRole>,
    #[cfg_attr(not(feature = "overlays"), allow(dead_code))]
    tooltip: Option<SharedString>,
}

impl IconButton {
    /// ID、アイコン、ラベルから作る。
    pub fn new(id: impl Into<ElementId>, icon: IconName, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            icon,
            label: label.into(),
            variant: IconButtonVariant::default(),
            selected: None,
            disabled: false,
            on_click: None,
            content_role: None,
            tooltip: None,
        }
    }

    /// 種類を変える。
    pub fn variant(mut self, variant: IconButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// トグルボタンにして、選択状態を指定する。
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }

    /// 無効にする。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 押されたときの処理。トグルなら、呼び出し側で選択状態を反転する。
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// ポインタを重ねたときに Plain ツールチップを出す。同じ文を読み上げ用の説明にも入れる。
    #[cfg(feature = "overlays")]
    pub fn tooltip(mut self, text: impl Into<SharedString>) -> Self {
        self.tooltip = Some(text.into());
        self
    }

    /// 内容（ラベル・アイコン・状態レイヤー）の色のロールを変える（Snackbar の操作など）。
    #[cfg_attr(not(feature = "overlays"), allow(dead_code))]
    pub(crate) fn content_color(mut self, role: ColorRole) -> Self {
        self.content_role = Some(role);
        self
    }

    /// アクセシビリティツリーに出す情報。
    pub fn a11y_node(&self) -> A11yNode {
        let node = A11yNode::new(Role::Button)
            .label(self.label.clone())
            .disabled(self.disabled);
        match self.selected {
            Some(selected) => node.toggled(toggled(selected)),
            None => node,
        }
    }
}

impl RenderOnce for IconButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let node = self.a11y_node();
        let (base, control) = control(&self.id, self.disabled, true, window, cx);
        let theme = cx.md3();
        let colors = IconButtonColors::of(self.variant, self.selected);
        let radius = theme.corner(small_icon_button::CONTAINER_SHAPE_ROUND);
        let size = theme.dense(small_icon_button::CONTAINER_HEIGHT);
        let disabled_opacity = standard_icon_button::DISABLED_OPACITY;
        let content = if self.disabled {
            theme
                .color(standard_icon_button::DISABLED_COLOR)
                .opacity(disabled_opacity)
        } else {
            theme.color(self.content_role.unwrap_or(colors.content))
        };

        let mut button = base
            .a11y(node)
            .size(size)
            .flex()
            .items_center()
            .justify_center()
            .corner_radius(radius);
        #[cfg(feature = "overlays")]
        if let Some(text) = self.tooltip.clone() {
            button = crate::overlays::TooltipExt::md3_tooltip(button, text);
        }
        if let Some(container) = colors.container {
            button = if self.disabled {
                button.bg(theme
                    .color(filled_icon_button::DISABLED_CONTAINER_COLOR)
                    .opacity(filled_icon_button::DISABLED_CONTAINER_OPACITY))
            } else {
                button.bg(theme.color(container))
            };
        }
        if let Some(outline) = colors.outline {
            let color = if self.disabled {
                theme.color(outlined_icon_button::DISABLED_OUTLINE_COLOR)
            } else {
                theme.color(outline)
            };
            button = button
                .border(px(small_icon_button::OUTLINED_OUTLINE_WIDTH))
                .border_color(color);
        }

        button
            .child(
                Icon::new(self.icon)
                    .size(px(small_icon_button::ICON_SIZE))
                    .tint(content),
            )
            .child(
                StateLayer::new(
                    SharedString::from(format!("{}-layer", control.group)),
                    control.group.clone(),
                    self.content_role.unwrap_or(colors.content),
                )
                .radius(radius)
                .focused(control.focus_visible)
                .disabled(self.disabled),
            )
            .child(FocusRing::new(radius).visible(control.focus_visible))
            .when_some(self.on_click.filter(|_| !self.disabled), |this, handler| {
                this.on_click(handler)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::Toggled;

    #[test]
    fn toggle_changes_colors() {
        let on = IconButtonColors::of(IconButtonVariant::Filled, Some(true));
        let off = IconButtonColors::of(IconButtonVariant::Filled, Some(false));
        assert_ne!(on, off);
        assert_eq!(on.container, Some(ColorRole::Primary));
    }

    #[test]
    fn toggle_reports_toggled_state() {
        let node = IconButton::new("i", IconName::Favorite, "お気に入り")
            .selected(true)
            .a11y_node();
        assert_eq!(node.role, Role::Button);
        assert_eq!(node.toggled, Some(Toggled::True));
        let plain = IconButton::new("i", IconName::Favorite, "お気に入り").a11y_node();
        assert_eq!(plain.toggled, None);
    }
}
