// 寸法と向きは Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Divider.kt
//         (`HorizontalDivider`、`VerticalDivider`、`DividerDefaults`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2022 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 の区切り線

use gpui::{App, IntoElement, Refineable, RenderOnce, StyleRefinement, Styled, Window, div};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::divider;
use md3_theme::{SharedTheme, ThemeExt, dp};

/// 区切り線を引く向き。線の太さを取る辺と、置く親の並べ方が決まる
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DividerOrientation {
    /// 横線。縦に並べる flex の親か、flex でない親の中に置く
    #[default]
    Horizontal,
    /// 縦線。横に並べる flex の親の中に置く
    Vertical,
}

/// MD3 の区切り線
///
/// 一覧や配置の中で内容のまとまりを分ける細い線を描く。
/// 線は装飾として扱い、支援技術には公開しない
///
/// 太さは 1dp で、線の長さは親から外側の余白を除いた長さになる。
/// [`Styled`] で指定したスタイルは線に付き、向きで決まる既定の大きさより優先する。
/// 字下げ (inset) は外側の余白 (`ml`、`mx` など) で、太さは高さ (横線) か幅 (縦線) で変える。
/// 色は [`Divider::color`] で変える
#[derive(IntoElement)]
pub struct Divider {
    orientation: DividerOrientation,
    style: StyleRefinement,
    color: ColorRole,
    theme: Option<SharedTheme>,
}

impl Divider {
    /// [`DividerOrientation::Horizontal`] の区切り線を作る
    pub fn new() -> Self {
        Self {
            orientation: DividerOrientation::default(),
            style: StyleRefinement::default(),
            color: divider::COLOR,
            theme: None,
        }
    }

    /// 線を引く向きを設定する
    pub fn orientation(mut self, orientation: DividerOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// 線の色を `color` にする。既定は [`ColorRole::OutlineVariant`]
    pub fn color(mut self, color: ColorRole) -> Self {
        self.color = color;
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl RenderOnce for Divider {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        // 長さは指定せず、交差軸に伸ばして外側の余白を除いた長さにする
        let line = div().flex_none().self_stretch();
        let mut line = match self.orientation {
            DividerOrientation::Horizontal => line.h(dp(divider::THICKNESS)),
            DividerOrientation::Vertical => line.w(dp(divider::THICKNESS)),
        }
        .bg(theme.color(self.color));
        line.style().refine(&self.style);
        line
    }
}

impl Default for Divider {
    fn default() -> Self {
        Self::new()
    }
}

impl Styled for Divider {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

#[cfg(test)]
mod tests {
    use gpui::{AnyElement, Context, Div, ParentElement, Render, TestAppContext};

    use super::*;

    struct ElementView(Box<dyn Fn() -> AnyElement>);

    impl Render for ElementView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            (self.0)()
        }
    }

    /// 矩形の論理ピクセルの x, y, 幅, 高さと塗り
    type Quad = ((f32, f32, f32, f32), gpui::Background);

    /// `parent` に `divider` を入れ、200×100 の大きさで描いたときの矩形と、描いたテーマの OutlineVariant の色
    fn painted_quads(
        cx: &mut TestAppContext,
        parent: fn() -> Div,
        divider: fn() -> Divider,
    ) -> (Vec<Quad>, gpui::Background) {
        let (_, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            ElementView(Box::new(move || {
                div()
                    .child(
                        parent()
                            .w(gpui::px(200.))
                            .h(gpui::px(100.))
                            .child(divider()),
                    )
                    .into_any_element()
            }))
        });
        cx.update(|window, cx| {
            let scale = window.scale_factor();
            let quads = window
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
                        quad.background,
                    )
                })
                .collect();
            (
                quads,
                cx.md3_theme().color(ColorRole::OutlineVariant).into(),
            )
        })
    }

    #[gpui::test]
    fn 横線は親の幅いっぱいに_1dp_の太さで描かれる(cx: &mut TestAppContext) {
        // 交差軸の揃え方に関わらず伸びる
        let (quads, color) =
            painted_quads(cx, || div().flex().flex_col().items_start(), Divider::new);
        assert_eq!(quads, vec![((0., 0., 200., 1.), color)], "縦に並べる flex");

        let (quads, color) = painted_quads(cx, div, Divider::new);
        assert_eq!(quads, vec![((0., 0., 200., 1.), color)], "flex でない親");
    }

    #[gpui::test]
    fn 縦線は親の高さいっぱいに_1dp_の太さで描かれる(cx: &mut TestAppContext) {
        let (quads, color) = painted_quads(
            cx,
            || div().flex().items_center(),
            || Divider::new().orientation(DividerOrientation::Vertical),
        );
        assert_eq!(quads, vec![((0., 0., 1., 100.), color)]);
    }

    #[gpui::test]
    fn 外側の余白で字下げできる(cx: &mut TestAppContext) {
        let (quads, color) = painted_quads(
            cx,
            || div().flex().flex_col(),
            || Divider::new().ml(gpui::px(16.)),
        );
        assert_eq!(quads, vec![((16., 0., 184., 1.), color)], "横線");

        let (quads, color) = painted_quads(
            cx,
            || div().flex(),
            || {
                Divider::new()
                    .orientation(DividerOrientation::Vertical)
                    .my(gpui::px(8.))
            },
        );
        assert_eq!(quads, vec![((0., 8., 1., 84.), color)], "縦線");
    }

    #[gpui::test]
    fn 色を変えられる(cx: &mut TestAppContext) {
        let (quads, _) = painted_quads(
            cx,
            || div().flex().flex_col(),
            || Divider::new().color(ColorRole::Primary),
        );
        let primary: gpui::Background =
            cx.update(|cx| cx.md3_theme().color(ColorRole::Primary).into());
        assert_eq!(quads, vec![((0., 0., 200., 1.), primary)]);
    }

    #[gpui::test]
    fn 指定した太さは向きの既定より優先する(cx: &mut TestAppContext) {
        // 向きは Styled の指定の後に変えても、指定した大きさを上書きしない
        let (quads, color) = painted_quads(
            cx,
            || div().flex(),
            || {
                Divider::new()
                    .w(gpui::px(2.))
                    .orientation(DividerOrientation::Vertical)
            },
        );
        assert_eq!(quads, vec![((0., 0., 2., 100.), color)]);
    }
}
