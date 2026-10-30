//! Card（Elevated / Filled / Outlined）。

use gpui_headless::focus_handle;
use gpui_headless::is_focus_visible;
use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, Div, ElementId, IntoElement, ParentElement, RenderOnce, Role,
    SharedString, StyleRefinement, Styled, Window, div, prelude::*, px,
};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::{elevated_card, filled_card, outlined_card};
use md3_theme::md3_tokens::state;
use md3_theme::{Elevation, Md3ThemeExt, Shape};

use crate::{FocusRing, StateLayer, StyledExt};
use gpui_headless::{A11yExt, A11yNode};

/// カードの種類。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardVariant {
    /// 影で浮かせる（`SurfaceContainerLow`、Level 1）。
    #[default]
    Elevated,
    /// 塗りで区別する（`SurfaceContainerHighest`）。
    Filled,
    /// 枠線で区別する（`Surface` + `OutlineVariant`）。
    Outlined,
}

/// 種類ごとの色と高さ。
struct CardStyle {
    container: ColorRole,
    elevation: f32,
    hovered_elevation: f32,
    shape: Shape,
    disabled_container: ColorRole,
    disabled_container_opacity: f32,
    /// 無効状態の色を重ねる下地（Compose の `compositeOver` の相手）。
    disabled_base: ColorRole,
    outline: Option<(ColorRole, f32)>,
}

impl CardVariant {
    fn style(self) -> CardStyle {
        match self {
            Self::Elevated => CardStyle {
                container: elevated_card::CONTAINER_COLOR,
                elevation: elevated_card::CONTAINER_ELEVATION,
                hovered_elevation: elevated_card::HOVER_CONTAINER_ELEVATION,
                shape: elevated_card::CONTAINER_SHAPE,
                disabled_container: elevated_card::DISABLED_CONTAINER_COLOR,
                disabled_container_opacity: elevated_card::DISABLED_CONTAINER_OPACITY,
                disabled_base: elevated_card::DISABLED_CONTAINER_COLOR,
                outline: None,
            },
            Self::Filled => CardStyle {
                container: filled_card::CONTAINER_COLOR,
                elevation: filled_card::CONTAINER_ELEVATION,
                hovered_elevation: filled_card::HOVER_CONTAINER_ELEVATION,
                shape: filled_card::CONTAINER_SHAPE,
                disabled_container: filled_card::DISABLED_CONTAINER_COLOR,
                disabled_container_opacity: filled_card::DISABLED_CONTAINER_OPACITY,
                disabled_base: filled_card::CONTAINER_COLOR,
                outline: None,
            },
            Self::Outlined => CardStyle {
                container: outlined_card::CONTAINER_COLOR,
                elevation: outlined_card::CONTAINER_ELEVATION,
                hovered_elevation: outlined_card::HOVER_CONTAINER_ELEVATION,
                shape: outlined_card::CONTAINER_SHAPE,
                // Outlined の無効状態はコンテナの色を変えず、枠線を薄くする。
                disabled_container: outlined_card::CONTAINER_COLOR,
                disabled_container_opacity: 1.0,
                disabled_base: outlined_card::CONTAINER_COLOR,
                outline: Some((outlined_card::OUTLINE_COLOR, outlined_card::OUTLINE_WIDTH)),
            },
        }
    }
}

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// 押せるカードの設定。
struct Clickable {
    id: ElementId,
    label: SharedString,
    handler: ClickHandler,
}

/// カード。中身は子要素で渡す（余白も呼び出し側が決める）。
///
/// [`Card::on_click`] を付けると押せるカードになる（役割 Button、ホバーで高さが上がる）。
/// 付けなければ装飾の入れ物で、アクセシビリティツリーには出ない（中身の要素がそれぞれ出る）。
#[derive(IntoElement)]
pub struct Card {
    variant: CardVariant,
    base: Div,
    clickable: Option<Clickable>,
    disabled: bool,
}

impl Card {
    /// 種類を指定して作る。
    pub fn new(variant: CardVariant) -> Self {
        Self {
            variant,
            base: div(),
            clickable: None,
            disabled: false,
        }
    }

    /// 押せるカードにする。`label` は読み上げられる名前（カードの見出しなど）。
    pub fn on_click(
        mut self,
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.clickable = Some(Clickable {
            id: id.into(),
            label: label.into(),
            handler: Rc::new(handler),
        });
        self
    }

    /// 無効にする（押せるカードだけに効く）。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// アクセシビリティツリーに出す情報。押せないカードは装飾なので出さない（`None`）。
    pub fn a11y_node(&self) -> Option<A11yNode> {
        self.clickable.as_ref().map(|clickable| {
            A11yNode::new(Role::Button)
                .label(clickable.label.clone())
                .disabled(self.disabled)
        })
    }
}

impl ParentElement for Card {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Card {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Card {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let node = self.a11y_node();
        let style = self.variant.style();
        let theme = cx.md3().clone();
        let radius = theme.corner(style.shape);
        let disabled = self.disabled && self.clickable.is_some();

        let mut card = self.base.relative().corner_radius(radius);
        card = if disabled {
            // Compose と同じく、半透明の色を下地に重ねた不透明な色で塗る（影が透けないように）。
            let container = theme.color(style.disabled_base).blend(
                theme
                    .color(style.disabled_container)
                    .opacity(style.disabled_container_opacity),
            );
            card.bg(container).text_color(
                theme
                    .color(ColorRole::OnSurface)
                    .opacity(state::DISABLED_CONTENT_OPACITY),
            )
        } else {
            card.bg(theme.color(style.container))
        };
        if let Some((role, width)) = style.outline {
            let color = if disabled {
                theme
                    .color(outlined_card::DISABLED_OUTLINE_COLOR)
                    .opacity(outlined_card::DISABLED_OUTLINE_OPACITY)
            } else {
                theme.color(role)
            };
            card = card.border(px(width)).border_color(color);
        }
        let resting = theme.shadows(Elevation::from_dp(style.elevation));

        let (Some(clickable), Some(node)) = (self.clickable, node) else {
            return card.shadow(resting).into_any_element();
        };

        // 利用者が付けた大きさをそのまま使えるよう、操作の土台（control）で包まずに直接付ける。
        let group: SharedString = format!("md3-{}", clickable.id).into();
        let mut card = card
            .id(clickable.id.clone())
            .group(group.clone())
            .a11y(node);
        let focus_visible = if disabled {
            false
        } else {
            let handle = focus_handle(clickable.id.clone(), window, cx);
            let handler = clickable.handler;
            card = card
                .track_focus(&handle)
                .on_mouse_down(gpui::MouseButton::Left, {
                    let handle = handle.clone();
                    move |_, window, cx| gpui_headless::clear_focus_visible(&handle, window, cx)
                })
                .child(crate::scroll::reveal_on_focus(
                    &clickable.id,
                    handle.clone(),
                    window,
                    cx,
                ))
                .cursor_pointer()
                .on_click(move |event, window, cx| handler(event, window, cx));
            is_focus_visible(&handle, window, cx)
        };
        let hovered = theme.shadows(Elevation::from_dp(style.hovered_elevation));
        let card = if disabled {
            card.shadow(resting)
        } else {
            card.shadow(resting)
                .hover(move |s| s.shadow(hovered.clone()))
        };
        card.child(
            StateLayer::new(
                SharedString::from(format!("{group}-layer")),
                group,
                ColorRole::OnSurface,
            )
            .radius(radius)
            .focused(focus_visible)
            .disabled(disabled),
        )
        .child(FocusRing::new(radius).visible(focus_visible))
        .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clickable_cards_are_buttons() {
        let card = Card::new(CardVariant::Filled).on_click("c", "旅行の計画", |_, _, _| {});
        let node = card.a11y_node().expect("clickable");
        assert_eq!(node.role, Role::Button);
        assert_eq!(node.label.as_deref(), Some("旅行の計画"));
        assert_eq!(Card::new(CardVariant::Outlined).a11y_node(), None);
    }

    #[test]
    fn variants_use_their_tokens() {
        assert_eq!(
            CardVariant::Elevated.style().container,
            ColorRole::SurfaceContainerLow
        );
        assert_eq!(
            CardVariant::Filled.style().container,
            ColorRole::SurfaceContainerHighest
        );
        assert!(CardVariant::Outlined.style().outline.is_some());
    }
}
