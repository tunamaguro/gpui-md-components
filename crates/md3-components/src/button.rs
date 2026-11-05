//! MD3 のボタン
//!
//! 寸法・色・形は Compose Material3 の `Button.kt` (`ButtonDefaults`) と生成トークン
//! (@ 2a941dd11aabd9bb7d2016ed159764b220117c20) に従う。Compose の実装がトークンと異なる値を使う箇所と、
//! Compose にない値は Material Web の Expressive 世代のトークン ([`web_component`]) に従う

use gpui::{
    AnimationExt as _, App, ClickEvent, ElementId, IntoElement, ParentElement as _,
    Refineable as _, RenderOnce, SharedString, SpringAnimation, StatefulInteractiveElement as _,
    Styled as _, Window, div, prelude::FluentBuilder as _, size,
};
use md3_icons::IconName;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::{
    button_large, button_medium, button_small, button_x_large, button_x_small, elevated_button,
    filled_button, outlined_button, text_button, tonal_button,
};
use md3_theme::md3_tokens::shape::ShapeKey;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::md3_tokens::{Dp, elevation, web_component};
use md3_theme::{ThemeExt as _, dp};

use crate::indication::{self, Interaction};
use crate::{MIN_INTERACTIVE_SIZE, icon};

/// ボタンの強調の度合いと見た目
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ButtonVariant {
    /// 影を持つ面のボタン。背景の模様から切り離す必要がある場合に使う
    Elevated,
    /// 主色で塗ったボタン。最も強調する操作に使う
    #[default]
    Filled,
    /// 第二色のコンテナで塗ったボタン。Filled と Outlined の中間の強調に使う
    Tonal,
    /// 枠線だけのボタン。中程度の強調に使う
    Outlined,
    /// 文字だけのボタン。最も弱い強調に使う
    Text,
}

/// ボタンの高さと、それに合わせた余白・アイコン・文字の大きさ
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ButtonSize {
    /// 高さ 32dp
    ExtraSmall,
    /// 高さ 40dp
    #[default]
    Small,
    /// 高さ 56dp
    Medium,
    /// 高さ 96dp
    Large,
    /// 高さ 136dp
    ExtraLarge,
}

/// 押していないときのボタンの角の形。押している間は大きさごとに決まった角の小さい形に変わる
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ButtonShape {
    /// 両端が半円の形
    #[default]
    Round,
    /// 角を大きさごとに決まった半径で丸めた形
    Square,
}

/// Compose の `ButtonDefaults.MinWidth`
const MIN_WIDTH: Dp = Dp(58.0);

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// MD3 のボタン
///
/// 支援技術には role `button` と、ラベルの文字列を名前として公開する。
/// マウスのクリック・フォーカス中の Enter と Space・支援技術からのクリック操作で [`Button::on_click`] のハンドラを呼ぶ。
/// 操作できる範囲は、見た目の高さが 48dp 未満の大きさでも 48x48dp を下回らない
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: SharedString,
    icon: Option<IconName>,
    variant: ButtonVariant,
    size: ButtonSize,
    shape: ButtonShape,
    disabled: bool,
    on_click: Option<ClickHandler>,
}

impl Button {
    /// `label` を表示するボタンを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。`id` が変わるか 1 フレームでも描画されないと、
    /// フォーカスと押下の表示が失われる
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            variant: ButtonVariant::default(),
            size: ButtonSize::default(),
            shape: ButtonShape::default(),
            disabled: false,
            on_click: None,
        }
    }

    /// 強調の度合いと見た目を決める。既定では [`ButtonVariant::Filled`]
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// 高さと、余白・アイコン・文字の大きさを決める。既定では [`ButtonSize::Small`]
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// 押していないときの角の形を決める。既定では [`ButtonShape::Round`]
    pub fn shape(mut self, shape: ButtonShape) -> Self {
        self.shape = shape;
        self
    }

    /// ラベルの前に `icon` を表示する。アイコンは名前に含まれない
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// ボタンを無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効なボタンは押されてもハンドラを呼ばず、Tab で到達できず、支援技術には disabled として公開される。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 押されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// 設定しない場合、ボタンは押されても何も呼ばず、支援技術にクリック操作を公開しない
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

/// 大きさごとの寸法
struct Metrics {
    height: Dp,
    round: ShapeKey,
    square: ShapeKey,
    pressed: ShapeKey,
    icon_size: Dp,
    icon_label_space: Dp,
    leading: Dp,
    trailing: Dp,
    outline_width: Dp,
    type_scale: TypeScale,
}

impl ButtonSize {
    fn metrics(self) -> Metrics {
        // 書体は ButtonDefaults.textStyleFor による
        match self {
            // 左右の余白は ButtonDefaults.ExtraSmallContentPadding と Material Web の
            // md.comp.button.xsmall.leading-space (12) による。ButtonXSmallTokens は 16
            ButtonSize::ExtraSmall => Metrics {
                height: button_x_small::CONTAINER_HEIGHT,
                round: button_x_small::CONTAINER_SHAPE_ROUND,
                square: button_x_small::CONTAINER_SHAPE_SQUARE,
                pressed: button_x_small::PRESSED_CONTAINER_SHAPE,
                icon_size: button_x_small::ICON_SIZE,
                icon_label_space: button_x_small::ICON_LABEL_SPACE,
                leading: Dp(12.0),
                trailing: Dp(12.0),
                outline_width: button_x_small::OUTLINED_OUTLINE_WIDTH,
                type_scale: TypeScale::LabelLarge,
            },
            ButtonSize::Small => Metrics {
                height: button_small::CONTAINER_HEIGHT,
                round: button_small::CONTAINER_SHAPE_ROUND,
                square: button_small::CONTAINER_SHAPE_SQUARE,
                pressed: button_small::PRESSED_CONTAINER_SHAPE,
                icon_size: button_small::ICON_SIZE,
                icon_label_space: button_small::ICON_LABEL_SPACE,
                leading: button_small::LEADING_SPACE,
                trailing: button_small::TRAILING_SPACE,
                outline_width: button_small::OUTLINED_OUTLINE_WIDTH,
                type_scale: TypeScale::LabelLarge,
            },
            ButtonSize::Medium => Metrics {
                height: button_medium::CONTAINER_HEIGHT,
                round: button_medium::CONTAINER_SHAPE_ROUND,
                square: button_medium::CONTAINER_SHAPE_SQUARE,
                pressed: button_medium::PRESSED_CONTAINER_SHAPE,
                icon_size: button_medium::ICON_SIZE,
                icon_label_space: button_medium::ICON_LABEL_SPACE,
                leading: button_medium::LEADING_SPACE,
                trailing: button_medium::TRAILING_SPACE,
                outline_width: button_medium::OUTLINED_OUTLINE_WIDTH,
                type_scale: TypeScale::TitleMedium,
            },
            ButtonSize::Large => Metrics {
                height: button_large::CONTAINER_HEIGHT,
                round: button_large::CONTAINER_SHAPE_ROUND,
                square: button_large::CONTAINER_SHAPE_SQUARE,
                pressed: button_large::PRESSED_CONTAINER_SHAPE,
                icon_size: button_large::ICON_SIZE,
                icon_label_space: button_large::ICON_LABEL_SPACE,
                leading: button_large::LEADING_SPACE,
                trailing: button_large::TRAILING_SPACE,
                outline_width: button_large::OUTLINED_OUTLINE_WIDTH,
                type_scale: TypeScale::HeadlineSmall,
            },
            ButtonSize::ExtraLarge => Metrics {
                height: button_x_large::CONTAINER_HEIGHT,
                round: button_x_large::CONTAINER_SHAPE_ROUND,
                square: button_x_large::CONTAINER_SHAPE_SQUARE,
                pressed: button_x_large::PRESSED_CONTAINER_SHAPE,
                icon_size: button_x_large::ICON_SIZE,
                icon_label_space: button_x_large::ICON_LABEL_SPACE,
                leading: button_x_large::LEADING_SPACE,
                trailing: button_x_large::TRAILING_SPACE,
                outline_width: button_x_large::OUTLINED_OUTLINE_WIDTH,
                type_scale: TypeScale::HeadlineLarge,
            },
        }
    }
}

/// 種類ごとの色と高さ
struct Style {
    container: Option<ColorRole>,
    content: ColorRole,
    disabled_container: Option<(ColorRole, f32)>,
    disabled_content: (ColorRole, f32),
    outline: Option<(ColorRole, ColorRole)>,
    /// (通常, hover, フォーカス, 押下, 無効)
    elevation: [Dp; 5],
}

impl ButtonVariant {
    fn style(self) -> Style {
        let none = [elevation::LEVEL0; 5];
        match self {
            ButtonVariant::Elevated => Style {
                container: Some(elevated_button::CONTAINER_COLOR),
                content: elevated_button::LABEL_TEXT_COLOR,
                disabled_container: Some((
                    elevated_button::DISABLED_CONTAINER_COLOR,
                    elevated_button::DISABLED_CONTAINER_OPACITY,
                )),
                disabled_content: (
                    elevated_button::DISABLED_LABEL_TEXT_COLOR,
                    elevated_button::DISABLED_LABEL_TEXT_OPACITY,
                ),
                outline: None,
                elevation: [
                    elevated_button::CONTAINER_ELEVATION,
                    elevated_button::HOVERED_CONTAINER_ELEVATION,
                    elevated_button::FOCUSED_CONTAINER_ELEVATION,
                    elevated_button::PRESSED_CONTAINER_ELEVATION,
                    elevated_button::DISABLED_CONTAINER_ELEVATION,
                ],
            },
            ButtonVariant::Filled => Style {
                container: Some(filled_button::CONTAINER_COLOR),
                content: filled_button::LABEL_TEXT_COLOR,
                disabled_container: Some((
                    filled_button::DISABLED_CONTAINER_COLOR,
                    filled_button::DISABLED_CONTAINER_OPACITY,
                )),
                disabled_content: (
                    filled_button::DISABLED_LABEL_TEXT_COLOR,
                    filled_button::DISABLED_LABEL_TEXT_OPACITY,
                ),
                outline: None,
                elevation: [
                    filled_button::CONTAINER_ELEVATION,
                    filled_button::HOVERED_CONTAINER_ELEVATION,
                    filled_button::FOCUSED_CONTAINER_ELEVATION,
                    filled_button::PRESSED_CONTAINER_ELEVATION,
                    filled_button::DISABLED_CONTAINER_ELEVATION,
                ],
            },
            // Compose の FilledTonalButton は旧世代の FilledTonalButtonTokens (v0_103) を使うが、
            // Expressive 世代の TonalButtonTokens (v0_11_0) に従う
            ButtonVariant::Tonal => Style {
                container: Some(tonal_button::CONTAINER_COLOR),
                content: tonal_button::LABEL_TEXT_COLOR,
                disabled_container: Some((
                    tonal_button::DISABLED_CONTAINER_COLOR,
                    tonal_button::DISABLED_CONTAINER_OPACITY,
                )),
                disabled_content: (
                    tonal_button::DISABLED_LABEL_TEXT_COLOR,
                    tonal_button::DISABLED_LABEL_TEXT_OPACITY,
                ),
                outline: None,
                elevation: [
                    tonal_button::CONTAINER_ELEVATION,
                    tonal_button::HOVERED_CONTAINER_ELEVATION,
                    tonal_button::FOCUSED_CONTAINER_ELEVATION,
                    tonal_button::PRESSED_CONTAINER_ELEVATION,
                    tonal_button::DISABLED_CONTAINER_ELEVATION,
                ],
            },
            ButtonVariant::Outlined => Style {
                container: None,
                content: outlined_button::LABEL_TEXT_COLOR,
                disabled_container: None,
                disabled_content: (
                    outlined_button::DISABLED_LABEL_TEXT_COLOR,
                    outlined_button::DISABLED_LABEL_TEXT_OPACITY,
                ),
                outline: Some((
                    outlined_button::OUTLINE_COLOR,
                    outlined_button::DISABLED_OUTLINE_COLOR,
                )),
                elevation: none,
            },
            ButtonVariant::Text => Style {
                container: None,
                content: web_component::text_button::LABEL_TEXT_COLOR,
                disabled_container: None,
                disabled_content: (
                    text_button::DISABLED_LABEL_COLOR,
                    text_button::DISABLED_LABEL_OPACITY,
                ),
                outline: None,
                elevation: none,
            },
        }
    }
}

impl RenderOnce for Button {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let focus_handle = crate::focus_handle(&self.id, window, cx);
        let interaction = Interaction::state(&self.id, window, cx);
        let enabled = !self.disabled;
        let focused = focus_handle.is_focused(window);
        Interaction::sync(&interaction, enabled, focused, cx);
        let focus_visible = focused && window.last_input_was_keyboard();
        let (hovered, pressed, layer_opacity) = {
            let state = interaction.read(cx);
            (
                state.is_hovered(),
                state.is_pressed(),
                state.state_layer_opacity(focus_visible, cx),
            )
        };

        let metrics = self.size.metrics();
        let style = self.variant.style();
        let content = if enabled {
            theme.color(style.content)
        } else {
            let (role, opacity) = style.disabled_content;
            theme.color(role).opacity(opacity)
        };
        let container = if enabled {
            style.container.map(|role| theme.color(role))
        } else {
            style
                .disabled_container
                .map(|(role, opacity)| theme.color(role).opacity(opacity))
        };
        let outline = style.outline.map(|(enabled_role, disabled_role)| {
            theme.color(if enabled { enabled_role } else { disabled_role })
        });
        let [rest, hover, focus, press, disabled] = style.elevation;
        let elevation = if !enabled {
            disabled
        } else if pressed {
            press
        } else if hovered {
            hover
        } else if focus_visible {
            focus
        } else {
            rest
        };
        let shadow = elevation::level_of(elevation)
            .filter(|level| *level > 0)
            .map(|level| theme.shadow(f32::from(level)));

        // 幅は高さ以上になるため、Full の角は高さの半分になる
        let height = dp(metrics.height);
        let square = size(height, height);
        let shape = match self.shape {
            ButtonShape::Round => metrics.round,
            ButtonShape::Square => metrics.square,
        };
        let rest_radii = theme.corner_radii(shape, square);
        let pressed_radii = theme.corner_radii(metrics.pressed, square);
        let spring = theme.spring(web_component::button::PRESSED_CONTAINER_CORNER_SIZE_MOTION);

        let mut label = div().whitespace_nowrap();
        label
            .text_style()
            .refine(&theme.text_style(metrics.type_scale));
        let label = label.child(self.label.clone());
        let ripple_state = interaction.clone();

        let container = div()
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .gap(dp(metrics.icon_label_space))
            .h(height)
            .min_w(dp(MIN_WIDTH))
            .pl(dp(metrics.leading))
            .pr(dp(metrics.trailing))
            .text_color(content)
            .when_some(container, |this, color| this.bg(color))
            .when_some(outline, |this, color| {
                this.border(dp(metrics.outline_width)).border_color(color)
            })
            .when_some(shadow, |this, shadow| this.shadow(shadow))
            .when_some(self.icon, |this, name| {
                this.child(icon(name, dp(metrics.icon_size), content))
            })
            .child(label)
            .with_spring(
                "md3-shape",
                SpringAnimation::new(spring).to(pressed),
                move |this, phase| {
                    let radii = indication::lerp_corners(rest_radii, pressed_radii, phase.0);
                    indication::with_corners(this, radii)
                        .when(enabled, |this| {
                            this.child(indication::state_layer(content, layer_opacity, radii))
                                .child(indication::ripple(ripple_state, content, radii))
                        })
                        .when(focus_visible, |this| {
                            this.child(indication::focus_ring(&theme, radii))
                        })
                },
            );

        let button = gpui_headless::button::Button::new(self.id)
            .track_focus(&focus_handle)
            .disabled(self.disabled)
            .aria_label(self.label)
            .flex()
            .items_center()
            .justify_center()
            .min_w(dp(MIN_INTERACTIVE_SIZE))
            .min_h(dp(MIN_INTERACTIVE_SIZE))
            .child(container)
            .when_some(self.on_click, |this, handler| this.on_click(handler));
        if enabled {
            indication::track(button, &interaction, &["enter", "space"])
        } else {
            button
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use gpui::{Context, Entity, Modifiers, Render, TestAppContext, VisualTestContext, point, px};

    use super::*;
    use crate::test_support::{focus_next, init_theme, press_key};

    struct ButtonView {
        size: ButtonSize,
        disabled: bool,
        clicks: Rc<Cell<usize>>,
    }

    impl Render for ButtonView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let clicks = self.clicks.clone();
            div().size_full().child(
                Button::new("button", "Send")
                    .size(self.size)
                    .disabled(self.disabled)
                    .on_click(move |_, _, _| clicks.set(clicks.get() + 1)),
            )
        }
    }

    fn setup(
        cx: &mut TestAppContext,
        size: ButtonSize,
        disabled: bool,
    ) -> (Entity<ButtonView>, &mut VisualTestContext) {
        init_theme(cx);
        cx.add_window_view(|_, _| ButtonView {
            size,
            disabled,
            clicks: Rc::default(),
        })
    }

    fn clicks(view: &Entity<ButtonView>, cx: &mut VisualTestContext) -> usize {
        view.read_with(cx, |view, _| view.clicks.get())
    }

    #[gpui::test]
    fn 見た目より外側の_48dp_の範囲を押しても押せる(cx: &mut TestAppContext) {
        // XS の見た目の高さは 32dp で、上下に 8dp ずつ操作できる範囲が広がる
        let (view, cx) = setup(cx, ButtonSize::ExtraSmall, false);
        cx.simulate_click(point(px(20.0), px(3.0)), Modifiers::none());
        assert_eq!(clicks(&view, cx), 1);
        cx.simulate_click(point(px(20.0), px(52.0)), Modifiers::none());
        assert_eq!(clicks(&view, cx), 1, "48dp の範囲の外は押せないこと");
    }

    #[gpui::test]
    fn enter_と_space_で_1_回ずつ押せる(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, ButtonSize::Small, false);
        focus_next(cx);
        press_key(cx, "enter");
        press_key(cx, "space");
        assert_eq!(clicks(&view, cx), 2);
    }

    #[gpui::test]
    fn 無効なボタンは_tab_で到達できず押せない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, ButtonSize::Small, true);
        focus_next(cx);
        press_key(cx, "enter");
        cx.simulate_click(point(px(20.0), px(20.0)), Modifiers::none());
        assert_eq!(clicks(&view, cx), 0);
    }
}
