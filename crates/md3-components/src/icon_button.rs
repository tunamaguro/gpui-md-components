//! MD3 のアイコンボタン
//!
//! 寸法・色・形は Compose Material3 の `IconButton.kt` / `IconButtonDefaults.kt` と生成トークン
//! (@ 2a941dd11aabd9bb7d2016ed159764b220117c20) に従う。色は Compose の `*Vibrant*` の既定値と同じく
//! トークンの色を使う。押下時の形の変化のバネは Material Web の Expressive 世代のトークン ([`web_component`]) に従う。
//! 選択状態を持つトグルのアイコンボタンは含まない

use gpui::{
    AnimationExt as _, App, ClickEvent, ElementId, IntoElement, ParentElement as _, RenderOnce,
    SharedString, SpringAnimation, StatefulInteractiveElement as _, Styled as _, Window, div,
    prelude::FluentBuilder as _, size,
};
use md3_icons::IconName;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::{
    filled_icon_button, filled_tonal_icon_button, icon_button, large_icon_button,
    medium_icon_button, outlined_icon_button, small_icon_button, x_large_icon_button,
    x_small_icon_button,
};
use md3_theme::md3_tokens::shape::ShapeKey;
use md3_theme::md3_tokens::{Dp, web_component};
use md3_theme::{ThemeExt as _, dp};

use crate::indication::{self, Indication};
use crate::{MIN_INTERACTIVE_SIZE, icon};

/// アイコンボタンの強調の度合いと見た目
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum IconButtonVariant {
    /// アイコンだけのボタン。最も弱い強調に使う
    #[default]
    Standard,
    /// 主色で塗ったボタン。最も強調する操作に使う
    Filled,
    /// 第二色のコンテナで塗ったボタン
    Tonal,
    /// 枠線だけのボタン
    Outlined,
}

/// アイコンボタンの高さと、それに合わせたアイコンの大きさ・左右の余白
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum IconButtonSize {
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

/// アイコンの左右の余白の広さ。アイコンボタンの幅を決める
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum IconButtonWidth {
    /// 狭い余白
    Narrow,
    /// 標準の余白
    #[default]
    Default,
    /// 広い余白
    Wide,
}

/// 押していないときのアイコンボタンの角の形。押している間は大きさごとに決まった角の小さい形に変わる
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum IconButtonShape {
    /// 円、または両端が半円の形
    #[default]
    Round,
    /// 角を大きさごとに決まった半径で丸めた形
    Square,
}

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// MD3 のアイコンボタン
///
/// 支援技術には role `button` と、[`IconButton::new`] に渡したラベルを名前として公開する。
/// マウスのクリック・フォーカス中の Enter と Space・支援技術からのクリック操作で [`IconButton::on_click`] のハンドラを呼ぶ。
/// 操作できる範囲は、見た目が 48dp 未満の大きさでも 48x48dp を下回らない
#[derive(IntoElement)]
pub struct IconButton {
    id: ElementId,
    icon: IconName,
    label: SharedString,
    variant: IconButtonVariant,
    size: IconButtonSize,
    width: IconButtonWidth,
    shape: IconButtonShape,
    disabled: bool,
    on_click: Option<ClickHandler>,
}

impl IconButton {
    /// `icon` を表示するアイコンボタンを作る。`label` は表示せず、支援技術に名前として公開する
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。`id` が変わるか 1 フレームでも描画されないと、
    /// フォーカスと押下の表示が失われる
    pub fn new(id: impl Into<ElementId>, icon: IconName, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            icon,
            label: label.into(),
            variant: IconButtonVariant::default(),
            size: IconButtonSize::default(),
            width: IconButtonWidth::default(),
            shape: IconButtonShape::default(),
            disabled: false,
            on_click: None,
        }
    }

    /// 強調の度合いと見た目を決める。既定では [`IconButtonVariant::Standard`]
    pub fn variant(mut self, variant: IconButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// 高さとアイコンの大きさを決める。既定では [`IconButtonSize::Small`]
    pub fn size(mut self, size: IconButtonSize) -> Self {
        self.size = size;
        self
    }

    /// アイコンの左右の余白の広さを決める。既定では [`IconButtonWidth::Default`]
    pub fn width(mut self, width: IconButtonWidth) -> Self {
        self.width = width;
        self
    }

    /// 押していないときの角の形を決める。既定では [`IconButtonShape::Round`]
    pub fn shape(mut self, shape: IconButtonShape) -> Self {
        self.shape = shape;
        self
    }

    /// アイコンボタンを無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効なアイコンボタンは押されてもハンドラを呼ばず、Tab で到達できず、支援技術には disabled として公開される。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 押されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// 設定しない場合、アイコンボタンは押されても何も呼ばず、支援技術にクリック操作を公開しない
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
    /// (Narrow, Default, Wide) の左右それぞれの余白
    spaces: [Dp; 3],
    outline_width: Dp,
}

impl IconButtonSize {
    fn metrics(self) -> Metrics {
        // 幅は IconButtonDefaults.*ContainerSize と同じく、アイコンの大きさに左右の余白を足す
        match self {
            IconButtonSize::ExtraSmall => Metrics {
                height: x_small_icon_button::CONTAINER_HEIGHT,
                round: x_small_icon_button::CONTAINER_SHAPE_ROUND,
                square: x_small_icon_button::CONTAINER_SHAPE_SQUARE,
                pressed: x_small_icon_button::PRESSED_CONTAINER_SHAPE,
                icon_size: x_small_icon_button::ICON_SIZE,
                spaces: [
                    x_small_icon_button::NARROW_LEADING_SPACE,
                    x_small_icon_button::DEFAULT_LEADING_SPACE,
                    x_small_icon_button::WIDE_LEADING_SPACE,
                ],
                outline_width: x_small_icon_button::OUTLINED_OUTLINE_WIDTH,
            },
            IconButtonSize::Small => Metrics {
                height: small_icon_button::CONTAINER_HEIGHT,
                round: small_icon_button::CONTAINER_SHAPE_ROUND,
                square: small_icon_button::CONTAINER_SHAPE_SQUARE,
                pressed: small_icon_button::PRESSED_CONTAINER_SHAPE,
                icon_size: small_icon_button::ICON_SIZE,
                spaces: [
                    small_icon_button::NARROW_LEADING_SPACE,
                    small_icon_button::DEFAULT_LEADING_SPACE,
                    small_icon_button::WIDE_LEADING_SPACE,
                ],
                outline_width: small_icon_button::OUTLINED_OUTLINE_WIDTH,
            },
            IconButtonSize::Medium => Metrics {
                height: medium_icon_button::CONTAINER_HEIGHT,
                round: medium_icon_button::CONTAINER_SHAPE_ROUND,
                square: medium_icon_button::CONTAINER_SHAPE_SQUARE,
                pressed: medium_icon_button::PRESSED_CONTAINER_SHAPE,
                icon_size: medium_icon_button::ICON_SIZE,
                spaces: [
                    medium_icon_button::NARROW_LEADING_SPACE,
                    medium_icon_button::DEFAULT_LEADING_SPACE,
                    medium_icon_button::WIDE_LEADING_SPACE,
                ],
                outline_width: medium_icon_button::OUTLINED_OUTLINE_WIDTH,
            },
            // LargeIconButtonTokens (v0_11_0) は標準の余白を Uniform の名前で持つ
            IconButtonSize::Large => Metrics {
                height: large_icon_button::CONTAINER_HEIGHT,
                round: large_icon_button::CONTAINER_SHAPE_ROUND,
                square: large_icon_button::CONTAINER_SHAPE_SQUARE,
                pressed: large_icon_button::PRESSED_CONTAINER_SHAPE,
                icon_size: large_icon_button::ICON_SIZE,
                spaces: [
                    large_icon_button::NARROW_LEADING_SPACE,
                    large_icon_button::UNIFORM_LEADING_SPACE,
                    large_icon_button::WIDE_LEADING_SPACE,
                ],
                outline_width: large_icon_button::OUTLINED_OUTLINE_WIDTH,
            },
            IconButtonSize::ExtraLarge => Metrics {
                height: x_large_icon_button::CONTAINER_HEIGHT,
                round: x_large_icon_button::CONTAINER_SHAPE_ROUND,
                square: x_large_icon_button::CONTAINER_SHAPE_SQUARE,
                pressed: x_large_icon_button::PRESSED_CONTAINER_SHAPE,
                icon_size: x_large_icon_button::ICON_SIZE,
                spaces: [
                    x_large_icon_button::NARROW_LEADING_SPACE,
                    x_large_icon_button::DEFAULT_LEADING_SPACE,
                    x_large_icon_button::WIDE_LEADING_SPACE,
                ],
                outline_width: x_large_icon_button::OUTLINED_OUTLINE_WIDTH,
            },
        }
    }
}

/// 種類ごとの色
struct Style {
    container: Option<(ColorRole, (ColorRole, f32))>,
    content: ColorRole,
    disabled_content: (ColorRole, f32),
    outline: Option<(ColorRole, ColorRole)>,
}

impl IconButtonVariant {
    fn style(self) -> Style {
        match self {
            IconButtonVariant::Standard => Style {
                container: None,
                content: icon_button::COLOR,
                disabled_content: (icon_button::DISABLED_COLOR, icon_button::DISABLED_OPACITY),
                outline: None,
            },
            IconButtonVariant::Filled => Style {
                container: Some((
                    filled_icon_button::CONTAINER_COLOR,
                    (
                        filled_icon_button::DISABLED_CONTAINER_COLOR,
                        filled_icon_button::DISABLED_CONTAINER_OPACITY,
                    ),
                )),
                content: filled_icon_button::COLOR,
                disabled_content: (
                    filled_icon_button::DISABLED_COLOR,
                    filled_icon_button::DISABLED_OPACITY,
                ),
                outline: None,
            },
            IconButtonVariant::Tonal => Style {
                container: Some((
                    filled_tonal_icon_button::CONTAINER_COLOR,
                    (
                        filled_tonal_icon_button::DISABLED_CONTAINER_COLOR,
                        filled_tonal_icon_button::DISABLED_CONTAINER_OPACITY,
                    ),
                )),
                content: filled_tonal_icon_button::COLOR,
                disabled_content: (
                    filled_tonal_icon_button::DISABLED_COLOR,
                    filled_tonal_icon_button::DISABLED_OPACITY,
                ),
                outline: None,
            },
            IconButtonVariant::Outlined => Style {
                container: None,
                content: outlined_icon_button::COLOR,
                disabled_content: (
                    outlined_icon_button::DISABLED_COLOR,
                    outlined_icon_button::DISABLED_OPACITY,
                ),
                outline: Some((
                    outlined_icon_button::OUTLINE_COLOR,
                    outlined_icon_button::DISABLED_OUTLINE_COLOR,
                )),
            },
        }
    }
}

impl RenderOnce for IconButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let enabled = !self.disabled;
        let indication = Indication::new(&self.id, enabled, window, cx);

        let metrics = self.size.metrics();
        let style = self.variant.style();
        let content = if enabled {
            theme.color(style.content)
        } else {
            let (role, opacity) = style.disabled_content;
            theme.color(role).opacity(opacity)
        };
        let container = style.container.map(|(role, (disabled_role, opacity))| {
            if enabled {
                theme.color(role)
            } else {
                theme.color(disabled_role).opacity(opacity)
            }
        });
        let outline = style.outline.map(|(enabled_role, disabled_role)| {
            theme.color(if enabled { enabled_role } else { disabled_role })
        });

        let space = match self.width {
            IconButtonWidth::Narrow => metrics.spaces[0],
            IconButtonWidth::Default => metrics.spaces[1],
            IconButtonWidth::Wide => metrics.spaces[2],
        };
        let width = dp(metrics.icon_size) + dp(space) * 2.0;
        let height = dp(metrics.height);
        let shape = match self.shape {
            IconButtonShape::Round => metrics.round,
            IconButtonShape::Square => metrics.square,
        };
        let bounds = size(width, height);
        let rest_radii = theme.corner_radii(shape, bounds);
        let pressed_radii = theme.corner_radii(metrics.pressed, bounds);
        let spring = theme.spring(web_component::icon_button::PRESSED_CONTAINER_CORNER_SIZE_MOTION);

        let container = div()
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .flex_none()
            .w(width)
            .h(height)
            .when_some(container, |this, color| this.bg(color))
            .when_some(outline, |this, color| {
                this.border(dp(metrics.outline_width)).border_color(color)
            })
            .child(icon(self.icon, dp(metrics.icon_size), content))
            .with_spring(
                "md3-shape",
                SpringAnimation::new(spring).to(indication.pressed),
                {
                    let indication = indication.clone();
                    move |this, phase| {
                        let radii = indication::lerp_corners(rest_radii, pressed_radii, phase.0);
                        let this = indication::with_corners(this, radii);
                        let this = indication.layers(this, content, content, radii);
                        indication.focus_ring(this, &theme, radii)
                    }
                },
            );

        let button = gpui_headless::button::Button::new(self.id)
            .track_focus(indication.focus_handle())
            .disabled(self.disabled)
            .aria_label(self.label)
            .flex()
            .items_center()
            .justify_center()
            .min_w(dp(MIN_INTERACTIVE_SIZE))
            .min_h(dp(MIN_INTERACTIVE_SIZE))
            .child(container)
            .when_some(self.on_click, |this, handler| this.on_click(handler));
        indication.track(button, &["enter", "space"])
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use gpui::{Context, Entity, Modifiers, Render, TestAppContext, VisualTestContext, point, px};

    use super::*;
    use crate::test_support::{focus_next, init_theme, press_key};

    struct IconButtonView {
        disabled: bool,
        clicks: Rc<Cell<usize>>,
    }

    impl Render for IconButtonView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let clicks = self.clicks.clone();
            div().size_full().child(
                IconButton::new("icon-button", IconName::Favorite, "Favorite")
                    .size(IconButtonSize::ExtraSmall)
                    .width(IconButtonWidth::Narrow)
                    .disabled(self.disabled)
                    .on_click(move |_, _, _| clicks.set(clicks.get() + 1)),
            )
        }
    }

    fn setup(
        cx: &mut TestAppContext,
        disabled: bool,
    ) -> (Entity<IconButtonView>, &mut VisualTestContext) {
        init_theme(cx);
        cx.add_window_view(|_, _| IconButtonView {
            disabled,
            clicks: Rc::default(),
        })
    }

    fn clicks(view: &Entity<IconButtonView>, cx: &mut VisualTestContext) -> usize {
        view.read_with(cx, |view, _| view.clicks.get())
    }

    #[gpui::test]
    fn 見た目より外側の_48dp_の範囲を押しても押せる(cx: &mut TestAppContext) {
        // XS の Narrow は幅 28dp・高さ 32dp で、操作できる範囲の高さは 48dp に広がる
        let (view, cx) = setup(cx, false);
        cx.simulate_click(point(px(2.0), px(2.0)), Modifiers::none());
        assert_eq!(clicks(&view, cx), 1);
        cx.simulate_click(point(px(46.0), px(46.0)), Modifiers::none());
        assert_eq!(clicks(&view, cx), 2);
        cx.simulate_click(point(px(20.0), px(52.0)), Modifiers::none());
        assert_eq!(clicks(&view, cx), 2, "48dp の範囲の外は押せないこと");
    }

    #[gpui::test]
    fn enter_と_space_で_1_回ずつ押せる(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, false);
        focus_next(cx);
        press_key(cx, "enter");
        press_key(cx, "space");
        assert_eq!(clicks(&view, cx), 2);
    }

    #[gpui::test]
    fn 無効なアイコンボタンは_tab_で到達できず押せない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, true);
        focus_next(cx);
        press_key(cx, "enter");
        cx.simulate_click(point(px(20.0), px(20.0)), Modifiers::none());
        assert_eq!(clicks(&view, cx), 0);
    }
}
