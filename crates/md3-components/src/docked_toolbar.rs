// 配置と色は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/AppBar.kt
//         (`FlexibleBottomAppBar`、`BottomAppBarLayout`、`BottomAppBarDefaults`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 Expressive のドッキングツールバー

use gpui::{
    AnyElement, App, ElementId, IntoElement, ParentElement, Refineable, RenderOnce, SharedString,
    StatefulInteractiveElement, StyleRefinement, Styled, Window, div, prelude::FluentBuilder,
};
use gpui_headless::toolbar::Toolbar;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::docked_toolbar;
use md3_theme::{SharedTheme, ThemeExt, dp};

use crate::IconButton;

/// ドッキングツールバーの操作の並べ方
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DockedToolbarArrangement {
    /// 先頭の操作を左端、末尾の操作を右端に置き、残りの幅を操作の間に等しく分ける
    #[default]
    SpaceBetween,
    /// 操作を 32dp の間隔で中央に寄せる。幅が足りなければ間隔を 0 まで詰め、それでも足りなければ左端から並べる。
    /// 間隔は要素で作るため、[`Styled`] の `gap` を指定するとその分だけ間隔が広がり、詰めることもできない
    Centered,
}

/// MD3 Expressive のドッキングツールバー (Compose の `FlexibleBottomAppBar`)
///
/// 画面の端に付けて置く、操作を横に並べるバー。バーは置いた場所の幅に広がり、高さは 64dp で、縦にも横にも縮まない。
/// 左右に 16dp の余白を取り、操作は [`DockedToolbar::arrangement`] に従って並べる。
/// [`DockedToolbar::action`] で置いたアイコンボタンは、少なくとも 48dp 四方の枠の中央に置く。押せる範囲はアイコンボタン自身の大きさのまま。
/// 子の要素は操作の間に、置いた順に並ぶ。子の要素には区切り線のようなフォーカスを持たない要素を渡すこと。
/// 幅に収まらない操作はバーの右端を越えて描かれる。隠れた操作にも矢印キーで移れるため、操作の数と大きさは幅に収まるように選ぶこと。
/// 背景を [`Styled`] で変えても [`DockedToolbar::action`] のアイコンの色は変わらないため、
/// 背景に合わない場合はアイコンボタンごとに [`IconButton::colors`] で指定すること
///
/// 支援技術には role `toolbar` と横の向き、[`DockedToolbar::new`] の `label` を名前として公開される。
/// フォーカスできる操作があれば、そのうち Tab で到達できるのは 1 つだけで、
/// 修飾キーなしの左右の矢印キーで前後の操作へフォーカスを移す。端では反対の端に戻り、無効な操作は飛ばす。
/// メニューを持つ操作でも、移れる操作がほかにあれば左右の矢印キーは移動に使う
#[derive(IntoElement)]
pub struct DockedToolbar {
    id: ElementId,
    label: SharedString,
    arrangement: DockedToolbarArrangement,
    children: Vec<Child>,
    style: StyleRefinement,
    theme: Option<SharedTheme>,
}

impl DockedToolbar {
    /// 操作の無いツールバーを作る
    ///
    /// `label` はツールバーが何の操作をまとめるかを表す名前にする。画面には表示しない。
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            arrangement: DockedToolbarArrangement::default(),
            children: Vec::new(),
            style: StyleRefinement::default(),
            theme: None,
        }
    }

    /// 操作の並べ方を設定する
    pub fn arrangement(mut self, arrangement: DockedToolbarArrangement) -> Self {
        self.arrangement = arrangement;
        self
    }

    /// `icon` を操作として、それまでに置いた操作と子の要素の後に置く
    ///
    /// 有効な `icon` は、[`IconButton::colors`] でアイコンの色を指定すれば、どのバリアントでもその色で描く。
    /// 指定しなければ、Standard と Outlined は容器の上の内容の色 (OnSurface) で、Filled と FilledTonal はバリアントの色で描く。
    /// 無効な `icon` は [`IconButton`] の無効時の色で描く。
    /// [`IconButton::theme`] でテーマを指定しなければ、ツールバーのテーマで描く。
    /// [`IconButton::track_focus`] を指定しなければ、フォーカスは `icon` の id とツールバーの id で決まる
    pub fn action(mut self, icon: IconButton) -> Self {
        self.children.push(Child::Icon(Box::new(icon)));
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    ///
    /// [`DockedToolbar::action`] のアイコンボタンは、それぞれでテーマを指定していなければこのテーマで描く。
    /// 子として置いた要素には渡らないため、子の要素ごとに指定すること
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl RenderOnce for DockedToolbar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let mut toolbar = Toolbar::new(self.id.clone())
            .aria_label(self.label)
            .w_full()
            .flex()
            .flex_shrink_0()
            .items_center()
            .h(dp(docked_toolbar::CONTAINER_HEIGHT))
            .pl(dp(docked_toolbar::CONTAINER_LEADING_SPACE))
            .pr(dp(docked_toolbar::CONTAINER_TRAILING_SPACE))
            // 幅が足りない場合、space-between は左端に寄せる
            .when(
                self.arrangement == DockedToolbarArrangement::SpaceBetween,
                |this| this.justify_between(),
            )
            // CONTAINER_SHAPE は CornerNone なので角を丸めない
            .bg(theme.color(docked_toolbar::CONTAINER_COLOR));
        // Compose の `FlexibleFixedHorizontalArrangement` (`spacedBy` の中央寄せ)。
        // 操作の間には 32dp から縮む空きを、両端には余った幅を分け合う空きを置く。
        // 幅が足りなければ操作の間の空きから縮め、両端の空きは 0 になって左端から並ぶ
        let centered = self.arrangement == DockedToolbarArrangement::Centered;
        if centered {
            toolbar = toolbar.child(div().flex_1().min_w_0());
        }
        for (ix, child) in self.children.into_iter().enumerate() {
            if centered && ix > 0 {
                toolbar = toolbar.child(
                    div()
                        .flex_shrink(1.0)
                        .flex_basis(dp(docked_toolbar::CONTAINER_MAX_SPACING))
                        .min_w_0(),
                );
            }
            toolbar = match child {
                Child::Icon(icon) => toolbar.item(icon.into_toolbar_item(
                    &self.id,
                    // Compose は容器の色に対応する内容の色 (`contentColorFor`) を使う
                    ColorRole::OnSurface,
                    &theme,
                    window,
                    cx,
                )),
                Child::Element(element) => toolbar.child(element),
            };
        }
        if centered {
            toolbar = toolbar.child(div().flex_1().min_w_0());
        }
        toolbar.style().refine(&self.style);
        toolbar
    }
}

impl Styled for DockedToolbar {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for DockedToolbar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children
            .extend(elements.into_iter().map(Child::Element));
    }
}

/// 置いた順に並べる子
enum Child {
    /// [`DockedToolbar::action`] で置いたアイコンボタン
    Icon(Box<IconButton>),
    /// 子として置いた要素
    Element(AnyElement),
}

#[cfg(test)]
mod tests {
    use gpui::{Context, Render, TestAppContext, VisualTestContext, px};

    use super::*;
    use crate::ButtonColors;

    /// 位置を測るため、容器を `role` の色で塗ったアイコンボタン
    fn button(id: &'static str, role: ColorRole) -> IconButton {
        IconButton::new(id, md3_icons::MORE_VERT, id).colors(ButtonColors {
            container: Some(role),
            ..ButtonColors::default()
        })
    }

    /// 幅 600 の縦の flex の中に `toolbar` が作るツールバーと、高さを奪い合う要素を置く
    struct ToolbarView {
        toolbar: fn() -> DockedToolbar,
    }

    impl Render for ToolbarView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .w(px(600.0))
                .h(px(100.0))
                .flex()
                .flex_col()
                .child(div().h(px(100.0)))
                .child((self.toolbar)())
        }
    }

    fn open(cx: &mut TestAppContext, toolbar: fn() -> DockedToolbar) -> &mut VisualTestContext {
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(move |window, cx| {
            cx.init_md3_theme(window);
            ToolbarView { toolbar }
        });
        cx.run_until_parked();
        cx
    }

    /// `role` の色で塗った矩形の x・y・幅・高さ。x の順に並べる
    fn filled(cx: &mut VisualTestContext, role: ColorRole) -> Vec<(f32, f32, f32, f32)> {
        cx.update(|window, cx| {
            let scale = window.scale_factor();
            let background: gpui::Background = cx.md3_theme().color(role).into();
            let mut rects: Vec<_> = window
                .painted_quads()
                .into_iter()
                .filter(|quad| quad.background == background)
                .map(|quad| {
                    let b = quad.bounds;
                    (
                        b.origin.x.0 / scale,
                        b.origin.y.0 / scale,
                        b.size.width.0 / scale,
                        b.size.height.0 / scale,
                    )
                })
                .collect();
            rects.sort_by(|a, b| a.0.total_cmp(&b.0));
            rects
        })
    }

    /// 期待値は Compose の `FlexibleBottomAppBar`。容器は 64dp で、高さを奪い合う縦の flex の中でも縮まず、
    /// 前の要素 (100) が 36 に縮む。
    /// 操作は左右 16dp の余白の内側で両端に寄り、アイコンボタンの容器 (40dp) は 48dp の枠の中央にある
    #[gpui::test]
    fn 既定では操作を両端に寄せて並べる(cx: &mut TestAppContext) {
        let cx = open(cx, || {
            DockedToolbar::new("toolbar", "Edit")
                .action(button("a", ColorRole::Primary))
                .action(button("b", ColorRole::Primary))
                .action(button("c", ColorRole::Primary))
        });
        assert_eq!(
            filled(cx, ColorRole::SurfaceContainer),
            vec![(0.0, 36.0, 600.0, 64.0)]
        );
        assert_eq!(
            filled(cx, ColorRole::Primary),
            vec![
                (20.0, 48.0, 40.0, 40.0),
                (280.0, 48.0, 40.0, 40.0),
                (540.0, 48.0, 40.0, 40.0)
            ]
        );
    }

    /// 期待値は Compose の `FlexibleFixedHorizontalArrangement`。48dp の枠を 32dp の間隔で中央に寄せる
    #[gpui::test]
    fn centered_は_32dp_の間隔で中央に寄せる(cx: &mut TestAppContext) {
        let cx = open(cx, || {
            DockedToolbar::new("toolbar", "Edit")
                .arrangement(DockedToolbarArrangement::Centered)
                .action(button("a", ColorRole::Primary))
                .action(button("b", ColorRole::Primary))
        });
        // 枠の幅の合計は 48 + 32 + 48 = 128 で、左端は (600 - 128) / 2 = 236
        assert_eq!(
            filled(cx, ColorRole::Primary),
            vec![(240.0, 48.0, 40.0, 40.0), (320.0, 48.0, 40.0, 40.0)]
        );
    }

    /// 期待値は 6 つの 48dp の枠 (288) が余白を除いた幅 (288) にちょうど収まるため、間隔を 0 に詰めて並べる
    #[gpui::test]
    fn centered_は幅が足りなければ間隔を詰める(cx: &mut TestAppContext) {
        let cx = open(cx, || {
            DockedToolbar::new("toolbar", "Edit")
                .w(px(320.0))
                .arrangement(DockedToolbarArrangement::Centered)
                .action(button("a", ColorRole::Primary))
                .action(button("b", ColorRole::Primary))
                .action(button("c", ColorRole::Primary))
                .action(button("d", ColorRole::Primary))
                .action(button("e", ColorRole::Primary))
                .action(button("f", ColorRole::Primary))
        });
        let xs: Vec<f32> = filled(cx, ColorRole::Primary)
            .into_iter()
            .map(|rect| rect.0)
            .collect();
        assert_eq!(xs, vec![20.0, 68.0, 116.0, 164.0, 212.0, 260.0]);
    }

    /// 操作の間を左右の矢印キーで移る
    #[gpui::test]
    fn 左右の矢印キーで操作の間を移る(cx: &mut TestAppContext) {
        struct ClickView {
            clicked: Vec<&'static str>,
        }
        impl Render for ClickView {
            fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                let record = |name: &'static str| {
                    cx.listener(move |this: &mut Self, _: &gpui::ClickEvent, _, _| {
                        this.clicked.push(name)
                    })
                };
                DockedToolbar::new("toolbar", "Edit")
                    .action(
                        IconButton::new("cut", md3_icons::MORE_VERT, "Cut").on_click(record("cut")),
                    )
                    .action(
                        IconButton::new("copy", md3_icons::MORE_VERT, "Copy")
                            .on_click(record("copy")),
                    )
            }
        }
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            ClickView {
                clicked: Vec::new(),
            }
        });
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        cx.simulate_keystrokes("tab right");
        // ボタンはキーを離したときに押される
        let enter = gpui::Keystroke::parse("enter").expect("キーを読めること");
        cx.simulate_event(gpui::KeyDownEvent {
            keystroke: enter.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(gpui::KeyUpEvent { keystroke: enter });
        cx.run_until_parked();
        assert_eq!(
            view.read_with(cx, |view, _| view.clicked.clone()),
            vec!["copy"]
        );
    }
}
