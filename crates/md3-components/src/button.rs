// 寸法・色の選択・状態ごとの elevation は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Button.kt
//         (`Button`、`ButtonDefaults`、`ButtonElevation`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トグルボタンの色・形・枠線・モーションは同じく次から移植している
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ToggleButton.kt
//         (`ToggleButton`、`ToggleButtonDefaults`、`ElevatedToggleButtonDefaults`、
//         `FilledTonalToggleButtonDefaults`、`OutlinedToggleButtonDefaults`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2024 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 のボタン

use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    AnimationExt, AnimationPhase, AnyElement, App, BorderStyle, Bounds, ClickEvent, Corners, Div,
    ElementId, FocusHandle, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels,
    Refineable, RenderOnce, SharedString, SpringAnimation, SpringConfig,
    StatefulInteractiveElement, StyleRefinement, Styled, Window, canvas, div,
    prelude::FluentBuilder, quad, svg, transparent_black,
};
use gpui_headless::button::Button as HeadlessButton;
use gpui_headless::toggle::ToggleButton as HeadlessToggleButton;

#[cfg(feature = "menu")]
use crate::Menu;
use crate::shape::Shaped;
use crate::util;
use md3_icons::StyledIcon;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::{
    button_large, button_medium, button_small, button_x_large, button_x_small, elevated_button,
    filled_button, filled_tonal_button, outlined_button, text_button, tonal_button,
};
use md3_theme::md3_tokens::elevation::Elevation;
use md3_theme::md3_tokens::motion::MotionSchemeKey;
use md3_theme::md3_tokens::shape::{CornerShape, ShapeKey};
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::md3_tokens::{Dp, focus_ring, state, supplement};
use md3_theme::{SharedTheme, ThemeExt, dp};

/// 容器の層がボタンの hover を知るための group 名
const GROUP: &str = "md3-button";

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

/// ボタンの形
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ButtonShape {
    /// 短辺の両端を半円にした形
    #[default]
    Round,
    /// 角を小さく丸めた形。角の半径はサイズに合わせて決まる
    Square,
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
    round: ShapeKey,
    square: ShapeKey,
}

impl ButtonSize {
    // 余白は `ButtonDefaults.contentPaddingFor`、アイコンは `iconSizeFor` と `iconSpacingFor`、
    // 書体は `textStyleFor` が高さから選ぶ値。
    // 枠線の太さは、Compose の `outlinedButtonBorder` はどのサイズにも Small の値を使うが、サイズごとのトークンに従う
    fn tokens(self) -> SizeTokens {
        match self {
            Self::ExtraSmall => SizeTokens {
                height: button_x_small::CONTAINER_HEIGHT,
                horizontal_padding: supplement::button::EXTRA_SMALL_HORIZONTAL_PADDING,
                vertical_padding: supplement::button::EXTRA_SMALL_VERTICAL_PADDING,
                icon_size: button_x_small::ICON_SIZE,
                icon_spacing: supplement::button::EXTRA_SMALL_ICON_SPACING,
                label: supplement::button::SMALL_LABEL_TEXT,
                outline_width: button_x_small::OUTLINED_OUTLINE_WIDTH,
                round: button_x_small::CONTAINER_SHAPE_ROUND,
                square: button_x_small::CONTAINER_SHAPE_SQUARE,
            },
            Self::Small => SizeTokens {
                height: button_small::CONTAINER_HEIGHT,
                horizontal_padding: button_small::LEADING_SPACE,
                vertical_padding: supplement::button::SMALL_VERTICAL_PADDING,
                icon_size: button_small::ICON_SIZE,
                icon_spacing: button_small::ICON_LABEL_SPACE,
                label: supplement::button::SMALL_LABEL_TEXT,
                outline_width: button_small::OUTLINED_OUTLINE_WIDTH,
                round: button_small::CONTAINER_SHAPE_ROUND,
                square: button_small::CONTAINER_SHAPE_SQUARE,
            },
            Self::Medium => SizeTokens {
                height: button_medium::CONTAINER_HEIGHT,
                horizontal_padding: button_medium::LEADING_SPACE,
                vertical_padding: supplement::button::MEDIUM_VERTICAL_PADDING,
                icon_size: button_medium::ICON_SIZE,
                icon_spacing: button_medium::ICON_LABEL_SPACE,
                label: supplement::button::MEDIUM_LABEL_TEXT,
                outline_width: button_medium::OUTLINED_OUTLINE_WIDTH,
                round: button_medium::CONTAINER_SHAPE_ROUND,
                square: button_medium::CONTAINER_SHAPE_SQUARE,
            },
            Self::Large => SizeTokens {
                height: button_large::CONTAINER_HEIGHT,
                horizontal_padding: button_large::LEADING_SPACE,
                vertical_padding: supplement::button::LARGE_VERTICAL_PADDING,
                icon_size: button_large::ICON_SIZE,
                icon_spacing: button_large::ICON_LABEL_SPACE,
                label: supplement::button::LARGE_LABEL_TEXT,
                outline_width: button_large::OUTLINED_OUTLINE_WIDTH,
                round: button_large::CONTAINER_SHAPE_ROUND,
                square: button_large::CONTAINER_SHAPE_SQUARE,
            },
            Self::ExtraLarge => SizeTokens {
                height: button_x_large::CONTAINER_HEIGHT,
                horizontal_padding: button_x_large::LEADING_SPACE,
                vertical_padding: supplement::button::EXTRA_LARGE_VERTICAL_PADDING,
                icon_size: button_x_large::ICON_SIZE,
                icon_spacing: button_x_large::ICON_LABEL_SPACE,
                label: supplement::button::EXTRA_LARGE_LABEL_TEXT,
                outline_width: button_x_large::OUTLINED_OUTLINE_WIDTH,
                round: button_x_large::CONTAINER_SHAPE_ROUND,
                square: button_x_large::CONTAINER_SHAPE_SQUARE,
            },
        }
    }
}

/// 状態ごとの影の高さ
struct Elevations {
    default: Elevation,
    hovered: Elevation,
    focused: Elevation,
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
                content: supplement::button::TEXT_BUTTON_CONTENT_COLOR,
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

/// ボタンの色の上書き。`None` の項目はバリアントの色のまま
///
/// 色は役割で指定するため、テーマの明暗や配色が変わると追従する。
/// 状態レイヤーは内容の色から作る。枠線の色は変わらない
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ButtonColors {
    /// 有効なときの容器の色
    pub container: Option<ColorRole>,
    /// 有効なときのラベルとアイコンの色
    pub content: Option<ColorRole>,
    /// 無効なときの容器の色と不透明度
    pub disabled_container: Option<(ColorRole, f32)>,
    /// 無効なときのラベルとアイコンの色と不透明度
    pub disabled_content: Option<(ColorRole, f32)>,
}

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// MD3 のボタン
///
/// 子の内容をラベルとして、サイズの書体とバリアントの内容の色で描く。
/// 支援技術には role `button` として公開され、子孫に置いたテキストがアクセシブルな名前になる。
/// テキストは `gpui::text!` のように id を持つ要素にする。`&str`・`String`・`SharedString` をそのまま置いても名前にならない。
/// マウスのクリック・フォーカス中の Enter と Space で [`Button::on_click`] のハンドラを呼ぶ。
///
/// [`Styled`] で指定したスタイルはボタンの外側の箱に付く。大きさや外側の余白、flex の指定に使い、
/// 箱を広げるとボタンも箱いっぱいに広がる。箱を内容より狭くすると、ボタンは左右の余白を縮めて箱に合わせる。
/// 色や形は変えられないため、[`Button::colors`] と [`Button::shape`] を使う
///
/// テーマの密度 ([`ThemeSettings::density`](md3_theme::ThemeSettings::density)) は反映しない。
/// 大きさは [`Button::size`] で選ぶ
///
/// 押せる範囲はボタンの形を囲む矩形で、角丸の外側も含む
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    outer: Div,
    children: Vec<AnyElement>,
    variant: ButtonVariant,
    colors: ButtonColors,
    size: ButtonSize,
    shape: ButtonShape,
    icon: Option<StyledIcon>,
    on_click: Option<ClickHandler>,
    aria_label: Option<SharedString>,
    disabled: bool,
    focus_handle: Option<FocusHandle>,
    #[cfg(feature = "menu")]
    menu: Option<Menu>,
    theme: Option<SharedTheme>,
}

impl Button {
    /// [`ButtonVariant::Filled`]、[`ButtonSize::Small`]、[`ButtonShape::Round`] のボタンを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。[`Button::track_focus`] を指定しない場合、
    /// `id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            // 利用者が Styled で指定した display を render で上書きしないよう、既定はここで付ける
            outer: div().flex(),
            children: Vec::new(),
            variant: ButtonVariant::default(),
            colors: ButtonColors::default(),
            size: ButtonSize::default(),
            shape: ButtonShape::default(),
            icon: None,
            on_click: None,
            aria_label: None,
            disabled: false,
            focus_handle: None,
            #[cfg(feature = "menu")]
            menu: None,
            theme: None,
        }
    }

    /// ボタンの種類を設定する
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// バリアントの色を `colors` で上書きする
    ///
    /// 削除のような取り消せない操作を目立たせる場合などに使う
    pub fn colors(mut self, colors: ButtonColors) -> Self {
        self.colors = colors;
        self
    }

    /// ボタンの大きさを設定する
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// ボタンの形を設定する
    pub fn shape(mut self, shape: ButtonShape) -> Self {
        self.shape = shape;
        self
    }

    /// ラベルの前にアイコンを置く
    ///
    /// アイコンはラベルと同じ色になり、大きさとラベルとの間隔はサイズに合わせる。
    /// [`md3_icons::Icon`] を渡すと Outlined の塗りのない形で描く。スタイルや塗りは [`StyledIcon`] で指定する。
    /// アイコンは装飾として扱い、アクセシブルな名前には含めない
    pub fn icon(mut self, icon: impl Into<StyledIcon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// 子の内容の代わりに `label` をアクセシブルな名前にする
    ///
    /// 子孫に id を持つテキストがなく、名前を子の内容から決められない場合に使う
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
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

    /// このボタンを `menu` を開くボタンにする
    ///
    /// メニューの名前は既定で [`Button::aria_label`] になる。
    /// [`Button::on_click`] のハンドラがあれば、メニューの開閉を求める前に呼ぶ
    #[cfg(feature = "menu")]
    pub fn menu(mut self, menu: Menu) -> Self {
        self.menu = Some(menu);
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl Button {
    /// このボタンの見た目。アイコンと子の内容は取り出す
    pub(crate) fn look(&mut self, theme: &SharedTheme, focus_visible: bool) -> Look {
        let mut tokens = self.variant.tokens();
        tokens.container = self.colors.container.or(tokens.container);
        tokens.content = self.colors.content.unwrap_or(tokens.content);
        tokens.disabled_container = self.colors.disabled_container.or(tokens.disabled_container);
        tokens.disabled_content = self
            .colors
            .disabled_content
            .unwrap_or(tokens.disabled_content);
        let disabled = self.disabled;
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
        let size = self.size.tokens();
        Look {
            outline: tokens.outline.map(|outline| {
                (
                    outline_color(theme, outline, disabled),
                    dp(size.outline_width),
                )
            }),
            elevations: tokens.elevations,
            disabled,
            shape: match self.shape {
                ButtonShape::Round => size.round,
                ButtonShape::Square => size.square,
            },
            corner_shape: None,
            morph: None,
            padding: (size.horizontal_padding, size.horizontal_padding),
            vertical_padding: size.vertical_padding,
            min_width: supplement::button::MIN_WIDTH,
            ring_slot: None,
            size,
            theme: theme.clone(),
            container,
            content,
            focus_visible,
            icon: self.icon.take(),
            children: std::mem::take(&mut self.children),
        }
    }
}

impl RenderOnce for Button {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.clone().unwrap_or_else(|| cx.md3_theme().clone());
        let focus_handle = self
            .focus_handle
            .clone()
            .unwrap_or_else(|| util::keyed_focus_handle(&self.id, window, cx));
        // リングはボタンの外側に描くため、ボタン自身のスタイルでは表せない
        let focus_visible = gpui_headless::is_focus_visible(&focus_handle, window, cx);
        let look = self.look(&theme, focus_visible);
        let disabled = self.disabled;
        let mut button = HeadlessButton::new(self.id.clone())
            .track_focus(&focus_handle)
            .disabled(disabled)
            .when_some(self.aria_label.clone(), |this, label| {
                this.aria_label(label)
            })
            .when_some(self.on_click, |this, on_click| this.on_click(on_click));
        button = look.apply(button);
        #[cfg(feature = "menu")]
        if let Some(menu) = self.menu {
            button = menu.attach(
                button,
                &self.id,
                self.aria_label.as_ref(),
                &theme,
                window,
                cx,
            );
        }

        self.outer.child(button)
    }
}

/// 枠線の色。無効なときは `outline` の不透明度を掛ける
fn outline_color(theme: &SharedTheme, outline: (ColorRole, f32), disabled: bool) -> Hsla {
    let color = theme.color(outline.0);
    if disabled {
        color.opacity(outline.1)
    } else {
        color
    }
}

/// [`Button`] と [`ToggleButton`] で共通の見た目
pub(crate) struct Look {
    theme: SharedTheme,
    size: SizeTokens,
    shape: ShapeKey,
    /// `shape` の代わりに使う角ごとの形
    pub(crate) corner_shape: Option<CornerShape>,
    /// 状態に合わせて形を変える動き
    pub(crate) morph: Option<Morph>,
    /// 内容の前と後の余白。内容より狭い幅を指定されたときは余白から縮める
    pub(crate) padding: (Dp, Dp),
    /// 内容の上と下の余白
    pub(crate) vertical_padding: Dp,
    pub(crate) min_width: Dp,
    /// 指定すると、ボタンはフォーカスリングを描かず、リングを描く矩形 (外側に広げる前) と角の半径を prepaint で書く。
    /// 呼び出し側が [`paint_ring`] で描く
    pub(crate) ring_slot: Option<RingSlot>,
    /// `None` は透明
    container: Option<Hsla>,
    content: Hsla,
    /// 枠線の色と太さ。`None` は枠線なし
    outline: Option<(Hsla, Pixels)>,
    /// `None` は影なし。無効なときは影を描かない
    elevations: Option<Elevations>,
    /// 無効なときは状態レイヤーと影を描かない
    disabled: bool,
    focus_visible: bool,
    icon: Option<StyledIcon>,
    children: Vec<AnyElement>,
}

/// フォーカスリングを描く矩形と角の半径を受け取る場所。描画のたびに作り直す
pub(crate) type RingSlot = Rc<Cell<Option<(Bounds<Pixels>, Corners<Pixels>)>>>;

/// `ring` に書かれた矩形の外側にフォーカスリングを描く要素。リングを手前に描く要素の最後の子に置く。
/// ほかの要素と同じく親の中で描くため、祖先の切り取りの範囲と不透明度が効く
pub(crate) fn paint_ring(ring: RingSlot, theme: SharedTheme) -> impl IntoElement {
    canvas(
        |_, _, _| {},
        move |_, _, window, _| {
            let Some((bounds, radii)) = ring.get() else {
                return;
            };
            let outset = dp(focus_ring::OUTWARD_OFFSET) + dp(focus_ring::WIDTH);
            window.paint_quad(quad(
                bounds.dilate(outset),
                Corners {
                    top_left: radii.top_left + outset,
                    top_right: radii.top_right + outset,
                    bottom_right: radii.bottom_right + outset,
                    bottom_left: radii.bottom_left + outset,
                },
                transparent_black(),
                dp(focus_ring::WIDTH),
                theme.color(focus_ring::COLOR),
                BorderStyle::Solid,
            ));
        },
    )
    .absolute()
}

/// 状態に合わせて形を変える動き
pub(crate) struct Morph {
    /// バネの値 `i` (1 以上) で描く形。値 0 は [`Look`] の形で、間の値は隣り合う形を補間する
    pub(crate) shapes: Vec<CornerShape>,
    /// バネの目標の値
    pub(crate) target: f32,
    /// 形を変えるバネ
    pub(crate) spring: SpringConfig,
    /// バネの値が 0 から 1 へ進むほど枠線を細く薄くし、1 で消すか
    pub(crate) fade_outline: bool,
}

impl Look {
    /// 内容の色
    #[cfg(feature = "split-button")]
    pub(crate) fn content(&self) -> Hsla {
        self.content
    }

    /// 子の内容の末尾に `child` を加える
    #[cfg(feature = "split-button")]
    pub(crate) fn child(&mut self, child: impl IntoElement) {
        self.children.push(child.into_any_element());
    }

    /// 容器に内容の色を不透明度 `opacity` で重ねる
    #[cfg(feature = "split-button")]
    pub(crate) fn overlay_content(&mut self, opacity: f32) {
        let layer = self.content.opacity(opacity);
        self.container = Some(match self.container {
            Some(container) => container.blend(layer),
            None => layer,
        });
    }

    /// 容器・内容・フォーカスリングを `button` に描く
    ///
    /// [`Look::morph`] があれば、容器とリングの形をバネの値に合わせて変える
    pub(crate) fn apply<E>(self, button: E) -> E
    where
        E: ParentElement + Styled + InteractiveElement,
    {
        let Self {
            theme,
            size,
            shape,
            corner_shape,
            morph,
            padding,
            vertical_padding,
            min_width,
            ring_slot,
            container,
            content,
            outline,
            elevations,
            disabled,
            focus_visible,
            icon,
            children,
        } = self;
        // 状態レイヤーは内容の色を重ねる (Compose の `ripple()` は LocalContentColor を使う)
        let state_layer = move |opacity: f32| -> Hsla {
            let layer = content.opacity(opacity);
            match container {
                Some(container) => container.blend(layer),
                None => layer,
            }
        };
        let label = size.label;
        // 指定が無ければ、リングはボタンの最後の子で、子の内容より手前に描く
        let own_ring = ring_slot.is_none();
        let slot = ring_slot.unwrap_or_default();
        let painted_slot = slot.clone();

        // 容器とリングは形の動きの進み `phase` で描く。角丸は描画時の大きさで決まるため、
        // 背景・枠線・影は Shaped で包んだ層に描き、ボタン自身は透明な箱にする
        let theme_for_decoration = theme.clone();
        let shapes = morph.as_ref().map(|morph| morph.shapes.clone());
        let fade_outline = morph.as_ref().is_some_and(|morph| morph.fade_outline);
        let decoration = move |phase: f32| {
            let theme = theme_for_decoration;
            let shaped = |inner: Div| {
                let shaped = Shaped::new(inner, theme.clone(), shape);
                let shaped = match corner_shape {
                    Some(corner_shape) => shaped.corner_shape(corner_shape),
                    None => shaped,
                };
                match &shapes {
                    Some(shapes) => shaped.morph_through(shapes.clone(), phase),
                    None => shaped,
                }
            };
            // トグルの枠線は選ばれていないときだけ描き (`OutlinedToggleButtonDefaults.border`)、形と同じバネで変える。
            // gpui は正の太さを 1 物理ピクセルより細く描かないため、不透明度も下げて消す
            let outline = outline.map(|(color, width)| {
                if fade_outline {
                    let remaining = (1.0 - phase).clamp(0.0, 1.0);
                    (color.opacity(remaining), width * remaining)
                } else {
                    (color, width)
                }
            });
            let layer = div()
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .bottom_0()
                .when_some(container, |this, container| this.bg(container))
                .when_some(outline, |this, (color, width)| {
                    this.border(width).border_color(color)
                })
                .when(!disabled, |this| {
                    // Compose の OutlinedButton と TextButton は elevation の既定が null で、影を持たない
                    let shadow = |pick: fn(&Elevations) -> Elevation| {
                        elevations.as_ref().map(|e| theme.shadow(pick(e)))
                    };
                    let this = if focus_visible {
                        this.bg(state_layer(state::FOCUS_STATE_LAYER_OPACITY))
                            .when_some(shadow(|e| e.focused), |this, s| this.shadow(s))
                    } else {
                        this.when_some(shadow(|e| e.default), |this, s| this.shadow(s))
                    };
                    // gpui は group_hover を自身のスタイルの後に重ねるため、hover がフォーカスより優先される
                    this.group_hover(GROUP, |style| {
                        let style = style.bg(state_layer(state::HOVER_STATE_LAYER_OPACITY));
                        match shadow(|e| e.hovered) {
                            Some(s) => style.shadow(s),
                            None => style,
                        }
                    })
                });
            div()
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .bottom_0()
                .child(shaped(layer))
                .when(focus_visible, |this| {
                    // 形の動きの途中の角丸で、リングを描く矩形と角の半径を書く。
                    // 半径は gpui が容器を描くときと同じく短辺の半分までにする
                    let shape = shaped(div());
                    let slot = slot.clone();
                    this.child(
                        canvas(
                            move |bounds, _, _| {
                                let radii = shape.radii(bounds.size);
                                slot.set(Some((
                                    bounds,
                                    radii.clamp_radii_for_quad_size(bounds.size),
                                )));
                            },
                            |_, _, _, _| {},
                        )
                        .absolute()
                        .size_full(),
                    )
                })
        };
        let decoration = match morph {
            // 動きの要素はボタンの子に置き、ボタン自身の型を変えない。
            // 子が付ける文字列の id と重ならないよう、ソースの位置を id にする
            Some(morph) => div()
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .bottom_0()
                .with_spring(
                    ElementId::CodeLocation(*std::panic::Location::caller()),
                    SpringAnimation::new(morph.spring).to(AnimationPhase(morph.target)),
                    move |this, phase| this.child(decoration(phase.0)),
                )
                .into_any_element(),
            None => decoration(0.0).into_any_element(),
        };

        // 左右の余白は縮められる子で取り、内容より狭い幅を指定されたときは余白から縮める
        let horizontal_padding = |width: Dp| div().w(dp(width)).flex_shrink(1.0).min_w_0();
        let mut button = button
            .group(GROUP)
            .relative()
            .flex_grow_1()
            .flex()
            .items_center()
            .justify_center()
            .min_w(dp(min_width))
            .min_h(dp(size.height))
            .py(dp(vertical_padding))
            .text_color(content)
            .whitespace_nowrap()
            .child(decoration)
            .child(horizontal_padding(padding.0));
        // 項目を型で受け取るグループの項目 (gpui-headless の Radio など) は FluentBuilder を持たないため、分岐で付ける
        if let Some(icon) = icon {
            button = button.child(
                svg()
                    .data(icon.svg().as_bytes())
                    .flex_none()
                    .size(dp(size.icon_size))
                    .mr(dp(size.icon_spacing))
                    .text_color(content),
            );
        }
        button = button
            .child(div().flex().items_center().children(children))
            .child(horizontal_padding(padding.1));
        if own_ring {
            button = button.child(paint_ring(painted_slot, theme.clone()));
        }
        button.text_style().refine(&theme.text_style(label));
        button
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

/// トグルボタンの種類。強調の度合いが高い順に Filled、FilledTonal、Elevated、Outlined
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ToggleButtonVariant {
    /// 選ばれると塗りつぶすトグルボタン
    #[default]
    Filled,
    /// 影で浮かせたトグルボタン
    Elevated,
    /// 二次的な色のトグルボタン
    FilledTonal,
    /// 選ばれていない間は枠線で描くトグルボタン
    Outlined,
}

/// トグルボタンのバリアントごとの色
struct ToggleTokens {
    /// `None` は透明
    container: Option<ColorRole>,
    content: ColorRole,
    checked_container: ColorRole,
    checked_content: ColorRole,
    disabled_container: (ColorRole, f32),
    disabled_content: (ColorRole, f32),
}

impl ToggleButtonVariant {
    /// 枠線と影は同じ種類の [`ButtonVariant`] と同じ
    fn button_variant(self) -> ButtonVariant {
        match self {
            Self::Filled => ButtonVariant::Filled,
            Self::Elevated => ButtonVariant::Elevated,
            Self::FilledTonal => ButtonVariant::FilledTonal,
            Self::Outlined => ButtonVariant::Outlined,
        }
    }

    fn tokens(self) -> ToggleTokens {
        match self {
            // `defaultToggleButtonColors`
            Self::Filled => ToggleTokens {
                container: Some(filled_button::UNSELECTED_CONTAINER_COLOR),
                content: filled_button::UNSELECTED_PRESSED_LABEL_TEXT_COLOR,
                checked_container: filled_button::SELECTED_CONTAINER_COLOR,
                checked_content: filled_button::SELECTED_PRESSED_LABEL_TEXT_COLOR,
                disabled_container: (
                    filled_button::DISABLED_CONTAINER_COLOR,
                    filled_button::DISABLED_CONTAINER_OPACITY,
                ),
                disabled_content: (
                    filled_button::DISABLED_LABEL_TEXT_COLOR,
                    filled_button::DISABLED_LABEL_TEXT_OPACITY,
                ),
            },
            // `defaultElevatedToggleButtonColors`
            Self::Elevated => ToggleTokens {
                container: Some(elevated_button::UNSELECTED_CONTAINER_COLOR),
                content: elevated_button::UNSELECTED_PRESSED_LABEL_TEXT_COLOR,
                checked_container: elevated_button::SELECTED_CONTAINER_COLOR,
                checked_content: elevated_button::SELECTED_PRESSED_LABEL_TEXT_COLOR,
                disabled_container: (
                    elevated_button::DISABLED_CONTAINER_COLOR,
                    elevated_button::DISABLED_CONTAINER_OPACITY,
                ),
                disabled_content: (
                    elevated_button::DISABLED_LABEL_TEXT_COLOR,
                    elevated_button::DISABLED_LABEL_TEXT_OPACITY,
                ),
            },
            // `defaultFilledTonalToggleButtonColors`
            Self::FilledTonal => ToggleTokens {
                container: Some(tonal_button::UNSELECTED_CONTAINER_COLOR),
                content: tonal_button::UNSELECTED_LABEL_TEXT_COLOR,
                checked_container: tonal_button::SELECTED_CONTAINER_COLOR,
                checked_content: tonal_button::SELECTED_LABEL_TEXT_COLOR,
                disabled_container: (
                    tonal_button::DISABLED_CONTAINER_COLOR,
                    tonal_button::DISABLED_CONTAINER_OPACITY,
                ),
                disabled_content: (
                    tonal_button::DISABLED_LABEL_TEXT_COLOR,
                    tonal_button::DISABLED_LABEL_TEXT_OPACITY,
                ),
            },
            // `defaultOutlinedToggleButtonColors`。無効なときは選ばれているかによらず容器を薄く塗る
            Self::Outlined => ToggleTokens {
                container: None,
                content: outlined_button::UNSELECTED_LABEL_TEXT_COLOR,
                checked_container: outlined_button::SELECTED_CONTAINER_COLOR,
                checked_content: outlined_button::SELECTED_LABEL_TEXT_COLOR,
                disabled_container: (
                    outlined_button::DISABLED_OUTLINE_COLOR,
                    outlined_button::DISABLED_CONTAINER_OPACITY,
                ),
                disabled_content: (
                    outlined_button::DISABLED_LABEL_TEXT_COLOR,
                    outlined_button::DISABLED_LABEL_TEXT_OPACITY,
                ),
            },
        }
    }
}

/// トグルボタンの色の上書き。`None` の項目はバリアントの色のまま
///
/// 色は役割で指定するため、テーマの明暗や配色が変わると追従する。
/// 状態レイヤーは内容の色から作る。枠線の色は変わらない
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ToggleButtonColors {
    /// 選ばれていないときの容器の色
    pub container: Option<ColorRole>,
    /// 選ばれていないときのラベルとアイコンの色
    pub content: Option<ColorRole>,
    /// 選ばれているときの容器の色
    pub checked_container: Option<ColorRole>,
    /// 選ばれているときのラベルとアイコンの色
    pub checked_content: Option<ColorRole>,
    /// 無効なときの容器の色と不透明度
    pub disabled_container: Option<(ColorRole, f32)>,
    /// 無効なときのラベルとアイコンの色と不透明度
    pub disabled_content: Option<(ColorRole, f32)>,
}

type ChangeHandler = Box<dyn Fn(&bool, &mut Window, &mut App)>;

/// MD3 のトグルボタン。選ばれているかで色と形が変わるボタン
///
/// 選ばれている状態は呼び出し側が持ち、[`ToggleButton::new`] に毎回渡す。ボタン自身は状態を変えない。
/// 選ばれると、[`ButtonShape::Round`] は角を丸めた形に、[`ButtonShape::Square`] は短辺の両端を半円にした形に変わる。
/// [`ToggleButtonVariant::Outlined`] は選ばれていない間だけ枠線を描く
///
/// 子の内容をラベルとして描く。支援技術には role `button` と押されているかどうかの状態として公開され、
/// 子孫に置いたテキストがアクセシブルな名前になる。テキストは `gpui::text!` のように id を持つ要素にする。
/// 名前は選ばれているかによって変えないこと。
/// マウスのクリック・フォーカス中の Enter と Space で [`ToggleButton::on_change`] のハンドラを呼ぶ
///
/// [`Styled`] で指定したスタイルはボタンの外側の箱に付く。大きさや外側の余白、flex の指定に使い、
/// 箱を広げるとボタンも箱いっぱいに広がる。箱を内容より狭くすると、ボタンは左右の余白を縮めて箱に合わせる。
/// 色や形は変えられないため、[`ToggleButton::colors`] と [`ToggleButton::shape`] を使う
///
/// テーマの密度 ([`ThemeSettings::density`](md3_theme::ThemeSettings::density)) は反映しない。
/// 大きさは [`ToggleButton::size`] で選ぶ
///
/// 押せる範囲はボタンの形を囲む矩形で、角丸の外側も含む
#[derive(IntoElement)]
pub struct ToggleButton {
    id: ElementId,
    outer: Div,
    children: Vec<AnyElement>,
    checked: bool,
    variant: ToggleButtonVariant,
    colors: ToggleButtonColors,
    size: ButtonSize,
    shape: ButtonShape,
    icon: Option<StyledIcon>,
    on_change: Option<ChangeHandler>,
    aria_label: Option<SharedString>,
    disabled: bool,
    focus_handle: Option<FocusHandle>,
    theme: Option<SharedTheme>,
}

impl ToggleButton {
    /// `checked` の状態を表示する、[`ToggleButtonVariant::Filled`]、[`ButtonSize::Small`]、[`ButtonShape::Round`] のトグルボタンを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。[`ToggleButton::track_focus`] を指定しない場合、
    /// `id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(id: impl Into<ElementId>, checked: bool) -> Self {
        Self {
            id: id.into(),
            // 利用者が Styled で指定した display を render で上書きしないよう、既定はここで付ける
            outer: div().flex(),
            children: Vec::new(),
            checked,
            variant: ToggleButtonVariant::default(),
            colors: ToggleButtonColors::default(),
            size: ButtonSize::default(),
            shape: ButtonShape::default(),
            icon: None,
            on_change: None,
            aria_label: None,
            disabled: false,
            focus_handle: None,
            theme: None,
        }
    }

    /// トグルボタンの種類を設定する
    pub fn variant(mut self, variant: ToggleButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// バリアントの色を `colors` で上書きする
    pub fn colors(mut self, colors: ToggleButtonColors) -> Self {
        self.colors = colors;
        self
    }

    /// トグルボタンの大きさを設定する
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// 選ばれていないときの形を設定する
    pub fn shape(mut self, shape: ButtonShape) -> Self {
        self.shape = shape;
        self
    }

    /// ラベルの前にアイコンを置く
    ///
    /// アイコンはラベルと同じ色になり、大きさとラベルとの間隔はサイズに合わせる。
    /// [`md3_icons::Icon`] を渡すと Outlined の塗りのない形で描く。スタイルや塗りは [`StyledIcon`] で指定する。
    /// アイコンは装飾として扱い、アクセシブルな名前には含めない
    pub fn icon(mut self, icon: impl Into<StyledIcon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// 子の内容の代わりに `label` をアクセシブルな名前にする
    ///
    /// 子孫に id を持つテキストがなく、名前を子の内容から決められない場合に使う
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// 押されたときに、反転した状態で `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`ToggleButton::new`] に渡すこと
    pub fn on_change(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// トグルボタンを無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効なトグルボタンは選ばれているかによらず無効の色で描かれ、押されてもハンドラを呼ばず、Tab でも到達できない。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// `focus_handle` をこのトグルボタンのフォーカスとして使う
    ///
    /// 指定しない場合はトグルボタンが内部でフォーカスを持つ
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

impl ToggleButton {
    /// 色・形・枠線・アイコン・子の内容から見た目を作る。アイコンと子の内容は見た目に移す
    pub(crate) fn look(&mut self, theme: &SharedTheme, focus_visible: bool) -> Look {
        let tokens = self.variant.tokens();
        let button_tokens = self.variant.button_variant().tokens();
        let colors = self.colors;
        let disabled = self.disabled;
        let checked = self.checked;

        // Compose の `ToggleButtonColors` と同じく、無効なときは選ばれているかによらず無効の色にする
        let (container, content) = if disabled {
            let (role, opacity) = colors
                .disabled_container
                .unwrap_or(tokens.disabled_container);
            let (content, content_opacity) =
                colors.disabled_content.unwrap_or(tokens.disabled_content);
            (
                Some(theme.color(role).opacity(opacity)),
                theme.color(content).opacity(content_opacity),
            )
        } else if checked {
            (
                Some(theme.color(colors.checked_container.unwrap_or(tokens.checked_container))),
                theme.color(colors.checked_content.unwrap_or(tokens.checked_content)),
            )
        } else {
            (
                colors
                    .container
                    .or(tokens.container)
                    .map(|role| theme.color(role)),
                theme.color(colors.content.unwrap_or(tokens.content)),
            )
        };
        let size = self.size.tokens();
        let padding = (size.horizontal_padding, size.horizontal_padding);
        let vertical_padding = size.vertical_padding;
        // 選ばれると、丸い形は角を丸めた形に、角を丸めた形は丸い形に変わる
        let (shape, checked_shape) = match self.shape {
            ButtonShape::Round => (size.round, size.square),
            ButtonShape::Square => (size.square, size.round),
        };
        Look {
            // 枠線の太さはどのサイズでも Small の値 (`outlinedButtonBorder`)
            outline: button_tokens.outline.map(|outline| {
                (
                    outline_color(theme, outline, disabled),
                    dp(button_small::OUTLINED_OUTLINE_WIDTH),
                )
            }),
            elevations: button_tokens.elevations,
            disabled,
            size,
            shape,
            corner_shape: None,
            ring_slot: None,
            // Compose は形を FastSpatial のバネで変える
            morph: Some(Morph {
                shapes: vec![checked_shape.resolve(&theme.settings().shapes)],
                target: if checked { 1.0 } else { 0.0 },
                spring: theme.spring(MotionSchemeKey::FastSpatial),
                fade_outline: true,
            }),
            padding,
            vertical_padding,
            min_width: supplement::button::MIN_WIDTH,
            container,
            content,
            focus_visible,
            icon: self.icon.take(),
            children: std::mem::take(&mut self.children),
            theme: theme.clone(),
        }
    }
}

impl RenderOnce for ToggleButton {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.clone().unwrap_or_else(|| cx.md3_theme().clone());
        let focus_handle = self
            .focus_handle
            .clone()
            .unwrap_or_else(|| util::keyed_focus_handle(&self.id, window, cx));
        // リングはボタンの外側に描くため、ボタン自身のスタイルでは表せない
        let focus_visible = gpui_headless::is_focus_visible(&focus_handle, window, cx);
        let look = self.look(&theme, focus_visible);
        let button = HeadlessToggleButton::new(self.id, self.checked)
            .track_focus(&focus_handle)
            .disabled(self.disabled)
            .when_some(self.aria_label, |this, label| this.aria_label(label))
            .when_some(self.on_change, |this, on_change| this.on_change(on_change));
        self.outer.child(look.apply(button))
    }
}

impl Styled for ToggleButton {
    fn style(&mut self) -> &mut StyleRefinement {
        self.outer.style()
    }
}

impl ParentElement for ToggleButton {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Context, Render, TestAppContext, VisualTestContext, px};
    use md3_theme::md3_color::ColorRole;

    use super::*;

    struct ElementView(Box<dyn Fn() -> AnyElement>);

    impl Render for ElementView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            // 原点のマウスの位置で hover にならないよう、ボタンを原点から離す
            div()
                .flex()
                .items_start()
                .pl(px(100.0))
                .pt(px(100.0))
                .child((self.0)())
        }
    }

    /// 容器と、幅 60dp のラベルの x と幅
    fn layout(cx: &mut TestAppContext, button: fn() -> Button) -> ((f32, f32), (f32, f32)) {
        let (_, cx) = cx.add_window_view(move |window, cx| {
            cx.init_md3_theme(window);
            ElementView(Box::new(move || {
                button()
                    .child(div().w(px(60.0)).h(px(10.0)).bg(gpui::red()))
                    .into_any_element()
            }))
        });
        cx.run_until_parked();
        let rect = |cx: &mut VisualTestContext, background: gpui::Background| {
            cx.update(|window, _| {
                let scale = window.scale_factor();
                let quad = window
                    .painted_quads()
                    .into_iter()
                    .find(|quad| quad.background == background)
                    .expect("描かれていること");
                (
                    quad.bounds.origin.x.0 / scale - 100.0,
                    quad.bounds.size.width.0 / scale,
                )
            })
        };
        let container = cx.update(|_, cx| cx.md3_theme().color(ColorRole::Primary).into());
        (rect(cx, container), rect(cx, gpui::red().into()))
    }

    #[gpui::test]
    fn 内容の左右に余白を取った幅になる(cx: &mut TestAppContext) {
        let (container, label) = layout(cx, || Button::new("button"));
        assert_eq!(container, (0.0, 92.0), "16dp + 60dp + 16dp");
        assert_eq!(label, (16.0, 60.0));
    }

    #[gpui::test]
    fn 内容より狭い幅では左右の余白を縮める(cx: &mut TestAppContext) {
        let (container, label) = layout(cx, || Button::new("button").w(px(80.0)));
        assert_eq!(container, (0.0, 80.0));
        assert_eq!(label, (10.0, 60.0), "余白を 6dp ずつ縮めること");
    }

    /// 左上 (100, 100) に置いたトグルボタン。押すと `checked` を反転する
    fn open_toggle(
        cx: &mut TestAppContext,
        checked: bool,
        button: fn(bool) -> ToggleButton,
    ) -> (std::rc::Rc<std::cell::Cell<bool>>, &mut VisualTestContext) {
        let state = std::rc::Rc::new(std::cell::Cell::new(checked));
        let view_state = state.clone();
        let (_, cx) = cx.add_window_view(move |window, cx| {
            cx.init_md3_theme(window);
            let state = view_state.clone();
            ElementView(Box::new(move || {
                let state = state.clone();
                button(state.get())
                    .on_change(move |checked, window, _| {
                        state.set(*checked);
                        window.refresh();
                    })
                    .child(div().w(px(60.0)).h(px(10.0)))
                    .into_any_element()
            }))
        });
        // 形のバネを終端まで即座に進める
        cx.update(|_, cx| cx.set_reduce_motion(true));
        cx.run_until_parked();
        (state, cx)
    }

    /// トグルボタンを押し、hover の色が重ならないようポインターを外へ動かす
    fn click(cx: &mut VisualTestContext) {
        cx.simulate_click(gpui::point(px(130.0), px(120.0)), gpui::Modifiers::none());
        cx.simulate_mouse_move(
            gpui::point(px(10.0), px(10.0)),
            None,
            gpui::Modifiers::none(),
        );
        cx.run_until_parked();
    }

    /// `role` の色で塗った容器の、左上の角の半径と枠線の太さ
    fn container(cx: &mut VisualTestContext, role: ColorRole) -> Option<(f32, f32)> {
        cx.update(|window, cx| {
            let scale = window.scale_factor();
            let background: gpui::Background = cx.md3_theme().color(role).into();
            window
                .painted_quads()
                .into_iter()
                .find(|quad| quad.background == background)
                .map(|quad| {
                    (
                        quad.corner_radii.top_left.0 / scale,
                        quad.border_widths.top.0 / scale,
                    )
                })
        })
    }

    #[gpui::test]
    fn トグルボタンは押すと選ばれ_色と形が変わる(cx: &mut TestAppContext) {
        let (state, cx) = open_toggle(cx, false, |checked| ToggleButton::new("toggle", checked));
        assert_eq!(
            container(cx, ColorRole::SurfaceContainer),
            Some((20.0, 0.0)),
            "選ばれていない間は丸い形にすること"
        );
        click(cx);
        assert!(state.get(), "押すと選ぶよう求めること");
        assert_eq!(
            container(cx, ColorRole::Primary),
            Some((12.0, 0.0)),
            "選ばれると角を丸めた形にすること"
        );
    }

    #[gpui::test]
    fn 角を丸めたトグルボタンは選ばれると丸くなる(cx: &mut TestAppContext) {
        let (_, cx) = open_toggle(cx, true, |checked| {
            ToggleButton::new("toggle", checked).shape(ButtonShape::Square)
        });
        assert_eq!(container(cx, ColorRole::Primary), Some((20.0, 0.0)));
    }

    #[gpui::test]
    fn 枠線のトグルボタンは選ばれていない間だけ枠線を描く(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = open_toggle(cx, false, |checked| {
            ToggleButton::new("toggle", checked).variant(ToggleButtonVariant::Outlined)
        });
        let outlined = cx.update(|window, _| {
            window
                .painted_quads()
                .iter()
                .any(|quad| quad.border_widths.top.0 > 0.0)
        });
        assert!(outlined, "選ばれていない間は枠線を描くこと");
        click(cx);
        assert_eq!(container(cx, ColorRole::InverseSurface), Some((12.0, 0.0)));
        let outlined = cx.update(|window, _| {
            window
                .painted_quads()
                .iter()
                .any(|quad| quad.border_widths.top.0 > 0.0)
        });
        assert!(!outlined, "選ばれると枠線を描かないこと");
    }

    #[gpui::test]
    fn 無効なトグルボタンは選ばれていても無効の色で描く(
        cx: &mut TestAppContext,
    ) {
        let (state, cx) = open_toggle(cx, true, |checked| {
            ToggleButton::new("toggle", checked).disabled(true)
        });
        let disabled = cx.update(|_, cx| {
            gpui::Background::from(cx.md3_theme().color(ColorRole::OnSurface).opacity(0.1))
        });
        let painted = |cx: &mut VisualTestContext| {
            cx.update(|window, _| {
                window
                    .painted_quads()
                    .iter()
                    .any(|quad| quad.background == disabled)
            })
        };
        assert!(painted(cx), "選ばれていても無効の色で塗ること");
        click(cx);
        assert!(state.get(), "無効な間は押しても変えないこと");
    }

    #[gpui::test]
    fn フォーカスリングは子の内容より後に描く(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            ElementView(Box::new(|| {
                // ボタンより広く縮まない子は、ボタンの外側へはみ出してリングと重なる
                Button::new("button")
                    .w(px(100.0))
                    .child(div().flex_none().w(px(120.0)).h(px(10.0)).bg(gpui::red()))
                    .into_any_element()
            }))
        });
        cx.update(|window, _| window.activate_window());
        cx.simulate_keystrokes("tab");
        cx.run_until_parked();
        let (ring, content) = cx.update(|window, cx| {
            let ring = cx.md3_theme().color(focus_ring::COLOR);
            let content: gpui::Background = gpui::red().into();
            let quads = window.painted_quads();
            (
                quads
                    .iter()
                    .filter(|quad| quad.border_color == ring && quad.border_widths.top.0 > 0.0)
                    .map(|quad| quad.order)
                    .max(),
                quads
                    .iter()
                    .find(|quad| quad.background == content)
                    .map(|quad| quad.order),
            )
        });
        assert!(ring.expect("リングを描くこと") > content.expect("子を描くこと"));
    }
}
