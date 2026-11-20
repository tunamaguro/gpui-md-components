// 配置と余白は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/AppBar.kt
//         (`BottomAppBar`、`BottomAppBarLayout`、`BottomAppBarDefaults`)、
//         FloatingActionButton.kt (`FloatingActionButtonDefaults.bottomAppBarFabElevation`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 のボトムアプリバー

use gpui::{
    AnyElement, App, Div, ElementId, InteractiveElement, IntoElement, ParentElement, Refineable,
    RenderOnce, StyleRefinement, Styled, Window, div, prelude::FluentBuilder,
};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::bottom_app_bar;
use md3_theme::md3_tokens::supplement::app_bar as supplement;
use md3_theme::{SharedTheme, ThemeExt, dp};

use crate::{Fab, IconButton};

/// MD3 のボトムアプリバー
///
/// 画面の下端に、操作を左から並べ、右端に FAB を置く。バーは置いた場所の幅に広がり、高さは 80dp。
/// 子の要素は操作として左から並べる。[`BottomAppBar::action`] で置いたアイコンボタンは、少なくとも 48dp 四方の枠の中央に置く。
///
/// バー自身は支援技術に公開されない。
/// Tab で到達できる操作と FAB は、この順に並ぶ
#[derive(IntoElement)]
pub struct BottomAppBar {
    id: ElementId,
    actions: Vec<Action>,
    fab: Option<Fab>,
    style: StyleRefinement,
    theme: Option<SharedTheme>,
}

impl BottomAppBar {
    /// 操作の無いバーを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            actions: Vec::new(),
            fab: None,
            style: StyleRefinement::default(),
            theme: None,
        }
    }

    /// `icon` を操作として、それまでに置いた子の要素の後に置く
    ///
    /// 有効な `icon` は、[`IconButton::colors`] でアイコンの色を指定すれば、どのバリアントでもその色で描く。
    /// 指定しなければ、Standard と Outlined は容器の上の内容の色 (OnSurface) で、Filled と FilledTonal はバリアントの色で描く。
    /// 無効な `icon` は [`IconButton`] の無効時の色で描く。
    /// [`IconButton::theme`] でテーマを指定しなければ、バーのテーマで描く
    pub fn action(mut self, icon: IconButton) -> Self {
        self.actions.push(Action::Icon(Box::new(icon)));
        self
    }

    /// 右端に `fab` を置く
    ///
    /// `fab` は影を描かずに、バーと同じ高さに描く。
    /// 色は [`FabColor::SecondaryContainer`](crate::FabColor::SecondaryContainer) を勧める。
    /// 大きさは [`FabSize::Small`](crate::FabSize::Small) か [`FabSize::Baseline`](crate::FabSize::Baseline) にすること。
    /// それより大きい FAB は縮めずに置くため、容器の下端からはみ出す
    pub fn floating_action_button(mut self, fab: Fab) -> Self {
        self.fab = Some(fab);
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    ///
    /// [`BottomAppBar::action`] のアイコンボタンと FAB は、それぞれでテーマを指定していなければこのテーマで描く。
    /// 子として置いた要素には渡らないため、子の要素ごとに指定すること
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl RenderOnce for BottomAppBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let mut bar: Div = div()
            .w_full()
            .flex()
            .items_center()
            .flex_shrink_0()
            .h(dp(bottom_app_bar::CONTAINER_HEIGHT))
            .pl(dp(supplement::BOTTOM_HORIZONTAL_PADDING))
            .pr(dp(supplement::BOTTOM_HORIZONTAL_PADDING))
            .pt(dp(supplement::BOTTOM_VERTICAL_PADDING))
            .bg(theme.color(bottom_app_bar::CONTAINER_COLOR))
            .child(div().flex_1().min_w_0().flex().items_center().children(
                self.actions.into_iter().map(|action| {
                    match action {
                        // Compose のアイコンボタンと同じ間隔で並べるため、48dp 四方の配置枠の中央に置く。
                        // 押せる範囲はアイコンボタン自身の大きさのまま
                        Action::Icon(icon) => div()
                            .flex()
                            .flex_none()
                            .items_center()
                            .justify_center()
                            .min_w(dp(supplement::ICON_TARGET_SIZE))
                            .min_h(dp(supplement::ICON_TARGET_SIZE))
                            .child(
                                icon.default_content(ColorRole::OnSurface)
                                    .default_theme(theme.clone()),
                            )
                            .into_any_element(),
                        Action::Element(element) => element,
                    }
                }),
            ))
            .when_some(self.fab, |this, fab| {
                // FAB は内容の上端に寄せて置く (`Alignment.TopStart`)
                this.child(
                    div()
                        .flex_none()
                        .self_stretch()
                        .pt(dp(supplement::BOTTOM_FAB_VERTICAL_PADDING))
                        .pr(dp(supplement::BOTTOM_FAB_HORIZONTAL_PADDING))
                        .child(fab.flat().default_theme(theme.clone())),
                )
            });
        bar.style().refine(&self.style);
        bar.id(self.id)
    }
}

impl Styled for BottomAppBar {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for BottomAppBar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.actions
            .extend(elements.into_iter().map(Action::Element));
    }
}

/// 左から並べる操作
enum Action {
    /// [`BottomAppBar::action`] で置いたアイコンボタン
    Icon(Box<IconButton>),
    /// 子として置いた要素
    Element(AnyElement),
}

#[cfg(test)]
mod tests {
    use gpui::{Context, Render, TestAppContext, VisualTestContext, px};

    use super::*;
    use crate::{ButtonColors, ButtonSize, FabColor};

    /// 幅 600 の中に `bar` が作るバーを置く
    struct BarView {
        bar: fn() -> BottomAppBar,
    }

    impl Render for BarView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div().w(px(600.0)).child((self.bar)())
        }
    }

    fn open(cx: &mut TestAppContext, bar: fn() -> BottomAppBar) -> &mut VisualTestContext {
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(move |window, cx| {
            cx.init_md3_theme(window);
            BarView { bar }
        });
        cx.run_until_parked();
        cx
    }

    /// 位置を測るため、容器を `role` の色で塗ったアイコンボタン
    fn button(id: &'static str, role: ColorRole) -> IconButton {
        IconButton::new(id, md3_icons::MORE_VERT, id).colors(ButtonColors {
            container: Some(role),
            ..ButtonColors::default()
        })
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

    /// 期待値は Compose の `BottomAppBar`。内容は左右と上に 4dp の余白を取り、残りの 76dp の中央に並ぶ。
    /// アイコンボタンの容器 (40dp) は 48dp の配置枠の中央にある
    #[gpui::test]
    fn 操作を左から_48dp_ごとに並べる(cx: &mut TestAppContext) {
        let cx = open(cx, || {
            BottomAppBar::new("bar")
                .action(button("a", ColorRole::Primary))
                .action(button("b", ColorRole::Primary))
        });
        assert_eq!(
            filled(cx, ColorRole::SurfaceContainer),
            vec![(0.0, 0.0, 600.0, 80.0)]
        );
        assert_eq!(
            filled(cx, ColorRole::Primary),
            vec![(8.0, 22.0, 40.0, 40.0), (56.0, 22.0, 40.0, 40.0)]
        );
    }

    /// FAB は容器の右端から 16dp、上端から 12dp に置く
    #[gpui::test]
    fn fab_を右上に置く(cx: &mut TestAppContext) {
        let cx = open(cx, || {
            BottomAppBar::new("bar")
                .action(button("a", ColorRole::Primary))
                .floating_action_button(
                    Fab::new("fab", md3_icons::ADD, "Add").color(FabColor::SecondaryContainer),
                )
        });
        assert_eq!(
            filled(cx, ColorRole::SecondaryContainer),
            vec![(528.0, 12.0, 56.0, 56.0)]
        );
        assert_eq!(
            filled(cx, ColorRole::Primary),
            vec![(8.0, 22.0, 40.0, 40.0)]
        );
    }

    #[gpui::test]
    fn 子の要素は操作の並びに置いた順で並ぶ(cx: &mut TestAppContext) {
        let cx = open(cx, || {
            BottomAppBar::new("bar")
                .action(button("a", ColorRole::Primary))
                .child(div().size(px(20.0)).bg(gpui::red()))
                .action(button("b", ColorRole::Primary))
        });
        // 子の要素は配置枠を持たず、内容の高さの中央に置く
        assert_eq!(
            filled(cx, ColorRole::Primary),
            vec![(8.0, 22.0, 40.0, 40.0), (76.0, 22.0, 40.0, 40.0)]
        );
    }

    /// 配置枠は 48dp を下限にアイコンボタンの大きさまで広がり、大きなボタンどうしも重ならない
    #[gpui::test]
    fn _48dp_より大きいアイコンボタンは重ならずに並ぶ(cx: &mut TestAppContext) {
        let cx = open(cx, || {
            BottomAppBar::new("bar")
                .action(button("a", ColorRole::Primary).size(ButtonSize::Medium))
                .action(button("b", ColorRole::Primary).size(ButtonSize::Medium))
        });
        let rects = filled(cx, ColorRole::Primary);
        assert_eq!(rects.len(), 2);
        assert!(
            rects[0].0 + rects[0].2 <= rects[1].0,
            "隣のボタンと重ならないこと: {rects:?}"
        );
    }

    /// バーは縦の flex の中で、内容が親の高さを超えても縮まない
    #[gpui::test]
    fn 縦の_flex_の中でも高さを保つ(cx: &mut TestAppContext) {
        struct ColumnView;
        impl Render for ColumnView {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div()
                    .w(px(600.0))
                    .h(px(200.0))
                    .flex()
                    .flex_col()
                    .child(div().h(px(400.0)))
                    .child(BottomAppBar::new("bar"))
            }
        }
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            ColumnView
        });
        cx.run_until_parked();
        let rects = filled(cx, ColorRole::SurfaceContainer);
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].3, 80.0);
    }
}
