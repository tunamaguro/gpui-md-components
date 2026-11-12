// 寸法・色・形・状態ごとの elevation・Extended FAB の展開と折りたたみは Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/FloatingActionButton.kt
//         (`FloatingActionButton`、`SmallFloatingActionButton`、`MediumFloatingActionButton`、
//         `LargeFloatingActionButton`、`SmallExtendedFloatingActionButton`、`MediumExtendedFloatingActionButton`、
//         `LargeExtendedFloatingActionButton`、`FloatingActionButtonDefaults`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 の FAB (floating action button)

use gpui::{
    AnimationExt, App, ClickEvent, Div, ElementId, FocusHandle, InteractiveElement, IntoElement,
    ParentElement, Refineable, RenderOnce, SharedString, SpringAnimation,
    StatefulInteractiveElement, StyleRefinement, Styled, Window, div, prelude::FluentBuilder, svg,
};
use gpui_headless::button::Button as HeadlessButton;

use crate::shape::Shaped;
use crate::util;
use md3_icons::StyledIcon;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::{
    extended_fab_large, extended_fab_medium, extended_fab_small, fab_baseline, fab_large,
    fab_medium, fab_primary_container, fab_secondary_container, fab_small,
};
use md3_theme::md3_tokens::motion::MotionSchemeKey;
use md3_theme::md3_tokens::shape::ShapeKey;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::md3_tokens::{Dp, focus_ring, state, supplement};
use md3_theme::{SharedTheme, ThemeExt, dp};

/// 容器の層が FAB の hover を知るための group 名
const GROUP: &str = "md3-fab";

/// FAB の大きさ。容器・アイコン・形が大きさに合わせて決まる
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FabSize {
    /// 40dp 角
    Small,
    /// 56dp 角
    #[default]
    Baseline,
    /// 80dp 角
    Medium,
    /// 96dp 角
    Large,
}

/// Extended FAB の大きさ。高さ・余白・アイコン・ラベルの書体・形が大きさに合わせて決まる
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ExtendedFabSize {
    /// 高さ 56dp
    #[default]
    Small,
    /// 高さ 80dp
    Medium,
    /// 高さ 96dp
    Large,
}

/// FAB の容器の色。内容 (アイコンとラベル) は容器の色と組になる色で描く
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FabColor {
    /// 容器は [`ColorRole::PrimaryContainer`]、内容は [`ColorRole::OnPrimaryContainer`]
    #[default]
    PrimaryContainer,
    /// 容器は [`ColorRole::SecondaryContainer`]、内容は [`ColorRole::OnSecondaryContainer`]
    SecondaryContainer,
    /// 容器は [`ColorRole::TertiaryContainer`]、内容は [`ColorRole::OnTertiaryContainer`]
    TertiaryContainer,
    /// 容器は [`ColorRole::Primary`]、内容は [`ColorRole::OnPrimary`]
    Primary,
    /// 容器は [`ColorRole::Secondary`]、内容は [`ColorRole::OnSecondary`]
    Secondary,
    /// 容器は [`ColorRole::Tertiary`]、内容は [`ColorRole::OnTertiary`]
    Tertiary,
}

impl FabColor {
    /// 容器と内容の色
    // PrimaryContainer と SecondaryContainer は FabPrimaryContainerTokens・FabSecondaryContainerTokens に従う。
    // ほかはトークンが無いため、Compose の `contentColorFor` が容器の色に組み合わせる内容の色を使う
    fn roles(self) -> (ColorRole, ColorRole) {
        match self {
            Self::PrimaryContainer => (
                fab_primary_container::CONTAINER_COLOR,
                fab_primary_container::ICON_COLOR,
            ),
            Self::SecondaryContainer => (
                fab_secondary_container::CONTAINER_COLOR,
                fab_secondary_container::ICON_COLOR,
            ),
            Self::TertiaryContainer => {
                (ColorRole::TertiaryContainer, ColorRole::OnTertiaryContainer)
            }
            Self::Primary => (ColorRole::Primary, ColorRole::OnPrimary),
            Self::Secondary => (ColorRole::Secondary, ColorRole::OnSecondary),
            Self::Tertiary => (ColorRole::Tertiary, ColorRole::OnTertiary),
        }
    }
}

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// 容器の層。色・影・状態レイヤーを形 `shape` で描き、親いっぱいに広がる
fn container_layer(
    theme: &SharedTheme,
    color: FabColor,
    shape: ShapeKey,
    focus_visible: bool,
) -> Shaped {
    let (container, content) = color.roles();
    let container = theme.color(container);
    let content = theme.color(content);
    // 状態レイヤーは内容の色を重ねる (Compose の `ripple()` は LocalContentColor を使う)
    let state_layer = |opacity: f32| container.blend(content.opacity(opacity));
    let hovered_shadow = theme.shadow(fab_primary_container::HOVERED_CONTAINER_ELEVATION);
    // 影は `FloatingActionButtonDefaults.elevation()` と同じく、色によらず FabPrimaryContainerTokens の段階を使う
    let layer = div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .map(|this| {
            if focus_visible {
                this.bg(state_layer(state::FOCUS_STATE_LAYER_OPACITY))
                    .shadow(theme.shadow(fab_primary_container::FOCUSED_CONTAINER_ELEVATION))
            } else {
                this.bg(container)
                    .shadow(theme.shadow(fab_primary_container::CONTAINER_ELEVATION))
            }
        })
        // gpui は group_hover を自身のスタイルの後に重ねるため、hover がフォーカスより優先される
        .group_hover(GROUP, |style| {
            style
                .bg(state_layer(state::HOVER_STATE_LAYER_OPACITY))
                .shadow(hovered_shadow)
        });
    Shaped::new(layer, theme.clone(), shape)
}

/// 形 `shape` の容器の外側に描くフォーカスリング
fn focus_ring(theme: &SharedTheme, shape: ShapeKey) -> Shaped {
    let outset = dp(focus_ring::OUTWARD_OFFSET) + dp(focus_ring::WIDTH);
    let ring = div()
        .absolute()
        .top(-outset)
        .left(-outset)
        .right(-outset)
        .bottom(-outset)
        .border(dp(focus_ring::WIDTH))
        .border_color(theme.color(focus_ring::COLOR));
    Shaped::new(ring, theme.clone(), shape).outset(outset)
}

/// MD3 の FAB (floating action button)
///
/// 画面の主な操作を、影で浮かせた容器のアイコンで示す。
/// 支援技術には role `button` として公開され、[`Fab::new`] の `label` がアクセシブルな名前になる。
/// マウスのクリック・フォーカス中の Enter と Space で [`Fab::on_click`] のハンドラを呼ぶ。
/// 押せる範囲は形を囲む矩形で、角丸の外側も含む。[`FabSize::Small`] でも 48dp には広げない。
/// 無効にする手段は持たない
///
/// [`Styled`] で指定したスタイルは FAB の外側の箱に付く。位置や外側の余白、flex の指定に使い、
/// 箱を広げると FAB も箱いっぱいに広がる。色や形は変えられないため、[`Fab::color`] と [`Fab::size`] を使う
#[derive(IntoElement)]
pub struct Fab {
    id: ElementId,
    outer: Div,
    icon: StyledIcon,
    label: SharedString,
    size: FabSize,
    color: FabColor,
    on_click: Option<ClickHandler>,
    focus_handle: Option<FocusHandle>,
    theme: Option<SharedTheme>,
}

impl Fab {
    /// `icon` を描き、`label` をアクセシブルな名前にした FAB を作る
    ///
    /// [`FabSize::Baseline`]、[`FabColor::PrimaryContainer`] で描く。
    /// `label` は FAB が何をするかを表す文字列にする。画面には表示しない。
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。[`Fab::track_focus`] を指定しない場合、
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
            size: FabSize::default(),
            color: FabColor::default(),
            on_click: None,
            focus_handle: None,
            theme: None,
        }
    }

    /// FAB の大きさを設定する
    pub fn size(mut self, size: FabSize) -> Self {
        self.size = size;
        self
    }

    /// 容器と内容の色を設定する
    pub fn color(mut self, color: FabColor) -> Self {
        self.color = color;
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

    /// `focus_handle` をこの FAB のフォーカスとして使う
    ///
    /// 指定しない場合は FAB が内部でフォーカスを持つ
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

impl RenderOnce for Fab {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        // 容器の一辺、アイコンの大きさ、形
        let (container, icon_size, shape) = match self.size {
            FabSize::Small => (
                fab_small::CONTAINER_WIDTH,
                fab_small::ICON_SIZE,
                fab_small::CONTAINER_SHAPE,
            ),
            FabSize::Baseline => (
                fab_baseline::CONTAINER_WIDTH,
                fab_baseline::ICON_SIZE,
                fab_baseline::CONTAINER_SHAPE,
            ),
            FabSize::Medium => (
                fab_medium::CONTAINER_WIDTH,
                fab_medium::ICON_SIZE,
                supplement::fab::MEDIUM_CONTAINER_SHAPE,
            ),
            FabSize::Large => (
                fab_large::CONTAINER_WIDTH,
                supplement::fab::LARGE_ICON_SIZE,
                fab_large::CONTAINER_SHAPE,
            ),
        };
        let focus_handle = self
            .focus_handle
            .unwrap_or_else(|| util::keyed_focus_handle(&self.id, window, cx));
        // リングは容器の外側に描くため、FAB 自身のスタイルでは表せない
        let focus_visible = gpui_headless::is_focus_visible(&focus_handle, window, cx);
        let (_, content) = self.color.roles();

        let button = HeadlessButton::new(self.id)
            .track_focus(&focus_handle)
            .group(GROUP)
            .relative()
            .flex_grow_1()
            .flex()
            .items_center()
            .justify_center()
            .min_w(dp(container))
            .min_h(dp(container))
            .child(container_layer(&theme, self.color, shape, focus_visible))
            .child(
                svg()
                    .data(self.icon.svg().as_bytes())
                    .flex_none()
                    .size(dp(icon_size))
                    .text_color(theme.color(content)),
            )
            .when(focus_visible, |this| this.child(focus_ring(&theme, shape)))
            .aria_label(self.label)
            .when_some(self.on_click, |this, on_click| this.on_click(on_click));

        self.outer.child(button)
    }
}

impl Styled for Fab {
    fn style(&mut self) -> &mut StyleRefinement {
        self.outer.style()
    }
}

/// アイコンとラベルを並べた MD3 の FAB
///
/// 画面の主な操作を、影で浮かせた容器のアイコンとラベルで示す。
/// [`ExtendedFab::expanded`] で折りたたむと、ラベルを隠して高さと同じ幅になる。幅とラベルの不透明度はテーマのバネで滑らかに変わる。
/// 支援技術には role `button` として公開され、ラベルの文字列が、折りたたんでいる間もアクセシブルな名前になる。
/// マウスのクリック・フォーカス中の Enter と Space で [`ExtendedFab::on_click`] のハンドラを呼ぶ。
/// 押せる範囲は形を囲む矩形で、角丸の外側も含む。
/// 無効にする手段は持たない
///
/// [`Styled`] で指定したスタイルは FAB の外側の箱に付く。位置や外側の余白、flex の指定に使う。
/// FAB の高さは [`ExtendedFab::size`] で、幅はそれに加えてラベルと展開の状態で決まり、箱を広げても変わらない。
/// 色や形は変えられないため、[`ExtendedFab::color`] と [`ExtendedFab::size`] を使う
#[derive(IntoElement)]
pub struct ExtendedFab {
    id: ElementId,
    outer: Div,
    icon: StyledIcon,
    text: SharedString,
    size: ExtendedFabSize,
    color: FabColor,
    expanded: bool,
    on_click: Option<ClickHandler>,
    focus_handle: Option<FocusHandle>,
    theme: Option<SharedTheme>,
}

impl ExtendedFab {
    /// `icon` とラベル `text` を並べた、展開した Extended FAB を作る
    ///
    /// [`ExtendedFabSize::Small`]、[`FabColor::PrimaryContainer`] で描く。`text` は 1 行で描き、改行を含めないこと。
    /// 含めた場合、debug ビルドでは描画時に panic する。
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。[`ExtendedFab::track_focus`] を指定しない場合、
    /// `id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(
        id: impl Into<ElementId>,
        icon: impl Into<StyledIcon>,
        text: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            // 利用者が Styled で指定した display を render で上書きしないよう、既定はここで付ける
            outer: div().flex(),
            icon: icon.into(),
            text: text.into(),
            size: ExtendedFabSize::default(),
            color: FabColor::default(),
            expanded: true,
            on_click: None,
            focus_handle: None,
            theme: None,
        }
    }

    /// Extended FAB の大きさを設定する
    pub fn size(mut self, size: ExtendedFabSize) -> Self {
        self.size = size;
        self
    }

    /// 容器と内容の色を設定する
    pub fn color(mut self, color: FabColor) -> Self {
        self.color = color;
        self
    }

    /// `false` を渡すと、ラベルを隠して高さと同じ幅に折りたたむ。既定では展開している
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
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

    /// `focus_handle` をこの Extended FAB のフォーカスとして使う
    ///
    /// 指定しない場合は Extended FAB が内部でフォーカスを持つ
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

/// Extended FAB の大きさごとの寸法
struct ExtendedSizeTokens {
    height: Dp,
    icon_size: Dp,
    leading_space: Dp,
    trailing_space: Dp,
    icon_label_space: Dp,
    label: TypeScale,
    shape: ShapeKey,
}

impl ExtendedFabSize {
    fn tokens(self) -> ExtendedSizeTokens {
        match self {
            Self::Small => ExtendedSizeTokens {
                height: extended_fab_small::CONTAINER_HEIGHT,
                icon_size: extended_fab_small::ICON_SIZE,
                leading_space: extended_fab_small::LEADING_SPACE,
                trailing_space: extended_fab_small::TRAILING_SPACE,
                icon_label_space: extended_fab_small::ICON_LABEL_SPACE,
                label: supplement::fab::SMALL_EXTENDED_LABEL_FONT,
                shape: extended_fab_small::CONTAINER_SHAPE,
            },
            Self::Medium => ExtendedSizeTokens {
                height: extended_fab_medium::CONTAINER_HEIGHT,
                // Compose のサンプルは Medium の FAB と同じアイコンの大きさを指定する
                icon_size: fab_medium::ICON_SIZE,
                leading_space: extended_fab_medium::LEADING_SPACE,
                trailing_space: extended_fab_medium::TRAILING_SPACE,
                icon_label_space: supplement::fab::MEDIUM_EXTENDED_ICON_LABEL_SPACE,
                label: supplement::fab::MEDIUM_EXTENDED_LABEL_FONT,
                shape: supplement::fab::MEDIUM_CONTAINER_SHAPE,
            },
            Self::Large => ExtendedSizeTokens {
                height: extended_fab_large::CONTAINER_HEIGHT,
                icon_size: supplement::fab::LARGE_ICON_SIZE,
                leading_space: extended_fab_large::LEADING_SPACE,
                trailing_space: extended_fab_large::TRAILING_SPACE,
                icon_label_space: supplement::fab::LARGE_EXTENDED_ICON_LABEL_SPACE,
                label: supplement::fab::LARGE_EXTENDED_LABEL_FONT,
                shape: extended_fab_large::CONTAINER_SHAPE,
            },
        }
    }
}

impl RenderOnce for ExtendedFab {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let size = self.size.tokens();
        let focus_handle = self
            .focus_handle
            .unwrap_or_else(|| util::keyed_focus_handle(&self.id, window, cx));
        // リングは容器の外側に描くため、FAB 自身のスタイルでは表せない
        let focus_visible = gpui_headless::is_focus_visible(&focus_handle, window, cx);
        let (_, content) = self.color.roles();
        let content = theme.color(content);

        // 展開した幅は、Compose と同じく余白・アイコン・間隔・ラベルを並べた幅にする。
        // 幅を補間するため、ラベルを描く書体で幅を先に測る
        let mut text_style = window.text_style();
        text_style.refine(&theme.text_style(size.label));
        let label_width = window
            .text_system()
            .shape_line(
                self.text.clone(),
                text_style.font_size.to_pixels(window.rem_size()),
                &[text_style.to_run(self.text.len())],
                None,
            )
            .width
            // gpui のテキストの要素は幅を切り上げてレイアウトする
            .ceil();
        let collapsed_width = dp(size.height);
        let expanded_width = (dp(size.leading_space)
            + dp(size.icon_size)
            + dp(size.icon_label_space)
            + label_width
            + dp(size.trailing_space))
        .max(collapsed_width);

        let mut label = div()
            .flex_none()
            .ml(dp(size.icon_label_space))
            .whitespace_nowrap()
            .text_color(content)
            .child(self.text.clone());
        label.text_style().refine(&theme.text_style(size.label));
        // Compose は幅を FastSpatial、ラベルの不透明度を FastEffects で変える
        let label = label.with_spring(
            "md3-extended-fab-label",
            SpringAnimation::new(theme.spring(MotionSchemeKey::FastEffects)).to(self.expanded),
            |this, phase| this.opacity(phase.0.clamp(0.0, 1.0)),
        );
        // 折りたたむ途中のラベルは容器の外にはみ出すため、内容を FAB の範囲で切り取る
        let row = div()
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .bottom_0()
            .overflow_hidden()
            .flex()
            .items_center()
            .pl(dp(size.leading_space))
            .pr(dp(size.trailing_space))
            .child(
                svg()
                    .data(self.icon.svg().as_bytes())
                    .flex_none()
                    .size(dp(size.icon_size))
                    .text_color(content),
            )
            .child(label);

        let button = HeadlessButton::new(self.id.clone())
            .track_focus(&focus_handle)
            .group(GROUP)
            .relative()
            .flex_none()
            .h(dp(size.height))
            .child(container_layer(
                &theme,
                self.color,
                size.shape,
                focus_visible,
            ))
            .child(row)
            .when(focus_visible, |this| {
                this.child(focus_ring(&theme, size.shape))
            })
            .aria_label(self.text)
            .when_some(self.on_click, |this, on_click| this.on_click(on_click));

        self.outer.child(button.with_spring(
            self.id,
            SpringAnimation::new(theme.spring(MotionSchemeKey::FastSpatial)).to(self.expanded),
            move |button, phase| {
                button.w(collapsed_width + (expanded_width - collapsed_width) * phase.0)
            },
        ))
    }
}

impl Styled for ExtendedFab {
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

    struct ElementView(Box<dyn Fn(&App) -> AnyElement>);

    impl Render for ElementView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            // ルートの要素はウィンドウいっぱいに広がるため、FAB を伸ばさない箱に包む
            div().flex().items_start().child((self.0)(cx))
        }
    }

    fn add_view(
        cx: &mut TestAppContext,
        element: impl Fn(&App) -> AnyElement + 'static,
    ) -> &mut VisualTestContext {
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            ElementView(Box::new(element))
        });
        cx
    }

    /// 描いた矩形の x, y, 幅, 高さと、左上の角丸の半径と塗り (論理ピクセル)
    type Quad = ((f32, f32, f32, f32), f32, gpui::Background);

    fn painted_quads(cx: &mut VisualTestContext) -> Vec<Quad> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            window
                .painted_quads()
                .into_iter()
                .map(|quad| {
                    let b = quad.bounds;
                    (
                        (
                            b.origin.x.0 / scale,
                            b.origin.y.0 / scale,
                            b.size.width.0 / scale,
                            b.size.height.0 / scale,
                        ),
                        quad.corner_radii.top_left.0 / scale,
                        quad.background,
                    )
                })
                .collect()
        })
    }

    /// 状態レイヤーを描かないよう、マウスを FAB の外に置く
    fn move_mouse_away(cx: &mut VisualTestContext) {
        cx.simulate_mouse_move(point(px(500.), px(500.)), None, Modifiers::none());
    }

    /// 期待値は Compose の各サイズの FabTokens の一辺と、形 (Small は CornerMedium の 12dp、Baseline は
    /// CornerLarge の 16dp、Medium は `ShapeDefaults.LargeIncreased` の 20dp、Large は CornerExtraLarge の 28dp)
    #[gpui::test]
    fn fab_は大きさに合わせた容器と形で描く(cx: &mut TestAppContext) {
        for (size, side, radius) in [
            (FabSize::Small, 40.0, 12.0),
            (FabSize::Baseline, 56.0, 16.0),
            (FabSize::Medium, 80.0, 20.0),
            (FabSize::Large, 96.0, 28.0),
        ] {
            let cx = add_view(cx, move |_| {
                Fab::new("fab", md3_icons::ADD, "Add")
                    .size(size)
                    .into_any_element()
            });
            move_mouse_away(cx);
            let ((x, y, width, height), r, _) = painted_quads(cx)[0];
            assert_eq!(
                (x, y, width, height, r),
                (0.0, 0.0, side, side, radius),
                "{size:?}"
            );
        }
    }

    /// 期待値は FabPrimaryContainerTokens の容器の色と、状態レイヤー (内容の色を 8% 重ねる)
    #[gpui::test]
    fn fab_は既定で_primary_container_を塗り_hover_で状態レイヤーを重ねる(
        cx: &mut TestAppContext,
    ) {
        let cx = add_view(cx, |_| {
            Fab::new("fab", md3_icons::ADD, "Add").into_any_element()
        });
        let (container, hovered) = cx.update(|_, cx| {
            let theme = cx.md3_theme();
            let container = theme.color(ColorRole::PrimaryContainer);
            let content = theme.color(ColorRole::OnPrimaryContainer);
            (
                gpui::Background::from(container),
                gpui::Background::from(container.blend(content.opacity(0.08))),
            )
        });
        move_mouse_away(cx);
        assert_eq!(painted_quads(cx)[0].2, container);
        cx.simulate_mouse_move(point(px(28.), px(28.)), None, Modifiers::none());
        assert_eq!(painted_quads(cx)[0].2, hovered);
    }

    #[gpui::test]
    fn fab_は押すとハンドラを呼ぶ(cx: &mut TestAppContext) {
        let clicks = Rc::new(Cell::new(0));
        let cx = add_view(cx, {
            let clicks = clicks.clone();
            move |_| {
                let clicks = clicks.clone();
                Fab::new("fab", md3_icons::ADD, "Add")
                    .on_click(move |_, _, _| clicks.set(clicks.get() + 1))
                    .into_any_element()
            }
        });
        cx.simulate_click(point(px(28.), px(28.)), Modifiers::none());
        assert_eq!(clicks.get(), 1);
    }

    #[gpui::test]
    fn tab_のフォーカスを容器の外側のリングで示す(cx: &mut TestAppContext) {
        let cx = add_view(cx, |_| {
            Fab::new("fab", md3_icons::ADD, "Add").into_any_element()
        });
        cx.update(|window, _| window.activate_window());
        move_mouse_away(cx);
        // リングは 56×56 の容器の外側 5px に描く
        let ring = |quads: Vec<Quad>| {
            quads
                .iter()
                .any(|(rect, _, _)| *rect == (-5., -5., 66., 66.))
        };
        assert!(
            !ring(painted_quads(cx)),
            "フォーカスの前はリングを描かないこと"
        );
        cx.simulate_keystrokes("tab");
        assert!(
            ring(painted_quads(cx)),
            "フォーカスが移るとリングを描くこと"
        );
    }

    /// 展開の状態と大きさを持つ Extended FAB と、比べる用にラベルと同じ書体で描いたラベルの文字列
    struct ExtendedView {
        expanded: bool,
        size: ExtendedFabSize,
        /// 比べる用のラベルの書体
        label: TypeScale,
    }

    impl Render for ExtendedView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let mut probe = div().flex_none().bg(gpui::black()).child("Compose");
            probe
                .text_style()
                .refine(&cx.md3_theme().text_style(self.label));
            div()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(100.))
                .child(
                    ExtendedFab::new("fab", md3_icons::EDIT, "Compose")
                        .size(self.size)
                        .expanded(self.expanded),
                )
                .child(probe)
        }
    }

    fn add_extended(
        cx: &mut TestAppContext,
        expanded: bool,
    ) -> (Entity<ExtendedView>, &mut VisualTestContext) {
        add_extended_with_size(cx, expanded, ExtendedFabSize::Small, TypeScale::TitleMedium)
    }

    fn add_extended_with_size(
        cx: &mut TestAppContext,
        expanded: bool,
        size: ExtendedFabSize,
        label: TypeScale,
    ) -> (Entity<ExtendedView>, &mut VisualTestContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            ExtendedView {
                expanded,
                size,
                label,
            }
        });
        move_mouse_away(cx);
        (view, cx)
    }

    /// 容器の幅と高さと、比べる用のラベルの幅。影のある容器は後の層に描かれるため、塗りで見分ける
    fn extended_widths(cx: &mut VisualTestContext) -> (f32, f32, f32) {
        let probe = gpui::Background::from(gpui::black());
        let quads = painted_quads(cx);
        let rect = |is_probe: bool| {
            quads
                .iter()
                .find(|(_, _, background)| (*background == probe) == is_probe)
                .map(|(rect, _, _)| *rect)
                .expect("描かれていること")
        };
        let (_, _, width, height) = rect(false);
        let (_, _, label_width, _) = rect(true);
        (width, height, label_width)
    }

    /// 期待値は Compose の `SmallExtendedFloatingActionButton` の、左の余白 16dp、アイコン 24dp、
    /// 間隔 8dp、右の余白 16dp (ExtendedFabSmallTokens) とラベルの幅の和
    #[gpui::test]
    fn 展開した_extended_fab_は余白とアイコンとラベルを並べた幅になる(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_extended(cx, true);
        let (width, height, label_width) = extended_widths(cx);
        assert_eq!(height, 56.0);
        assert_eq!(width, 16.0 + 24.0 + 8.0 + label_width + 16.0);
    }

    /// 期待値は Compose の Medium と Large の Extended FAB の高さ・左右の余白 (ExtendedFabMediumTokens、
    /// ExtendedFabLargeTokens)、FloatingActionButton.kt 末尾の private な定数 (`MediumExtendedFabIconPadding`・
    /// `LargeExtendedFabIconPadding` の間隔 12dp・16dp、`MediumExtendedFabTextStyle`・`LargeExtendedFabTextStyle` の書体)、
    /// サンプルが指定するアイコンの大きさ (`FloatingActionButtonDefaults.MediumIconSize` の 28dp、`LargeIconSize` の 36dp)
    #[gpui::test]
    fn medium_と_large_の_extended_fab_は大きさごとの寸法で並べる(
        cx: &mut TestAppContext,
    ) {
        for (size, label, height, side, icon, space) in [
            (
                ExtendedFabSize::Medium,
                TypeScale::TitleLarge,
                80.0,
                26.0,
                28.0,
                12.0,
            ),
            (
                ExtendedFabSize::Large,
                TypeScale::HeadlineSmall,
                96.0,
                28.0,
                36.0,
                16.0,
            ),
        ] {
            let (_, cx) = add_extended_with_size(cx, true, size, label);
            let (width, actual_height, label_width) = extended_widths(cx);
            assert_eq!(actual_height, height, "{size:?}");
            assert_eq!(width, side + icon + space + label_width + side, "{size:?}");
            // アイコンは描画から幅を測れず、Large ではアイコンと間隔を ExtendedFabLargeTokens の値
            // (32dp・20dp) に取り違えても和が変わらないため、寸法を直接確かめる
            let tokens = size.tokens();
            assert_eq!(
                (tokens.icon_size, tokens.icon_label_space),
                (Dp(icon), Dp(space)),
                "{size:?}"
            );
        }
    }

    /// 折りたたんだ幅は、Compose と同じく高さ (ExtendedFabSmallTokens.ContainerHeight) と同じ
    #[gpui::test]
    fn 折りたたんだ_extended_fab_は高さと同じ幅になる(cx: &mut TestAppContext) {
        let (_, cx) = add_extended(cx, false);
        let (width, height, _) = extended_widths(cx);
        assert_eq!((width, height), (56.0, 56.0));
    }

    #[gpui::test]
    fn モーションを減らす設定では折りたたんだ幅をすぐに描く(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = add_extended(cx, true);
        cx.update(|_, cx| cx.set_reduce_motion(true));
        view.update(cx, |view, cx| {
            view.expanded = false;
            cx.notify();
        });
        cx.run_until_parked();
        assert_eq!(extended_widths(cx).0, 56.0);
    }

    // gpui のバネは実時間 (`Instant::now`) で進み、テストの時計では進められないため、
    // 途中の幅は確かめず、十分な時間の後に着くことだけを確かめる
    #[gpui::test]
    fn 折りたたむと高さと同じ幅まで縮む(cx: &mut TestAppContext) {
        let (view, cx) = add_extended(cx, true);
        view.update(cx, |view, cx| {
            view.expanded = false;
            cx.notify();
        });
        cx.run_until_parked();
        // FastSpatial のバネは 1 秒以内に収まる
        std::thread::sleep(std::time::Duration::from_secs(1));
        view.update(cx, |_, cx| cx.notify());
        cx.run_until_parked();
        assert_eq!(extended_widths(cx).0, 56.0);
    }
}
