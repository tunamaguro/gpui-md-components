// 寸法・色の選択・状態ごとの elevation は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Button.kt
//         (`Button`、`ButtonDefaults`、`ButtonElevation`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 のボタン

use gpui::{
    AnyElement, App, ClickEvent, Corners, Div, ElementId, FocusHandle, Hsla, InteractiveElement,
    IntoElement, ParentElement, Pixels, Refineable, RenderOnce, StatefulInteractiveElement,
    StyleRefinement, Styled, Window, div, prelude::FluentBuilder, px, size, svg,
};
use gpui_headless::button::Button as HeadlessButton;
use md3_icons::{Icon, IconStyle};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::{
    button_large, button_medium, button_small, button_x_large, button_x_small, elevated_button,
    filled_button, filled_tonal_button, outlined_button, text_button,
};
use md3_theme::md3_tokens::elevation::Elevation;
use md3_theme::md3_tokens::shape::ShapeKey;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::md3_tokens::{Dp, focus_ring, state};
use md3_theme::{SharedTheme, Theme, ThemeExt, dp};

/// `ButtonDefaults.MinWidth`。どのサイズにも使う
const MIN_WIDTH: Dp = Dp(58.0);

/// ボタンの種類。強調の度合いが高い順に Filled、FilledTonal、Elevated、Outlined、Text
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ButtonVariant {
    /// 塗りつぶしのボタン。保存や確定など、流れを完了させる重要な操作に使う
    #[default]
    Filled,
    /// 影で浮かせたボタン。背景と区別する必要がある場合に使う
    Elevated,
    /// 二次的な色で塗りつぶしたボタン。Filled と Outlined の中間の強調に使う
    FilledTonal,
    /// 枠線のボタン。重要だが主ではない操作に使う
    Outlined,
    /// 文字だけのボタン。強調の低い操作に使う
    Text,
}

/// ボタンの大きさ。高さ・余白・アイコン・ラベルの書体が大きさに合わせて決まる
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
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

/// サイズごとの寸法と書体
struct SizeTokens {
    height: Dp,
    /// 左右の余白。アイコンの有無で変わらない
    horizontal_padding: Dp,
    vertical_padding: Dp,
    icon_size: Dp,
    icon_spacing: Dp,
    label: TypeScale,
    outline_width: Dp,
    shape: ShapeKey,
}

impl ButtonSize {
    // 余白は `ButtonDefaults.contentPaddingFor`、アイコンは `iconSizeFor` と `iconSpacingFor`、
    // 書体は `textStyleFor` が高さから選ぶ値。精密なポインタ向けの値は Compose でも既定で使われないため採らない。
    // 枠線の太さは、Compose の `outlinedButtonBorder` はどのサイズにも Small の値を使うが、サイズごとのトークンに従う
    fn tokens(self) -> SizeTokens {
        match self {
            // `ExtraSmallContentPadding` と `ExtraSmallIconSpacing` は、Compose が TODO 付きで
            // ButtonXSmallTokens (余白 16dp、間隔 8dp) と異なる値を使っている
            Self::ExtraSmall => SizeTokens {
                height: button_x_small::CONTAINER_HEIGHT,
                horizontal_padding: Dp(12.0),
                vertical_padding: Dp(6.0),
                icon_size: button_x_small::ICON_SIZE,
                icon_spacing: Dp(4.0),
                label: TypeScale::LabelLarge,
                outline_width: button_x_small::OUTLINED_OUTLINE_WIDTH,
                shape: button_x_small::CONTAINER_SHAPE_ROUND,
            },
            Self::Small => SizeTokens {
                height: button_small::CONTAINER_HEIGHT,
                horizontal_padding: button_small::LEADING_SPACE,
                vertical_padding: Dp(10.0),
                icon_size: button_small::ICON_SIZE,
                icon_spacing: button_small::ICON_LABEL_SPACE,
                label: TypeScale::LabelLarge,
                outline_width: button_small::OUTLINED_OUTLINE_WIDTH,
                shape: button_small::CONTAINER_SHAPE_ROUND,
            },
            Self::Medium => SizeTokens {
                height: button_medium::CONTAINER_HEIGHT,
                horizontal_padding: button_medium::LEADING_SPACE,
                vertical_padding: Dp(16.0),
                icon_size: button_medium::ICON_SIZE,
                icon_spacing: button_medium::ICON_LABEL_SPACE,
                label: TypeScale::TitleMedium,
                outline_width: button_medium::OUTLINED_OUTLINE_WIDTH,
                shape: button_medium::CONTAINER_SHAPE_ROUND,
            },
            Self::Large => SizeTokens {
                height: button_large::CONTAINER_HEIGHT,
                horizontal_padding: button_large::LEADING_SPACE,
                vertical_padding: Dp(32.0),
                icon_size: button_large::ICON_SIZE,
                icon_spacing: button_large::ICON_LABEL_SPACE,
                label: TypeScale::HeadlineSmall,
                outline_width: button_large::OUTLINED_OUTLINE_WIDTH,
                shape: button_large::CONTAINER_SHAPE_ROUND,
            },
            Self::ExtraLarge => SizeTokens {
                height: button_x_large::CONTAINER_HEIGHT,
                horizontal_padding: button_x_large::LEADING_SPACE,
                vertical_padding: Dp(48.0),
                icon_size: button_x_large::ICON_SIZE,
                icon_spacing: button_x_large::ICON_LABEL_SPACE,
                label: TypeScale::HeadlineLarge,
                outline_width: button_x_large::OUTLINED_OUTLINE_WIDTH,
                shape: button_x_large::CONTAINER_SHAPE_ROUND,
            },
        }
    }
}

/// 状態ごとの影の高さ
struct Elevations {
    default: Elevation,
    hovered: Elevation,
    focused: Elevation,
    pressed: Elevation,
}

/// バリアントごとの色・枠線・影
struct VariantTokens {
    /// `None` は透明
    container: Option<ColorRole>,
    content: ColorRole,
    /// `None` は透明
    disabled_container: Option<(ColorRole, f32)>,
    disabled_content: (ColorRole, f32),
    /// 枠線の色と、無効時の枠線の不透明度
    outline: Option<(ColorRole, f32)>,
    /// 無効時の影はどのバリアントにもない
    elevations: Option<Elevations>,
}

impl ButtonVariant {
    fn tokens(self) -> VariantTokens {
        match self {
            // `defaultButtonColors`、`buttonElevation`
            Self::Filled => VariantTokens {
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
                elevations: Some(Elevations {
                    default: filled_button::CONTAINER_ELEVATION,
                    hovered: filled_button::HOVERED_CONTAINER_ELEVATION,
                    focused: filled_button::FOCUSED_CONTAINER_ELEVATION,
                    pressed: filled_button::PRESSED_CONTAINER_ELEVATION,
                }),
            },
            // `defaultElevatedButtonColors`、`elevatedButtonElevation`
            Self::Elevated => VariantTokens {
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
                elevations: Some(Elevations {
                    default: elevated_button::CONTAINER_ELEVATION,
                    hovered: elevated_button::HOVERED_CONTAINER_ELEVATION,
                    focused: elevated_button::FOCUSED_CONTAINER_ELEVATION,
                    pressed: elevated_button::PRESSED_CONTAINER_ELEVATION,
                }),
            },
            // `defaultFilledTonalButtonColors`、`filledTonalButtonElevation`。
            // Compose はこのバリアントだけ版の古い FilledTonalButtonTokens (v0_103) を参照している
            Self::FilledTonal => VariantTokens {
                container: Some(filled_tonal_button::CONTAINER_COLOR),
                content: filled_tonal_button::LABEL_TEXT_COLOR,
                disabled_container: Some((
                    filled_tonal_button::DISABLED_CONTAINER_COLOR,
                    filled_tonal_button::DISABLED_CONTAINER_OPACITY,
                )),
                disabled_content: (
                    filled_tonal_button::DISABLED_LABEL_TEXT_COLOR,
                    filled_tonal_button::DISABLED_LABEL_TEXT_OPACITY,
                ),
                outline: None,
                elevations: Some(Elevations {
                    default: filled_tonal_button::CONTAINER_ELEVATION,
                    hovered: filled_tonal_button::HOVER_CONTAINER_ELEVATION,
                    focused: filled_tonal_button::FOCUS_CONTAINER_ELEVATION,
                    pressed: filled_tonal_button::PRESSED_CONTAINER_ELEVATION,
                }),
            },
            // `defaultOutlinedButtonColors`、`outlinedButtonBorder`
            Self::Outlined => VariantTokens {
                container: None,
                content: outlined_button::LABEL_TEXT_COLOR,
                disabled_container: None,
                disabled_content: (
                    outlined_button::DISABLED_LABEL_TEXT_COLOR,
                    outlined_button::DISABLED_LABEL_TEXT_OPACITY,
                ),
                outline: Some((
                    outlined_button::OUTLINE_COLOR,
                    outlined_button::DISABLED_CONTAINER_OPACITY,
                )),
                elevations: None,
            },
            // `defaultTextButtonColors`
            Self::Text => VariantTokens {
                container: None,
                // Compose は TextButtonTokens.LabelColor (OnSurfaceVariant) ではなく Primary を使う
                content: ColorRole::Primary,
                disabled_container: None,
                disabled_content: (
                    text_button::DISABLED_LABEL_COLOR,
                    text_button::DISABLED_LABEL_OPACITY,
                ),
                outline: None,
                elevations: None,
            },
        }
    }
}

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// MD3 のボタン
///
/// 子の内容をラベルとして、サイズの書体とバリアントの内容の色で描く。
/// 支援技術には role `button` として公開され、子の内容がアクセシブルな名前になる。
/// マウスのクリック・フォーカス中の Enter と Space で [`Button::on_click`] のハンドラを呼ぶ。
///
/// [`Styled`] で指定したスタイルはボタンの外側の箱に付く。大きさや外側の余白、flex の指定に使い、
/// 箱を広げるとボタンも箱いっぱいに広がる。色や形は変えられない
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    outer: Div,
    children: Vec<AnyElement>,
    variant: ButtonVariant,
    size: ButtonSize,
    icon: Option<Icon>,
    on_click: Option<ClickHandler>,
    disabled: bool,
    focus_handle: Option<FocusHandle>,
    theme: Option<SharedTheme>,
}

impl Button {
    /// [`ButtonVariant::Filled`] で [`ButtonSize::Small`] のボタンを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。[`Button::track_focus`] を指定しない場合、
    /// `id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            outer: div(),
            children: Vec::new(),
            variant: ButtonVariant::default(),
            size: ButtonSize::default(),
            icon: None,
            on_click: None,
            disabled: false,
            focus_handle: None,
            theme: None,
        }
    }

    /// ボタンの種類を設定する
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// ボタンの大きさを設定する
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// ラベルの前にアイコンを置く
    ///
    /// アイコンはラベルと同じ色になり、大きさとラベルとの間隔はサイズに合わせる。
    /// アイコンは装飾として扱い、アクセシブルな名前には含めない
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
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

    /// ボタンを無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効なボタンは無効の色で描かれ、押されてもハンドラを呼ばず、Tab でも到達できない。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// `focus_handle` をこのボタンのフォーカスとして使う
    ///
    /// 指定しない場合はボタンが内部でフォーカスを持つ
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

/// 形が `key` の要素の角の半径
///
/// ボタンの形は CornerFull (短辺の 50%) で、gpui は描画時に半径を短辺の半分までに切り詰める。
/// そのため描画前に大きさが分からなくても、十分に大きい寸法で求めれば実際の大きさの半円になる
fn corner_radii(theme: &Theme, key: ShapeKey, outset: Pixels) -> Corners<Pixels> {
    let radii = theme.corner_radii(key, size(px(f32::MAX), px(f32::MAX)));
    Corners {
        top_left: radii.top_left + outset,
        top_right: radii.top_right + outset,
        bottom_right: radii.bottom_right + outset,
        bottom_left: radii.bottom_left + outset,
    }
}

fn rounded<E: Styled>(element: E, radii: Corners<Pixels>) -> E {
    element
        .rounded_tl(radii.top_left)
        .rounded_tr(radii.top_right)
        .rounded_br(radii.bottom_right)
        .rounded_bl(radii.bottom_left)
}

impl RenderOnce for Button {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let tokens = self.variant.tokens();
        let size = self.size.tokens();
        let disabled = self.disabled;

        let focus_handle = self.focus_handle.unwrap_or_else(|| {
            window
                .use_keyed_state(self.id.clone(), cx, |_, cx| cx.focus_handle())
                .read(cx)
                .clone()
        });
        // gpui の focus_visible と同じ条件。リングはボタンの外側に描くため、ボタン自身のスタイルでは表せない
        let focus_visible = focus_handle.is_focused(window) && window.last_input_was_keyboard();

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
        let radii = corner_radii(&theme, size.shape, px(0.0));

        let mut button = HeadlessButton::new(self.id)
            .track_focus(&focus_handle)
            .disabled(disabled)
            .relative()
            .flex_grow_1()
            .flex()
            .items_center()
            .justify_center()
            .gap(dp(size.icon_spacing))
            .min_w(dp(MIN_WIDTH))
            .min_h(dp(size.height))
            .px(dp(size.horizontal_padding))
            .py(dp(size.vertical_padding))
            .text_color(content)
            .whitespace_nowrap()
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
                // 透明なバリアントは高さ 0 の影も付けない。gpui は 0 の影でも要素の真下を塗るため
                let layer =
                    |style: StyleRefinement, opacity: f32, pick: fn(&Elevations) -> Elevation| {
                        let style = style.bg(state_layer(opacity));
                        match &tokens.elevations {
                            Some(elevations) => style.shadow(theme.shadow(pick(elevations))),
                            None => style,
                        }
                    };
                // gpui は focus_visible、hover、active の順に重ねるため、押下 > hover > フォーカスの順に優先される
                this.focus_visible(|style| {
                    layer(style, state::FOCUS_STATE_LAYER_OPACITY, |e| e.focused)
                })
                .hover(|style| layer(style, state::HOVER_STATE_LAYER_OPACITY, |e| e.hovered))
                .active(|style| layer(style, state::PRESSED_STATE_LAYER_OPACITY, |e| e.pressed))
                .when_some(tokens.elevations.as_ref(), |this, e| {
                    this.shadow(theme.shadow(e.default))
                })
            })
            .when_some(self.icon, |this, icon| {
                this.child(
                    svg()
                        .data(icon.svg(IconStyle::Outlined, false).as_bytes())
                        .flex_none()
                        .size(dp(size.icon_size))
                        .text_color(content),
                )
            })
            .child(div().flex().items_center().children(self.children))
            .when(focus_visible, |this| {
                let outset = dp(focus_ring::OUTWARD_OFFSET) + dp(focus_ring::WIDTH);
                this.child(
                    rounded(div(), corner_radii(&theme, size.shape, outset))
                        .absolute()
                        .top(-outset)
                        .left(-outset)
                        .right(-outset)
                        .bottom(-outset)
                        .border(dp(focus_ring::WIDTH))
                        .border_color(theme.color(focus_ring::COLOR)),
                )
            })
            .when_some(self.on_click, |this, on_click| this.on_click(on_click));
        button.text_style().refine(&theme.text_style(size.label));

        self.outer.flex().child(rounded(button, radii))
    }
}

impl Styled for Button {
    fn style(&mut self) -> &mut StyleRefinement {
        self.outer.style()
    }
}

impl ParentElement for Button {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}
