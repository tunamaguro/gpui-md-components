// 寸法と色の選択は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/IconButton.kt
//         (`IconButton`、`FilledIconButton`、`FilledTonalIconButton`、`OutlinedIconButton`)
//         compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/IconButtonDefaults.kt
//         (`IconButtonDefaults`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 のアイコンボタン

use gpui::{
    App, ClickEvent, Div, ElementId, FocusHandle, Hsla, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, StyleRefinement, Styled,
    Window, div, prelude::FluentBuilder, svg,
};
use gpui_headless::button::Button as HeadlessButton;

use crate::shape::Shaped;
use crate::util;
use crate::{ButtonColors, ButtonShape, ButtonSize};
use md3_icons::StyledIcon;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::{
    filled_icon_button, filled_tonal_icon_button, icon_button, large_icon_button,
    medium_icon_button, outlined_icon_button, small_icon_button, x_large_icon_button,
    x_small_icon_button,
};
use md3_theme::md3_tokens::shape::ShapeKey;
use md3_theme::md3_tokens::{Dp, focus_ring, state};
use md3_theme::{SharedTheme, ThemeExt, dp};

/// 容器の層がボタンの hover を知るための group 名
const GROUP: &str = "md3-icon-button";

/// アイコンボタンの種類。強調の度合いが高い順に Filled、FilledTonal、Outlined、Standard
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum IconButtonVariant {
    /// 容器を持たないアイコンボタン。ツールバーなど、まとめて並べる補助的な操作に使う
    #[default]
    Standard,
    /// 塗りつぶしのアイコンボタン。最も強調したい操作に使う
    Filled,
    /// 二次的な色で塗りつぶしたアイコンボタン。Filled と Outlined の中間の強調に使う
    FilledTonal,
    /// 枠線のアイコンボタン。背景との区別が要る、強調の中程度の操作に使う
    Outlined,
}

/// アイコンボタンの幅。高さは [`ButtonSize`] で決まり、幅はアイコンの左右の余白で変わる
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum IconButtonWidth {
    /// 左右の余白を狭くした幅
    Narrow,
    /// 既定の幅。Small 以下では高さと同じになる
    #[default]
    Uniform,
    /// 左右の余白を広くした幅
    Wide,
}

/// サイズごとの寸法
struct SizeTokens {
    height: Dp,
    icon_size: Dp,
    /// 幅ごとの、アイコンの左右の余白の合計
    narrow_space: Dp,
    uniform_space: Dp,
    wide_space: Dp,
    outline_width: Dp,
    round: ShapeKey,
    square: ShapeKey,
}

// 余白は `IconButtonDefaults.smallContainerSize` などが幅ごとに足す値。
// Uniform は Compose と同じく左の余白を 2 回足し、Large だけは Default の代わりに Uniform の余白を使う。
// 枠線の太さは、Compose の `outlinedIconButtonBorder` はどのサイズにも Small の値を使うが、サイズごとのトークンに従う
fn size_tokens(size: ButtonSize) -> SizeTokens {
    match size {
        ButtonSize::ExtraSmall => SizeTokens {
            height: x_small_icon_button::CONTAINER_HEIGHT,
            icon_size: x_small_icon_button::ICON_SIZE,
            narrow_space: Dp(x_small_icon_button::NARROW_LEADING_SPACE.0
                + x_small_icon_button::NARROW_TRAILING_SPACE.0),
            uniform_space: Dp(x_small_icon_button::DEFAULT_LEADING_SPACE.0 * 2.0),
            wide_space: Dp(x_small_icon_button::WIDE_LEADING_SPACE.0
                + x_small_icon_button::WIDE_TRAILING_SPACE.0),
            outline_width: x_small_icon_button::OUTLINED_OUTLINE_WIDTH,
            round: x_small_icon_button::CONTAINER_SHAPE_ROUND,
            square: x_small_icon_button::CONTAINER_SHAPE_SQUARE,
        },
        ButtonSize::Small => SizeTokens {
            height: small_icon_button::CONTAINER_HEIGHT,
            icon_size: small_icon_button::ICON_SIZE,
            narrow_space: Dp(small_icon_button::NARROW_LEADING_SPACE.0
                + small_icon_button::NARROW_TRAILING_SPACE.0),
            uniform_space: Dp(small_icon_button::DEFAULT_LEADING_SPACE.0 * 2.0),
            wide_space: Dp(
                small_icon_button::WIDE_LEADING_SPACE.0 + small_icon_button::WIDE_TRAILING_SPACE.0
            ),
            outline_width: small_icon_button::OUTLINED_OUTLINE_WIDTH,
            round: small_icon_button::CONTAINER_SHAPE_ROUND,
            square: small_icon_button::CONTAINER_SHAPE_SQUARE,
        },
        ButtonSize::Medium => SizeTokens {
            height: medium_icon_button::CONTAINER_HEIGHT,
            icon_size: medium_icon_button::ICON_SIZE,
            narrow_space: Dp(medium_icon_button::NARROW_LEADING_SPACE.0
                + medium_icon_button::NARROW_TRAILING_SPACE.0),
            uniform_space: Dp(medium_icon_button::DEFAULT_LEADING_SPACE.0 * 2.0),
            wide_space: Dp(medium_icon_button::WIDE_LEADING_SPACE.0
                + medium_icon_button::WIDE_TRAILING_SPACE.0),
            outline_width: medium_icon_button::OUTLINED_OUTLINE_WIDTH,
            round: medium_icon_button::CONTAINER_SHAPE_ROUND,
            square: medium_icon_button::CONTAINER_SHAPE_SQUARE,
        },
        ButtonSize::Large => SizeTokens {
            height: large_icon_button::CONTAINER_HEIGHT,
            icon_size: large_icon_button::ICON_SIZE,
            narrow_space: Dp(large_icon_button::NARROW_LEADING_SPACE.0
                + large_icon_button::NARROW_TRAILING_SPACE.0),
            uniform_space: Dp(large_icon_button::UNIFORM_LEADING_SPACE.0 * 2.0),
            wide_space: Dp(
                large_icon_button::WIDE_LEADING_SPACE.0 + large_icon_button::WIDE_TRAILING_SPACE.0
            ),
            outline_width: large_icon_button::OUTLINED_OUTLINE_WIDTH,
            round: large_icon_button::CONTAINER_SHAPE_ROUND,
            square: large_icon_button::CONTAINER_SHAPE_SQUARE,
        },
        ButtonSize::ExtraLarge => SizeTokens {
            height: x_large_icon_button::CONTAINER_HEIGHT,
            icon_size: x_large_icon_button::ICON_SIZE,
            narrow_space: Dp(x_large_icon_button::NARROW_LEADING_SPACE.0
                + x_large_icon_button::NARROW_TRAILING_SPACE.0),
            uniform_space: Dp(x_large_icon_button::DEFAULT_LEADING_SPACE.0 * 2.0),
            wide_space: Dp(x_large_icon_button::WIDE_LEADING_SPACE.0
                + x_large_icon_button::WIDE_TRAILING_SPACE.0),
            outline_width: x_large_icon_button::OUTLINED_OUTLINE_WIDTH,
            round: x_large_icon_button::CONTAINER_SHAPE_ROUND,
            square: x_large_icon_button::CONTAINER_SHAPE_SQUARE,
        },
    }
}

/// バリアントごとの色と枠線
struct VariantTokens {
    /// `None` は透明
    container: Option<ColorRole>,
    content: ColorRole,
    /// `None` は透明
    disabled_container: Option<(ColorRole, f32)>,
    disabled_content: (ColorRole, f32),
    /// 枠線の色と、無効時の枠線の不透明度
    outline: Option<(ColorRole, f32)>,
}

impl IconButtonVariant {
    // Standard と Outlined の内容の色は、Compose の既定 (`iconButtonColors` など) では親から受け継いだ
    // LocalContentColor になる。gpui の要素には色を役割で受け継ぐ仕組みが無いため、トークンの色を使う
    // `iconButtonVibrantColors`・`outlinedIconButtonVibrantColors`・`outlinedIconButtonVibrantBorder` に従う
    fn tokens(self) -> VariantTokens {
        match self {
            // `defaultIconButtonVibrantColors`
            Self::Standard => VariantTokens {
                container: None,
                content: icon_button::COLOR,
                disabled_container: None,
                disabled_content: (icon_button::DISABLED_COLOR, icon_button::DISABLED_OPACITY),
                outline: None,
            },
            // `defaultFilledIconButtonColors`
            Self::Filled => VariantTokens {
                container: Some(filled_icon_button::CONTAINER_COLOR),
                content: filled_icon_button::COLOR,
                disabled_container: Some((
                    filled_icon_button::DISABLED_CONTAINER_COLOR,
                    filled_icon_button::DISABLED_CONTAINER_OPACITY,
                )),
                disabled_content: (
                    filled_icon_button::DISABLED_COLOR,
                    filled_icon_button::DISABLED_OPACITY,
                ),
                outline: None,
            },
            // `defaultFilledTonalIconButtonColors`
            Self::FilledTonal => VariantTokens {
                container: Some(filled_tonal_icon_button::CONTAINER_COLOR),
                content: filled_tonal_icon_button::COLOR,
                disabled_container: Some((
                    filled_tonal_icon_button::DISABLED_CONTAINER_COLOR,
                    filled_tonal_icon_button::DISABLED_CONTAINER_OPACITY,
                )),
                disabled_content: (
                    filled_tonal_icon_button::DISABLED_COLOR,
                    filled_tonal_icon_button::DISABLED_OPACITY,
                ),
                outline: None,
            },
            // `defaultOutlinedIconButtonVibrantColors`、`outlinedIconButtonVibrantBorder`。
            // 無効時の枠線は、`DisabledOutlineColor` ではなく `OutlineColor` に内容と同じ不透明度を掛ける
            Self::Outlined => VariantTokens {
                container: None,
                content: outlined_icon_button::COLOR,
                disabled_container: None,
                disabled_content: (
                    outlined_icon_button::DISABLED_COLOR,
                    outlined_icon_button::DISABLED_OPACITY,
                ),
                outline: Some((
                    outlined_icon_button::OUTLINE_COLOR,
                    outlined_icon_button::DISABLED_OPACITY,
                )),
            },
        }
    }
}

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// MD3 のアイコンボタン
///
/// アイコンだけをバリアントの内容の色で描く。
/// 支援技術には role `button` として公開され、[`IconButton::new`] の `label` がアクセシブルな名前になる。
/// マウスのクリック・フォーカス中の Enter と Space で [`IconButton::on_click`] のハンドラを呼ぶ。
///
/// [`Styled`] で指定したスタイルはボタンの外側の箱に付く。外側の余白や flex の指定に使い、
/// 箱を広げるとボタンも箱いっぱいに広がる。色や形は変えられないため、[`IconButton::colors`] と [`IconButton::shape`] を使う
///
/// テーマの密度 ([`ThemeSettings::density`](md3_theme::ThemeSettings::density)) は反映しない。
/// 大きさは [`IconButton::size`] と [`IconButton::width`] で選ぶ
///
/// 押せる範囲はボタンの形を囲む矩形で、角丸の外側も含む。Extra small と Small でも 48dp には広げない
#[derive(IntoElement)]
pub struct IconButton {
    id: ElementId,
    outer: Div,
    icon: StyledIcon,
    label: SharedString,
    variant: IconButtonVariant,
    colors: ButtonColors,
    size: ButtonSize,
    width: IconButtonWidth,
    shape: ButtonShape,
    on_click: Option<ClickHandler>,
    disabled: bool,
    focus_handle: Option<FocusHandle>,
    theme: Option<SharedTheme>,
}

impl IconButton {
    /// `icon` を描き、`label` をアクセシブルな名前にしたアイコンボタンを作る
    ///
    /// [`IconButtonVariant::Standard`]、[`ButtonSize::Small`]、[`IconButtonWidth::Uniform`]、[`ButtonShape::Round`] で描く。
    /// [`md3_icons::Icon`] を渡すと Outlined の塗りのない形で描く。スタイルや塗りは [`StyledIcon`] で指定する。
    /// `label` はボタンが何をするかを表す文字列にする。画面には表示しない。
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。[`IconButton::track_focus`] を指定しない場合、
    /// `id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(
        id: impl Into<ElementId>,
        icon: impl Into<StyledIcon>,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            // 利用者が Styled で指定した display を render で上書きしないよう、既定はここで付ける
            outer: div().flex(),
            icon: icon.into(),
            label: label.into(),
            variant: IconButtonVariant::default(),
            colors: ButtonColors::default(),
            size: ButtonSize::default(),
            width: IconButtonWidth::default(),
            shape: ButtonShape::default(),
            on_click: None,
            disabled: false,
            focus_handle: None,
            theme: None,
        }
    }

    /// アイコンボタンの種類を設定する
    pub fn variant(mut self, variant: IconButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// バリアントの色を `colors` で上書きする
    pub fn colors(mut self, colors: ButtonColors) -> Self {
        self.colors = colors;
        self
    }

    /// アイコンボタンの高さとアイコンの大きさを設定する
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// アイコンボタンの幅を設定する
    pub fn width(mut self, width: IconButtonWidth) -> Self {
        self.width = width;
        self
    }

    /// アイコンボタンの形を設定する
    pub fn shape(mut self, shape: ButtonShape) -> Self {
        self.shape = shape;
        self
    }

    /// 押されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// アイコンボタンを無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効なアイコンボタンは無効の色で描かれ、押されてもハンドラを呼ばず、Tab でも到達できない。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// `focus_handle` をこのアイコンボタンのフォーカスとして使う
    ///
    /// 指定しない場合はアイコンボタンが内部でフォーカスを持つ
    pub fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle.clone());
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl RenderOnce for IconButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let mut tokens = self.variant.tokens();
        tokens.container = self.colors.container.or(tokens.container);
        tokens.content = self.colors.content.unwrap_or(tokens.content);
        tokens.disabled_container = self.colors.disabled_container.or(tokens.disabled_container);
        tokens.disabled_content = self
            .colors
            .disabled_content
            .unwrap_or(tokens.disabled_content);
        let size = size_tokens(self.size);
        let width = size.icon_size.0
            + match self.width {
                IconButtonWidth::Narrow => size.narrow_space.0,
                IconButtonWidth::Uniform => size.uniform_space.0,
                IconButtonWidth::Wide => size.wide_space.0,
            };
        let disabled = self.disabled;

        let focus_handle = self
            .focus_handle
            .unwrap_or_else(|| util::keyed_focus_handle(&self.id, window, cx));
        // リングはボタンの外側に描くため、ボタン自身のスタイルでは表せない
        let focus_visible = gpui_headless::is_focus_visible(&focus_handle, window, cx);

        let (container, content) = if disabled {
            (
                tokens
                    .disabled_container
                    .map(|(role, opacity)| theme.color(role).opacity(opacity)),
                theme
                    .color(tokens.disabled_content.0)
                    .opacity(tokens.disabled_content.1),
            )
        } else {
            (
                tokens.container.map(|role| theme.color(role)),
                theme.color(tokens.content),
            )
        };
        // 状態レイヤーは内容の色を重ねる (Compose の `ripple()` は LocalContentColor を使う)
        let state_layer = |opacity: f32| -> Hsla {
            let layer = content.opacity(opacity);
            match container {
                Some(container) => container.blend(layer),
                None => layer,
            }
        };
        let shape = match self.shape {
            ButtonShape::Round => size.round,
            ButtonShape::Square => size.square,
        };

        // 角丸は描画時の大きさで決まるため、背景と枠線は Shaped で包んだ層に描き、ボタン自身は透明な箱にする
        let container = div()
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .bottom_0()
            .when_some(container, |this, container| this.bg(container))
            .when_some(tokens.outline, |this, (role, disabled_opacity)| {
                let color = theme.color(role);
                this.border(dp(size.outline_width))
                    .border_color(if disabled {
                        color.opacity(disabled_opacity)
                    } else {
                        color
                    })
            })
            .when(!disabled, |this| {
                let this = this.when(focus_visible, |this| {
                    this.bg(state_layer(state::FOCUS_STATE_LAYER_OPACITY))
                });
                // gpui は group_hover を自身のスタイルの後に重ねるため、hover がフォーカスより優先される
                this.group_hover(GROUP, |style| {
                    style.bg(state_layer(state::HOVER_STATE_LAYER_OPACITY))
                })
            });

        let button = HeadlessButton::new(self.id)
            .track_focus(&focus_handle)
            .disabled(disabled)
            .group(GROUP)
            .relative()
            .flex_grow_1()
            .flex()
            .items_center()
            .justify_center()
            .min_w(dp(Dp(width)))
            .min_h(dp(size.height))
            .child(Shaped::new(container, theme.clone(), shape))
            .child(
                svg()
                    .data(self.icon.svg().as_bytes())
                    .flex_none()
                    .size(dp(size.icon_size))
                    .text_color(content),
            )
            .when(focus_visible, |this| {
                let outset = dp(focus_ring::OUTWARD_OFFSET) + dp(focus_ring::WIDTH);
                let ring = div()
                    .absolute()
                    .top(-outset)
                    .left(-outset)
                    .right(-outset)
                    .bottom(-outset)
                    .border(dp(focus_ring::WIDTH))
                    .border_color(theme.color(focus_ring::COLOR));
                this.child(Shaped::new(ring, theme.clone(), shape).outset(outset))
            })
            .aria_label(self.label)
            .when_some(self.on_click, |this, on_click| this.on_click(on_click));

        self.outer.child(button)
    }
}

impl Styled for IconButton {
    fn style(&mut self) -> &mut StyleRefinement {
        self.outer.style()
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use gpui::{
        AnyElement, Context, Modifiers, Render, TestAppContext, VisualTestContext, point, px,
    };

    use super::*;

    struct ElementView(Box<dyn Fn() -> AnyElement>);

    impl Render for ElementView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            // ルートの要素はウィンドウいっぱいに広がるため、ボタンを伸ばさない箱に包む
            div().flex().items_start().child((self.0)())
        }
    }

    fn add_view(
        cx: &mut TestAppContext,
        element: impl Fn() -> AnyElement + 'static,
    ) -> &mut VisualTestContext {
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            ElementView(Box::new(element))
        });
        cx
    }

    /// 描いた矩形の左上の座標と幅と高さ (論理ピクセル)
    fn quad_rects(cx: &mut VisualTestContext) -> Vec<(f32, f32, f32, f32)> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            window
                .painted_quads()
                .into_iter()
                .map(|quad| {
                    let b = quad.bounds;
                    (
                        b.origin.x.0 / scale,
                        b.origin.y.0 / scale,
                        b.size.width.0 / scale,
                        b.size.height.0 / scale,
                    )
                })
                .collect()
        })
    }

    /// 期待値は Compose の `IconButtonDefaults.smallContainerSize` などの式 (アイコンの大きさ + 左右の余白) に、
    /// 各サイズの IconButtonTokens の値を手で代入して求めた
    #[gpui::test]
    fn 容器の大きさがサイズと幅に従う(cx: &mut TestAppContext) {
        for (size, width, expected) in [
            (ButtonSize::ExtraSmall, IconButtonWidth::Uniform, (32., 32.)),
            (ButtonSize::Small, IconButtonWidth::Narrow, (32., 40.)),
            (ButtonSize::Small, IconButtonWidth::Uniform, (40., 40.)),
            (ButtonSize::Small, IconButtonWidth::Wide, (52., 40.)),
            (ButtonSize::Medium, IconButtonWidth::Uniform, (56., 56.)),
            (ButtonSize::Large, IconButtonWidth::Narrow, (64., 96.)),
            (ButtonSize::Large, IconButtonWidth::Uniform, (96., 96.)),
            (ButtonSize::ExtraLarge, IconButtonWidth::Wide, (184., 136.)),
        ] {
            let cx = add_view(cx, move || {
                IconButton::new("icon-button", md3_icons::ADD, "Add")
                    .variant(IconButtonVariant::Filled)
                    .size(size)
                    .width(width)
                    .into_any_element()
            });
            assert_eq!(
                quad_rects(cx),
                vec![(0., 0., expected.0, expected.1)],
                "{size:?} {width:?} の容器を {expected:?} で描くこと"
            );
        }
    }

    #[gpui::test]
    fn standard_は容器を描かない(cx: &mut TestAppContext) {
        let cx = add_view(cx, || {
            IconButton::new("icon-button", md3_icons::ADD, "Add").into_any_element()
        });
        cx.simulate_mouse_move(point(px(300.), px(300.)), None, Modifiers::none());
        assert_eq!(quad_rects(cx), vec![]);
    }

    #[gpui::test]
    fn hover_中は容器の範囲に状態レイヤーを描く(cx: &mut TestAppContext) {
        let cx = add_view(cx, || {
            IconButton::new("icon-button", md3_icons::ADD, "Add").into_any_element()
        });
        let layer = (0., 0., 40., 40.);
        cx.simulate_mouse_move(point(px(20.), px(20.)), None, Modifiers::none());
        assert!(quad_rects(cx).contains(&layer), "ボタンに乗ると描くこと");
        cx.simulate_mouse_move(point(px(300.), px(300.)), None, Modifiers::none());
        assert!(
            !quad_rects(cx).contains(&layer),
            "ボタンから外れると消すこと"
        );
    }

    #[gpui::test]
    fn tab_のフォーカスを容器の外側のリングで示す(cx: &mut TestAppContext) {
        let cx = add_view(cx, || {
            IconButton::new("icon-button", md3_icons::ADD, "Add").into_any_element()
        });
        cx.update(|window, _| window.activate_window());
        // リングは 40×40px の容器の外側 5px に描く
        let ring = (-5., -5., 50., 50.);
        assert!(
            !quad_rects(cx).contains(&ring),
            "フォーカスの前はリングを描かないこと"
        );
        cx.simulate_keystrokes("tab");
        assert!(
            quad_rects(cx).contains(&ring),
            "フォーカスが移るとリングを描くこと"
        );
    }

    #[gpui::test]
    fn 押すとハンドラを呼び_無効なら呼ばない(cx: &mut TestAppContext) {
        for (disabled, expected) in [(false, 1), (true, 0)] {
            let clicks = Rc::new(Cell::new(0));
            let cx = add_view(cx, {
                let clicks = clicks.clone();
                move || {
                    let clicks = clicks.clone();
                    IconButton::new("icon-button", md3_icons::ADD, "Add")
                        .disabled(disabled)
                        .on_click(move |_, _, _| clicks.set(clicks.get() + 1))
                        .into_any_element()
                }
            });
            cx.simulate_click(point(px(20.), px(20.)), Modifiers::none());
            assert_eq!(clicks.get(), expected, "disabled={disabled}");
        }
    }
}
