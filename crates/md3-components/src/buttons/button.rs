//! Button（Elevated / Filled / Filled tonal / Outlined / Text）。

use gpui::{
    App, ClickEvent, ElementId, IntoElement, RenderOnce, SharedString, Window, prelude::*, px,
};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::{
    baseline_button, elevated_button, filled_button, filled_tonal_button, outlined_button,
    text_button,
};
use md3_theme::md3_tokens::layout;
use md3_theme::{Elevation, Md3ThemeExt};

use crate::{FocusRing, Icon, IconName, StateLayer, StyledExt};

/// ボタンの種類。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ButtonVariant {
    /// 影のあるボタン。
    Elevated,
    /// 塗りつぶしのボタン（最も強い強調）。
    #[default]
    Filled,
    /// 落ち着いた色の塗りつぶしのボタン。
    FilledTonal,
    /// 枠線のボタン。
    Outlined,
    /// 文字だけのボタン（最も弱い強調）。
    Text,
}

impl ButtonVariant {
    /// すべての種類。
    pub const ALL: [ButtonVariant; 5] = [
        ButtonVariant::Elevated,
        ButtonVariant::Filled,
        ButtonVariant::FilledTonal,
        ButtonVariant::Outlined,
        ButtonVariant::Text,
    ];
}

/// 種類ごとの見た目（トークンの値）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ButtonStyle {
    pub container: Option<ColorRole>,
    pub content: ColorRole,
    pub outline: Option<ColorRole>,
    pub elevation: f32,
    pub hovered_elevation: f32,
    pub disabled_container_opacity: f32,
    pub disabled_content: ColorRole,
    pub disabled_content_opacity: f32,
    pub leading_space: f32,
    pub leading_space_with_icon: f32,
    pub trailing_space: f32,
}

impl ButtonStyle {
    pub(crate) fn of(variant: ButtonVariant) -> Self {
        let filled_spacing = |style: Self| Self {
            leading_space: baseline_button::LEADING_SPACE,
            leading_space_with_icon: layout::BUTTON_WITH_ICON_LEADING_SPACE,
            trailing_space: baseline_button::TRAILING_SPACE,
            ..style
        };
        match variant {
            ButtonVariant::Elevated => filled_spacing(Self {
                container: Some(elevated_button::CONTAINER_COLOR),
                content: elevated_button::LABEL_TEXT_COLOR,
                outline: None,
                elevation: elevated_button::CONTAINER_ELEVATION,
                hovered_elevation: elevated_button::HOVERED_CONTAINER_ELEVATION,
                disabled_container_opacity: elevated_button::DISABLED_CONTAINER_OPACITY,
                disabled_content: elevated_button::DISABLED_LABEL_TEXT_COLOR,
                disabled_content_opacity: elevated_button::DISABLED_LABEL_TEXT_OPACITY,
                ..Self::EMPTY
            }),
            ButtonVariant::Filled => filled_spacing(Self {
                container: Some(filled_button::CONTAINER_COLOR),
                content: filled_button::LABEL_TEXT_COLOR,
                outline: None,
                elevation: filled_button::CONTAINER_ELEVATION,
                hovered_elevation: filled_button::HOVERED_CONTAINER_ELEVATION,
                disabled_container_opacity: filled_button::DISABLED_CONTAINER_OPACITY,
                disabled_content: filled_button::DISABLED_LABEL_TEXT_COLOR,
                disabled_content_opacity: filled_button::DISABLED_LABEL_TEXT_OPACITY,
                ..Self::EMPTY
            }),
            ButtonVariant::FilledTonal => filled_spacing(Self {
                container: Some(filled_tonal_button::CONTAINER_COLOR),
                content: filled_tonal_button::LABEL_TEXT_COLOR,
                outline: None,
                elevation: filled_tonal_button::CONTAINER_ELEVATION,
                hovered_elevation: filled_tonal_button::HOVER_CONTAINER_ELEVATION,
                disabled_container_opacity: filled_tonal_button::DISABLED_CONTAINER_OPACITY,
                disabled_content: filled_tonal_button::DISABLED_LABEL_TEXT_COLOR,
                disabled_content_opacity: filled_tonal_button::DISABLED_LABEL_TEXT_OPACITY,
                ..Self::EMPTY
            }),
            ButtonVariant::Outlined => filled_spacing(Self {
                container: None,
                content: outlined_button::LABEL_TEXT_COLOR,
                outline: Some(outlined_button::OUTLINE_COLOR),
                elevation: 0.0,
                hovered_elevation: 0.0,
                disabled_container_opacity: outlined_button::DISABLED_CONTAINER_OPACITY,
                disabled_content: outlined_button::DISABLED_LABEL_TEXT_COLOR,
                disabled_content_opacity: outlined_button::DISABLED_LABEL_TEXT_OPACITY,
                ..Self::EMPTY
            }),
            // Compose（Button.kt の defaultTextButtonColors）は、トークンの LabelColor
            // （OnSurfaceVariant）が誤りだとして Primary を使っている。それに合わせる。
            ButtonVariant::Text => Self {
                container: None,
                content: ColorRole::Primary,
                outline: None,
                elevation: 0.0,
                hovered_elevation: 0.0,
                disabled_container_opacity: 0.0,
                disabled_content: text_button::DISABLED_LABEL_COLOR,
                disabled_content_opacity: text_button::DISABLED_LABEL_OPACITY,
                leading_space: layout::TEXT_BUTTON_HORIZONTAL_SPACE,
                leading_space_with_icon: layout::TEXT_BUTTON_HORIZONTAL_SPACE,
                trailing_space: layout::TEXT_BUTTON_WITH_ICON_TRAILING_SPACE,
            },
        }
    }

    const EMPTY: Self = Self {
        container: None,
        content: ColorRole::Primary,
        outline: None,
        elevation: 0.0,
        hovered_elevation: 0.0,
        disabled_container_opacity: 0.0,
        disabled_content: ColorRole::OnSurface,
        disabled_content_opacity: 0.0,
        leading_space: 0.0,
        leading_space_with_icon: 0.0,
        trailing_space: 0.0,
    };
}

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// ボタン。
///
/// ```ignore
/// Button::new("save", "保存").variant(ButtonVariant::Filled).on_click(|_, _, cx| { … })
/// ```
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: SharedString,
    aria_label: Option<Option<SharedString>>,
    variant: ButtonVariant,
    icon: Option<IconName>,
    disabled: bool,
    on_click: Option<ClickHandler>,
    content_role: Option<ColorRole>,
}

impl Button {
    /// ID とラベルから作る。
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            aria_label: None,
            variant: ButtonVariant::default(),
            icon: None,
            disabled: false,
            on_click: None,
            content_role: None,
        }
    }

    /// 読み上げる名前を指定する。表示文字を優先し、表示ラベルがある場合は名前に含める（WCAG 2.5.3）。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        self.aria_label = Some((!label.is_empty()).then_some(label));
        self
    }

    /// 種類を変える。
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// ラベルの前にアイコンを置く。
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// 無効にする。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 押されたときの処理（クリック、Enter、Space）。
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// 内容（ラベル・アイコン・状態レイヤー）の色のロールを変える（Snackbar の操作など）。
    #[cfg_attr(not(feature = "overlays"), allow(dead_code))]
    pub(crate) fn content_color(mut self, role: ColorRole) -> Self {
        self.content_role = Some(role);
        self
    }
}

impl RenderOnce for Button {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let mut behaviour = gpui_headless::Button::new(self.id.clone(), self.label.clone())
            .disabled(self.disabled)
            .group(format!("md3-{}", self.id))
            .reveal_margin(crate::focus::reveal_margin());
        if let Some(label) = self.aria_label {
            behaviour = behaviour.aria_label(label.unwrap_or_default());
        }
        if let Some(handler) = self.on_click {
            behaviour = behaviour.on_click(handler);
        }
        let (base, control) = behaviour.build(window, cx);
        let theme = cx.md3();
        let style = ButtonStyle::of(self.variant);
        let radius = theme.corner(baseline_button::CONTAINER_SHAPE_ROUND);
        let content = if self.disabled {
            theme
                .color(style.disabled_content)
                .opacity(style.disabled_content_opacity)
        } else {
            theme.color(self.content_role.unwrap_or(style.content))
        };
        let leading = if self.icon.is_some() {
            style.leading_space_with_icon
        } else {
            style.leading_space
        };
        let trailing = if self.icon.is_some() || self.variant != ButtonVariant::Text {
            style.trailing_space
        } else {
            style.leading_space
        };

        let mut button = base
            .h(theme.dense(baseline_button::CONTAINER_HEIGHT))
            .min_w(px(layout::BUTTON_MIN_WIDTH))
            .pl(px(leading))
            .pr(px(trailing))
            .flex()
            .items_center()
            .justify_center()
            .gap(px(baseline_button::ICON_LABEL_SPACE))
            .corner_radius(radius)
            .text_color(content)
            .type_style(&theme.text_style(filled_tonal_button::LABEL_TEXT_FONT));

        if let Some(container) = style.container {
            button = if self.disabled {
                button.bg(theme
                    .color(filled_button::DISABLED_CONTAINER_COLOR)
                    .opacity(style.disabled_container_opacity))
            } else {
                button.bg(theme.color(container))
            };
        }
        if let Some(outline) = style.outline {
            let color = if self.disabled {
                theme
                    .color(outline)
                    .opacity(style.disabled_container_opacity)
            } else {
                theme.color(outline)
            };
            button = button.border_1().border_color(color);
        }
        if !self.disabled && style.hovered_elevation > 0.0 {
            let hovered = theme.shadows(Elevation::from_dp(style.hovered_elevation));
            button = button
                .shadow(theme.shadows(Elevation::from_dp(style.elevation)))
                .hover(move |s| s.shadow(hovered.clone()));
        }

        let content_role = self.content_role.unwrap_or(style.content);
        button
            .when_some(self.icon, |this, icon| {
                this.child(
                    Icon::new(icon)
                        .size(px(layout::BUTTON_ICON_SIZE))
                        .tint(content),
                )
            })
            .child(self.label)
            .child(
                StateLayer::new(
                    ElementId::from(SharedString::from(format!("{}-layer", control.group))),
                    control.group.clone(),
                    content_role,
                )
                .radius(radius)
                .focused(control.focus_visible)
                .disabled(self.disabled),
            )
            .child(FocusRing::new(radius).visible(control.focus_visible))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filled_button_uses_primary() {
        let style = ButtonStyle::of(ButtonVariant::Filled);
        assert_eq!(style.container, Some(ColorRole::Primary));
        assert_eq!(style.content, ColorRole::OnPrimary);
        assert!(style.hovered_elevation > style.elevation);
    }

    #[test]
    fn outlined_and_text_buttons_have_no_container() {
        for variant in [ButtonVariant::Outlined, ButtonVariant::Text] {
            assert_eq!(ButtonStyle::of(variant).container, None);
        }
        assert!(ButtonStyle::of(ButtonVariant::Outlined).outline.is_some());
    }
}
