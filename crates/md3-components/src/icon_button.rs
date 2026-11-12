// 寸法と色の選択は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/IconButton.kt
//         (`IconButton`、`FilledIconButton`、`FilledTonalIconButton`、`OutlinedIconButton`、
//         `IconToggleButton`、`FilledIconToggleButton`、`FilledTonalIconToggleButton`、
//         `OutlinedIconToggleButton`、`IconToggleButtonColors`、`shapeForInteraction`)
//         compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/IconButtonDefaults.kt
//         (`IconButtonDefaults`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 のアイコンボタン

use gpui::{
    AnimationExt, App, ClickEvent, Div, ElementId, FocusHandle, Hsla, InteractiveElement,
    IntoElement, ParentElement, RenderOnce, SharedString, SpringAnimation,
    StatefulInteractiveElement, StyleRefinement, Styled, Window, div, prelude::FluentBuilder, svg,
};
use gpui_headless::button::Button as HeadlessButton;
use gpui_headless::toggle::ToggleButton as HeadlessToggleButton;

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
use md3_theme::md3_tokens::motion::MotionSchemeKey;
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
    /// 選ばれた toggle の、Round と Square それぞれの形
    selected_round: ShapeKey,
    selected_square: ShapeKey,
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
            selected_round: x_small_icon_button::SELECTED_CONTAINER_SHAPE_ROUND,
            selected_square: x_small_icon_button::SELECTED_CONTAINER_SHAPE_SQUARE,
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
            selected_round: small_icon_button::SELECTED_CONTAINER_SHAPE_ROUND,
            selected_square: small_icon_button::SELECTED_CONTAINER_SHAPE_SQUARE,
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
            selected_round: medium_icon_button::SELECTED_CONTAINER_SHAPE_ROUND,
            selected_square: medium_icon_button::SELECTED_CONTAINER_SHAPE_SQUARE,
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
            selected_round: large_icon_button::SELECTED_CONTAINER_SHAPE_ROUND,
            selected_square: large_icon_button::SELECTED_CONTAINER_SHAPE_SQUARE,
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
            selected_round: x_large_icon_button::SELECTED_CONTAINER_SHAPE_ROUND,
            selected_square: x_large_icon_button::SELECTED_CONTAINER_SHAPE_SQUARE,
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

    // Standard と Outlined は、`tokens` と同じ理由で `iconToggleButtonVibrantColors`・
    // `outlinedIconToggleButtonVibrantColors` に従う
    fn toggle_tokens(self) -> ToggleTokens {
        match self {
            // `defaultIconToggleButtonVibrantColors`
            Self::Standard => ToggleTokens {
                unselected: VariantTokens {
                    content: icon_button::UNSELECTED_COLOR,
                    ..self.tokens()
                },
                selected_container: None,
                selected_content: icon_button::SELECTED_COLOR,
            },
            // `defaultFilledIconToggleButtonColors`
            Self::Filled => ToggleTokens {
                unselected: VariantTokens {
                    container: Some(filled_icon_button::UNSELECTED_CONTAINER_COLOR),
                    content: filled_icon_button::UNSELECTED_COLOR,
                    ..self.tokens()
                },
                selected_container: Some(filled_icon_button::SELECTED_CONTAINER_COLOR),
                selected_content: filled_icon_button::SELECTED_COLOR,
            },
            // `defaultFilledTonalIconToggleButtonColors`
            Self::FilledTonal => ToggleTokens {
                unselected: VariantTokens {
                    container: Some(filled_tonal_icon_button::UNSELECTED_CONTAINER_COLOR),
                    content: filled_tonal_icon_button::UNSELECTED_COLOR,
                    ..self.tokens()
                },
                selected_container: Some(filled_tonal_icon_button::SELECTED_CONTAINER_COLOR),
                selected_content: filled_tonal_icon_button::SELECTED_COLOR,
            },
            // `defaultOutlinedIconToggleButtonVibrantColors`、`outlinedIconToggleButtonVibrantBorder`
            Self::Outlined => ToggleTokens {
                unselected: VariantTokens {
                    content: outlined_icon_button::UNSELECTED_COLOR,
                    ..self.tokens()
                },
                selected_container: Some(outlined_icon_button::SELECTED_CONTAINER_COLOR),
                selected_content: outlined_icon_button::SELECTED_COLOR,
            },
        }
    }
}

/// toggle のバリアントごとの色と枠線
struct ToggleTokens {
    /// 選ばれていないときの色と枠線、無効なときの色
    unselected: VariantTokens,
    /// 選ばれているときの容器の色。`None` は透明
    selected_container: Option<ColorRole>,
    /// 選ばれているときの内容の色
    selected_content: ColorRole,
}

/// アイコンボタンの見た目を決める値
struct Look {
    theme: SharedTheme,
    icon: StyledIcon,
    size: ButtonSize,
    width: IconButtonWidth,
    shape: ButtonShape,
    /// `None` は透明
    container: Option<Hsla>,
    content: Hsla,
    /// `None` は枠線なし
    outline: Option<Hsla>,
    disabled: bool,
    focus_visible: bool,
}

impl Look {
    /// 容器・アイコン・フォーカスリングを `button` に描き、`button` を容器の大きさにする
    ///
    /// `selected` を渡すと、容器とリングの形を選ばれた toggle の形へその割合だけ近づける。
    /// 0 ならボタンの形、1 なら選ばれた形で描く
    fn apply<E>(self, button: E, selected: Option<f32>) -> E
    where
        E: IntoElement + ParentElement + Styled + InteractiveElement,
    {
        let Self {
            theme,
            icon,
            size,
            width,
            shape,
            container,
            content,
            outline,
            disabled,
            focus_visible,
        } = self;
        let size = size_tokens(size);
        let width = size.icon_size.0
            + match width {
                IconButtonWidth::Narrow => size.narrow_space.0,
                IconButtonWidth::Uniform => size.uniform_space.0,
                IconButtonWidth::Wide => size.wide_space.0,
            };
        let (shape, selected_shape) = match shape {
            ButtonShape::Round => (size.round, size.selected_round),
            ButtonShape::Square => (size.square, size.selected_square),
        };
        let shaped = |inner: Div| {
            let shaped = Shaped::new(inner, theme.clone(), shape);
            match selected {
                Some(phase) => shaped.morph(selected_shape, phase),
                None => shaped,
            }
        };
        // 状態レイヤーは内容の色を重ねる (Compose の `ripple()` は LocalContentColor を使う)
        let state_layer = |opacity: f32| -> Hsla {
            let layer = content.opacity(opacity);
            match container {
                Some(container) => container.blend(layer),
                None => layer,
            }
        };

        // 角丸は描画時の大きさで決まるため、背景と枠線は Shaped で包んだ層に描き、ボタン自身は透明な箱にする
        let layer = div()
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .bottom_0()
            .when_some(container, |this, container| this.bg(container))
            .when_some(outline, |this, outline| {
                this.border(dp(size.outline_width)).border_color(outline)
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

        button
            .group(GROUP)
            .relative()
            .flex_grow_1()
            .flex()
            .items_center()
            .justify_center()
            .min_w(dp(Dp(width)))
            .min_h(dp(size.height))
            .child(shaped(layer))
            .child(
                svg()
                    .data(icon.svg().as_bytes())
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
                this.child(shaped(ring).outset(outset))
            })
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
        let outline = tokens.outline.map(|(role, disabled_opacity)| {
            let color = theme.color(role);
            if disabled {
                color.opacity(disabled_opacity)
            } else {
                color
            }
        });
        let look = Look {
            theme,
            icon: self.icon,
            size: self.size,
            width: self.width,
            shape: self.shape,
            container,
            content,
            outline,
            disabled,
            focus_visible,
        };

        let button = HeadlessButton::new(self.id)
            .track_focus(&focus_handle)
            .disabled(disabled)
            .aria_label(self.label)
            .when_some(self.on_click, |this, on_click| this.on_click(on_click));

        self.outer.child(look.apply(button, None))
    }
}

impl Styled for IconButton {
    fn style(&mut self) -> &mut StyleRefinement {
        self.outer.style()
    }
}

/// アイコンの toggle ボタンの色の上書き。`None` の項目はバリアントの色のまま
///
/// 色は役割で指定するため、テーマの明暗や配色が変わると追従する。
/// 状態レイヤーは内容の色から作る。枠線の色は変わらない
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct IconToggleButtonColors {
    /// 選ばれていない、有効なときの容器の色
    pub container: Option<ColorRole>,
    /// 選ばれていない、有効なときのアイコンの色
    pub content: Option<ColorRole>,
    /// 選ばれている、有効なときの容器の色
    pub selected_container: Option<ColorRole>,
    /// 選ばれている、有効なときのアイコンの色
    pub selected_content: Option<ColorRole>,
    /// 無効なときの容器の色と不透明度。選ばれているかどうかによらない
    pub disabled_container: Option<(ColorRole, f32)>,
    /// 無効なときのアイコンの色と不透明度。選ばれているかどうかによらない
    pub disabled_content: Option<(ColorRole, f32)>,
}

type ChangeHandler = Box<dyn Fn(&bool, &mut Window, &mut App)>;

/// 選ばれているかどうかを切り替える MD3 のアイコンボタン
///
/// アイコンだけを、選ばれているかどうかに応じたバリアントの色で描く。
/// 選ばれると容器の形が [`ButtonShape`] ごとの選ばれた形に変わる。形はテーマのバネで滑らかに変わる。
/// 選ばれている状態は呼び出し側が持ち、[`IconToggleButton::new`] に毎回渡す。ボタン自身は状態を変えない。
/// 支援技術には role `button` と押されているかどうかの状態として公開され、`label` がアクセシブルな名前になる。
/// マウスのクリック・フォーカス中の Enter と Space で [`IconToggleButton::on_change`] のハンドラを呼ぶ
///
/// [`Styled`] で指定したスタイルはボタンの外側の箱に付く。外側の余白や flex の指定に使い、
/// 箱を広げるとボタンも箱いっぱいに広がる。色や形は変えられないため、[`IconToggleButton::colors`] と
/// [`IconToggleButton::shape`] を使う
///
/// テーマの密度 ([`ThemeSettings::density`](md3_theme::ThemeSettings::density)) は反映しない。
/// 大きさは [`IconToggleButton::size`] と [`IconToggleButton::width`] で選ぶ
///
/// 押せる範囲はボタンの形を囲む矩形で、角丸の外側も含む。Extra small と Small でも 48dp には広げない
#[derive(IntoElement)]
pub struct IconToggleButton {
    id: ElementId,
    outer: Div,
    icon: StyledIcon,
    label: SharedString,
    selected: bool,
    variant: IconButtonVariant,
    colors: IconToggleButtonColors,
    size: ButtonSize,
    width: IconButtonWidth,
    shape: ButtonShape,
    on_change: Option<ChangeHandler>,
    disabled: bool,
    focus_handle: Option<FocusHandle>,
    theme: Option<SharedTheme>,
}

impl IconToggleButton {
    /// `icon` を描き、`label` をアクセシブルな名前にし、`selected` の状態を表示するアイコンボタンを作る
    ///
    /// [`IconButtonVariant::Standard`]、[`ButtonSize::Small`]、[`IconButtonWidth::Uniform`]、[`ButtonShape::Round`] で描く。
    /// `label` は状態によって変えないこと。選ばれているときに別のアイコンを描く場合は、`selected` に合わせて `icon` を選ぶ。
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。[`IconToggleButton::track_focus`] を指定しない場合、
    /// `id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(
        id: impl Into<ElementId>,
        icon: impl Into<StyledIcon>,
        label: impl Into<SharedString>,
        selected: bool,
    ) -> Self {
        Self {
            id: id.into(),
            // 利用者が Styled で指定した display を render で上書きしないよう、既定はここで付ける
            outer: div().flex(),
            icon: icon.into(),
            label: label.into(),
            selected,
            variant: IconButtonVariant::default(),
            colors: IconToggleButtonColors::default(),
            size: ButtonSize::default(),
            width: IconButtonWidth::default(),
            shape: ButtonShape::default(),
            on_change: None,
            disabled: false,
            focus_handle: None,
            theme: None,
        }
    }

    /// 操作されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には操作後に求められる状態として、現在の状態を反転した値を渡す。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`IconToggleButton::new`] に渡すこと
    pub fn on_change(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// アイコンボタンの種類を設定する
    pub fn variant(mut self, variant: IconButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// バリアントの色を `colors` で上書きする
    pub fn colors(mut self, colors: IconToggleButtonColors) -> Self {
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

    /// 選ばれていないときの形を設定する。選ばれたときの形は、Round では角の小さい形、Square では丸い形になる
    pub fn shape(mut self, shape: ButtonShape) -> Self {
        self.shape = shape;
        self
    }

    /// アイコンボタンを無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効なアイコンボタンは、選ばれているかどうかによらず無効の色で描かれ、押されてもハンドラを呼ばず、
    /// Tab でも到達できない。フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
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

impl RenderOnce for IconToggleButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let tokens = self.variant.toggle_tokens();
        let colors = self.colors;
        let disabled = self.disabled;
        let selected = self.selected;

        let focus_handle = self
            .focus_handle
            .unwrap_or_else(|| util::keyed_focus_handle(&self.id, window, cx));
        // リングはボタンの外側に描くため、ボタン自身のスタイルでは表せない
        let focus_visible = gpui_headless::is_focus_visible(&focus_handle, window, cx);

        // Compose の `IconToggleButtonColors` と同じく、無効なときは選ばれているかどうかによらず無効の色にする
        let (container, content) = if disabled {
            let (role, opacity) = colors
                .disabled_content
                .unwrap_or(tokens.unselected.disabled_content);
            (
                colors
                    .disabled_container
                    .or(tokens.unselected.disabled_container)
                    .map(|(role, opacity)| theme.color(role).opacity(opacity)),
                theme.color(role).opacity(opacity),
            )
        } else if selected {
            (
                colors
                    .selected_container
                    .or(tokens.selected_container)
                    .map(|role| theme.color(role)),
                theme.color(colors.selected_content.unwrap_or(tokens.selected_content)),
            )
        } else {
            (
                colors
                    .container
                    .or(tokens.unselected.container)
                    .map(|role| theme.color(role)),
                theme.color(colors.content.unwrap_or(tokens.unselected.content)),
            )
        };
        // 枠線は選ばれていないときだけ描く (`outlinedIconToggleButtonVibrantBorder`)
        let outline =
            tokens
                .unselected
                .outline
                .filter(|_| !selected)
                .map(|(role, disabled_opacity)| {
                    let color = theme.color(role);
                    if disabled {
                        color.opacity(disabled_opacity)
                    } else {
                        color
                    }
                });
        // Compose は形を DefaultEffects のバネで変える。跳ねずに収まる
        let spring = theme.spring(MotionSchemeKey::DefaultEffects);
        let look = Look {
            theme,
            icon: self.icon,
            size: self.size,
            width: self.width,
            shape: self.shape,
            container,
            content,
            outline,
            disabled,
            focus_visible,
        };

        let button = HeadlessToggleButton::new(self.id.clone(), selected)
            .track_focus(&focus_handle)
            .disabled(disabled)
            .aria_label(self.label)
            .when_some(self.on_change, |this, on_change| this.on_change(on_change));

        self.outer.child(button.with_spring(
            self.id,
            SpringAnimation::new(spring).to(selected),
            move |button, phase| look.apply(button, Some(phase.0)),
        ))
    }
}

impl Styled for IconToggleButton {
    fn style(&mut self) -> &mut StyleRefinement {
        self.outer.style()
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use gpui::{
        AnyElement, Context, Entity, Modifiers, Render, TestAppContext, VisualTestContext, point,
        px,
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

    /// 選ばれている状態を持ち、操作で求められた状態をそのまま反映する
    struct ToggleView {
        selected: bool,
        variant: IconButtonVariant,
        shape: ButtonShape,
        disabled: bool,
        changes: Vec<bool>,
    }

    impl Render for ToggleView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div().flex().items_start().child(
                IconToggleButton::new("toggle", md3_icons::FAVORITE, "Favorite", self.selected)
                    .variant(self.variant)
                    .shape(self.shape)
                    .disabled(self.disabled)
                    .on_change(cx.listener(|this, selected: &bool, _, cx| {
                        this.changes.push(*selected);
                        this.selected = *selected;
                        cx.notify();
                    })),
            )
        }
    }

    fn add_toggle(
        cx: &mut TestAppContext,
        init: impl FnOnce(&mut ToggleView),
    ) -> (Entity<ToggleView>, &mut VisualTestContext) {
        cx.update(crate::init);
        let mut view = ToggleView {
            selected: false,
            variant: IconButtonVariant::Filled,
            shape: ButtonShape::Round,
            disabled: false,
            changes: Vec::new(),
        };
        init(&mut view);
        cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            view
        })
    }

    /// 最初に描いた矩形 (容器) の左上の角丸の半径と塗りと枠線の太さ (論理ピクセル)。
    /// 状態レイヤーを描かないよう、マウスをボタンの外に置いてから測る
    fn container(cx: &mut VisualTestContext) -> (f32, gpui::Background, f32) {
        cx.simulate_mouse_move(point(px(300.), px(300.)), None, Modifiers::none());
        cx.update(|window, _| {
            let scale = window.scale_factor();
            let quad = window.painted_quads()[0];
            (
                quad.corner_radii.top_left.0 / scale,
                quad.background,
                quad.border_widths.top.0 / scale,
            )
        })
    }

    fn color(cx: &mut VisualTestContext, role: ColorRole, opacity: f32) -> gpui::Background {
        cx.update(|_, cx| cx.md3_theme().color(role).opacity(opacity).into())
    }

    #[gpui::test]
    fn toggle_は押すと反転した状態を求め_無効なら求めない(
        cx: &mut TestAppContext,
    ) {
        for (selected, disabled, expected) in [
            (false, false, vec![true]),
            (true, false, vec![false]),
            (false, true, vec![]),
        ] {
            let (view, cx) = add_toggle(cx, |view| {
                view.selected = selected;
                view.disabled = disabled;
            });
            cx.simulate_click(point(px(20.), px(20.)), Modifiers::none());
            let changes = view.read_with(cx, |view, _| view.changes.clone());
            assert_eq!(changes, expected, "selected={selected} disabled={disabled}");
        }
    }

    /// 期待値は Compose の `FilledIconButtonTokens` (UnselectedContainerColor、SelectedContainerColor、
    /// DisabledContainerColor と DisabledContainerOpacity) の値
    #[gpui::test]
    fn filled_の_toggle_は選ばれると容器の色が変わり_無効なら無効の色になる(
        cx: &mut TestAppContext,
    ) {
        for (selected, disabled, role, opacity) in [
            (false, false, ColorRole::SurfaceContainer, 1.0),
            (true, false, ColorRole::Primary, 1.0),
            (true, true, ColorRole::OnSurface, 0.1),
        ] {
            let (_, cx) = add_toggle(cx, |view| {
                view.selected = selected;
                view.disabled = disabled;
            });
            let expected = color(cx, role, opacity);
            assert_eq!(
                container(cx).1,
                expected,
                "selected={selected} disabled={disabled}"
            );
        }
    }

    /// 期待値は Compose の `OutlinedIconButtonTokens` (SelectedContainerColor) と、
    /// `SmallIconButtonTokens.OutlinedOutlineWidth` の 1dp
    #[gpui::test]
    fn outlined_の_toggle_は選ばれると枠線を消して容器を塗る(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_toggle(cx, |view| view.variant = IconButtonVariant::Outlined);
        let (_, _, border) = container(cx);
        assert_eq!(border, 1.0, "選ばれていなければ枠線を描くこと");

        let (_, cx) = add_toggle(cx, |view| {
            view.variant = IconButtonVariant::Outlined;
            view.selected = true;
        });
        let expected = color(cx, ColorRole::InverseSurface, 1.0);
        let (_, background, border) = container(cx);
        assert_eq!((background, border), (expected, 0.0));
    }

    /// 期待値は 40×40 の容器で、CornerFull は短辺の半分の 20、`SmallIconButtonTokens` の選ばれた形
    /// (Round は CornerMedium、Square は CornerFull) と Square の形 (CornerMedium) の 12dp
    #[gpui::test]
    fn 初めから選ばれていれば選ばれた形で描く(cx: &mut TestAppContext) {
        for (shape, selected, expected) in [
            (ButtonShape::Round, false, 20.0),
            (ButtonShape::Round, true, 12.0),
            (ButtonShape::Square, false, 12.0),
            (ButtonShape::Square, true, 20.0),
        ] {
            let (_, cx) = add_toggle(cx, |view| {
                view.shape = shape;
                view.selected = selected;
            });
            assert_eq!(container(cx).0, expected, "{shape:?} selected={selected}");
        }
    }

    // gpui のバネは実時間 (`Instant::now`) で進み、テストの時計では進められないため、
    // 途中の形は確かめず、十分な時間の後に着くことだけを確かめる
    #[gpui::test]
    fn 選ばれると選ばれた形まで変わる(cx: &mut TestAppContext) {
        let (view, cx) = add_toggle(cx, |_| {});
        cx.simulate_click(point(px(20.), px(20.)), Modifiers::none());
        cx.run_until_parked();
        // DefaultEffects のバネは 1 秒以内に収まる
        std::thread::sleep(std::time::Duration::from_secs(1));
        view.update(cx, |_, cx| cx.notify());
        cx.run_until_parked();
        assert_eq!(container(cx).0, 12.0);
    }

    #[gpui::test]
    fn モーションを減らす設定では選ばれた形をすぐに描く(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_toggle(cx, |_| {});
        cx.update(|_, cx| cx.set_reduce_motion(true));
        cx.simulate_click(point(px(20.), px(20.)), Modifiers::none());
        cx.run_until_parked();
        assert_eq!(container(cx).0, 12.0);
    }
}
