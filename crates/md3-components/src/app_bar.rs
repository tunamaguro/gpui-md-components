// 配置・余白・タイトルの置き方は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/AppBar.kt
//         (`TopAppBar`、`CenterAlignedTopAppBar`、`MediumFlexibleTopAppBar`、`LargeFlexibleTopAppBar`、
//         `SingleRowTopAppBar`、`TwoRowsTopAppBar`、`TopAppBarLayout`、`TopAppBarMeasurePolicy`、`TopAppBarDefaults`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 のトップアプリバー

use gpui::{
    AnyElement, App, Div, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels,
    Refineable, RenderOnce, SharedString, StyleRefinement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use md3_theme::md3_tokens::component::{
    app_bar, app_bar_large_flexible, app_bar_medium_flexible, app_bar_small,
};
use md3_theme::md3_tokens::supplement::app_bar as supplement;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{SharedTheme, Theme, ThemeExt, dp};

use crate::IconButton;

/// トップアプリバーの大きさ
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TopAppBarVariant {
    /// ナビゲーションのアイコン・タイトル・操作を高さ 64dp の 1 行に並べる
    #[default]
    Small,
    /// ナビゲーションのアイコンと操作の行の下に、大きなタイトルの行を置く
    Medium,
    /// Medium よりさらに大きなタイトルの行を置く
    Large,
}

/// タイトルの横の位置
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TopAppBarTitleAlignment {
    /// 左端に寄せる
    #[default]
    Start,
    /// バーの幅の中央に置く。ナビゲーションのアイコンや操作に重なる場合は、重ならない位置までずらす
    Center,
}

/// MD3 のトップアプリバー
///
/// 画面の上端に、ナビゲーションのアイコン・タイトル・操作を並べる。子の要素は操作として右端に並べる。
/// 操作には [`IconButton`] を置く。ナビゲーションのアイコンと操作は、それぞれ 48dp 四方の枠の中央に置く。
/// バーは置いた場所の幅に広がる。タイトルと補足の文は、Small では 1 行に収まらない部分を省略し、
/// Medium と Large では折り返す。
///
/// タイトルと補足の文は支援技術にテキストとして公開される。
/// ナビゲーションのアイコンと操作は Tab でこの順に到達できる
#[derive(IntoElement)]
pub struct TopAppBar {
    id: ElementId,
    title: SharedString,
    subtitle: Option<SharedString>,
    variant: TopAppBarVariant,
    alignment: TopAppBarTitleAlignment,
    navigation_icon: Option<IconButton>,
    actions: Vec<AnyElement>,
    style: StyleRefinement,
    theme: Option<SharedTheme>,
}

impl TopAppBar {
    /// `title` をタイトルにしたバーを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            subtitle: None,
            variant: TopAppBarVariant::default(),
            alignment: TopAppBarTitleAlignment::default(),
            navigation_icon: None,
            actions: Vec::new(),
            style: StyleRefinement::default(),
            theme: None,
        }
    }

    /// タイトルの下に `subtitle` を補足の文として置く
    ///
    /// Medium と Large では、補足の文の分だけバーが高くなる
    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// バーの大きさを設定する
    pub fn variant(mut self, variant: TopAppBarVariant) -> Self {
        self.variant = variant;
        self
    }

    /// タイトルの横の位置を設定する
    pub fn title_alignment(mut self, alignment: TopAppBarTitleAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// 左端に `icon` をナビゲーションのアイコンとして置く
    ///
    /// [`IconButton::colors`] でアイコンの色を指定しなければ、ナビゲーションのアイコンの色で描く
    pub fn navigation_icon(mut self, icon: IconButton) -> Self {
        self.navigation_icon = Some(icon);
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

/// `scale` の書体で描いた 1 行の、行の上端から基準線までの距離と行の高さ
///
/// gpui は行の高さの中で文字の上下の余白を等しくして基準線を置く
fn baseline(
    theme: &Theme,
    scale: TypeScale,
    text: &SharedString,
    window: &Window,
) -> (Pixels, Pixels) {
    let mut style = window.text_style();
    style.refine(&theme.text_style(scale));
    let font_size = style.font_size.to_pixels(window.rem_size());
    let line_height = style.line_height_in_pixels(window.rem_size());
    let line =
        window
            .text_system()
            .shape_line(text.clone(), font_size, &[style.to_run(text.len())], None);
    (
        (line_height - line.ascent - line.descent) / 2.0 + line.ascent,
        line_height,
    )
}

impl RenderOnce for TopAppBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let edge = dp(app_bar::LEADING_SPACE);
        let centered = self.alignment == TopAppBarTitleAlignment::Center;
        let (title_font, subtitle_font) = match self.variant {
            TopAppBarVariant::Small => (app_bar_small::TITLE_FONT, app_bar_small::SUBTITLE_FONT),
            TopAppBarVariant::Medium => (
                app_bar_medium_flexible::TITLE_FONT,
                app_bar_medium_flexible::SUBTITLE_FONT,
            ),
            TopAppBarVariant::Large => (
                app_bar_large_flexible::TITLE_FONT,
                app_bar_large_flexible::SUBTITLE_FONT,
            ),
        };
        let text = |font: TypeScale, color: Hsla, id: &'static str, content: SharedString| {
            let mut element = div().text_color(color);
            element.text_style().refine(&theme.text_style(font));
            element
                .when(self.variant == TopAppBarVariant::Small, |this| {
                    this.truncate()
                })
                .when(centered, |this| this.text_center())
                .child(gpui::text!(id = id, content))
        };
        let title = div()
            .debug_selector(|| "md3-app-bar-title".into())
            .flex()
            .flex_col()
            .min_w_0()
            .px(edge)
            .when(centered, |this| this.items_center())
            .child(text(
                title_font,
                theme.color(app_bar::TITLE_COLOR),
                "title",
                self.title.clone(),
            ))
            .when_some(self.subtitle.clone(), |this, subtitle| {
                this.child(text(
                    subtitle_font,
                    theme.color(app_bar::SUBTITLE_COLOR),
                    "subtitle",
                    subtitle,
                ))
            });

        // Compose のアイコンボタンと同じ間隔で並べるため、アイコンボタンを押せる範囲の大きさの枠の中央に置く
        let target = |element: AnyElement| {
            div()
                .flex()
                .flex_none()
                .items_center()
                .justify_center()
                .min_w(dp(supplement::ICON_TARGET_SIZE))
                .min_h(dp(supplement::ICON_TARGET_SIZE))
                .child(element)
        };
        // 行の左端。ナビゲーションのアイコンが無くてもタイトルの前に `TITLE_INSET` を空ける
        let leading = |navigation: Option<IconButton>| {
            div()
                .flex()
                .items_center()
                .flex_none()
                .min_w(dp(supplement::TITLE_INSET))
                .when_some(navigation, |this, icon| {
                    this.pl(edge).child(target(
                        icon.default_content(app_bar::LEADING_ICON_COLOR)
                            .theme(theme.clone())
                            .into_any_element(),
                    ))
                })
        };
        let trailing = |actions: Vec<AnyElement>| {
            div()
                .flex()
                .items_center()
                .flex_none()
                .gap(dp(app_bar::ICON_BUTTON_SPACE))
                .pr(dp(app_bar::TRAILING_SPACE))
                .children(actions.into_iter().map(target))
        };
        // タイトルと両端を 1 行に並べる。中央に置くときは、両端を同じ割合で伸ばしてタイトルを行の中央に置く。
        // 両端は中身より狭くならないため、タイトルが中身に重なる場合は重ならない位置までずれる
        let row = |leading: Div, title: Option<Div>, trailing: Div| {
            let row = div().flex().items_center().w_full();
            if centered {
                row.child(div().flex().flex_1().flex_basis(px(0.0)).child(leading))
                    .children(title.map(|title| title.flex_shrink(1.0)))
                    .child(
                        div()
                            .flex()
                            .flex_1()
                            .flex_basis(px(0.0))
                            .justify_end()
                            .child(trailing),
                    )
            } else {
                row.child(leading)
                    .child(div().flex_1().min_w_0().flex().children(title))
                    .child(trailing)
            }
        };

        let collapsed_height = dp(app_bar_small::CONTAINER_HEIGHT);
        let bar = div()
            .id(self.id)
            .w_full()
            .flex()
            .flex_col()
            .bg(theme.color(app_bar::CONTAINER_COLOR));
        let bar = match self.variant {
            TopAppBarVariant::Small => bar.child(
                row(
                    leading(self.navigation_icon),
                    Some(title),
                    trailing(self.actions),
                )
                .min_h(collapsed_height),
            ),
            TopAppBarVariant::Medium | TopAppBarVariant::Large => {
                let has_subtitle = self.subtitle.is_some();
                let (expanded_height, bottom_padding) = match self.variant {
                    TopAppBarVariant::Medium => (
                        if has_subtitle {
                            app_bar_medium_flexible::LARGE_CONTAINER_HEIGHT
                        } else {
                            app_bar_medium_flexible::CONTAINER_HEIGHT
                        },
                        supplement::MEDIUM_TITLE_BOTTOM_PADDING,
                    ),
                    _ => (
                        if has_subtitle {
                            app_bar_large_flexible::LARGE_CONTAINER_HEIGHT
                        } else {
                            app_bar_large_flexible::CONTAINER_HEIGHT
                        },
                        supplement::LARGE_TITLE_BOTTOM_PADDING,
                    ),
                };
                // 下の行は、タイトルの最後の行 (補足の文があればその行) の基準線を下端から決まった距離に置く。
                // 行の高さに収まらなければ、収まるまで距離を縮める。高さはタイトルと補足の文を 1 行ずつとして見積もる
                let (title_baseline, title_line_height) =
                    baseline(&theme, title_font, &self.title, window);
                let (last_baseline, last_line_height, block_height) = match &self.subtitle {
                    Some(subtitle) => {
                        let (subtitle_baseline, subtitle_line_height) =
                            baseline(&theme, subtitle_font, subtitle, window);
                        (
                            subtitle_baseline,
                            subtitle_line_height,
                            title_line_height + subtitle_line_height,
                        )
                    }
                    None => (title_baseline, title_line_height, title_line_height),
                };
                let row_height = dp(expanded_height) - collapsed_height;
                let title_bottom = (dp(bottom_padding) - (last_line_height - last_baseline))
                    .min(row_height - block_height)
                    .max(px(0.0));
                bar.child(
                    row(leading(self.navigation_icon), None, trailing(self.actions))
                        .h(collapsed_height),
                )
                .child(
                    row(leading(None), Some(title), trailing(Vec::new()))
                        .items_end()
                        .min_h(row_height)
                        .pb(title_bottom),
                )
            }
        };
        let mut bar = bar;
        bar.style().refine(&self.style);
        bar
    }
}

impl Styled for TopAppBar {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for TopAppBar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.actions.extend(elements);
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Context, Render, TestAppContext, VisualTestContext};
    use md3_theme::md3_color::ColorRole;

    use super::*;
    use crate::ButtonColors;

    /// 幅 `width` の中に `bar` が作るバーを置く
    struct BarView {
        bar: fn() -> TopAppBar,
        width: f32,
    }

    impl Render for BarView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div().w(px(self.width)).child((self.bar)())
        }
    }

    fn open(cx: &mut TestAppContext, width: f32, bar: fn() -> TopAppBar) -> &mut VisualTestContext {
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(move |window, cx| {
            cx.init_md3_theme(window);
            BarView { bar, width }
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

    fn small() -> TopAppBar {
        TopAppBar::new("bar", "Title")
            .navigation_icon(button("navigation", ColorRole::Primary))
            .child(button("first", ColorRole::Tertiary))
            .child(button("second", ColorRole::Tertiary))
    }

    #[gpui::test]
    fn small_は高さ_64dp_の_surface_の容器に左右_4dp_を空けてアイコンを並べる(
        cx: &mut TestAppContext,
    ) {
        let cx = open(cx, 600.0, small);
        assert_eq!(
            filled(cx, ColorRole::Surface),
            vec![(0.0, 0.0, 600.0, 64.0)]
        );
        let navigation = filled(cx, ColorRole::Primary);
        let actions = filled(cx, ColorRole::Tertiary);
        assert_eq!(navigation.len(), 1);
        assert_eq!(actions.len(), 2);
        // アイコンボタンの容器は 48dp の押せる範囲の中央にある
        let inset = (48.0 - navigation[0].2) / 2.0;
        assert_eq!(navigation[0].0, 4.0 + inset);
        assert_eq!(navigation[0].1, (64.0 - navigation[0].3) / 2.0);
        assert_eq!(actions[1].0 + actions[1].2, 600.0 - 4.0 - inset);
        assert_eq!(actions[1].0 - actions[0].0, 48.0, "操作の間は空けないこと");
    }

    // gpui は debug_assertions のあるビルドでだけ debug_selector の位置を記録する
    #[cfg(debug_assertions)]
    #[gpui::test]
    fn タイトルはナビゲーションのアイコンの後ろか左端から_16dp_に置く(
        app: &mut TestAppContext,
    ) {
        let cx = open(app, 600.0, small);
        let title = cx
            .debug_bounds("md3-app-bar-title")
            .expect("タイトルを描くこと");
        // ナビゲーションのアイコンの領域 4 + 48 の後ろに、タイトルの領域の余白 4 を取る
        assert_eq!(f32::from(title.origin.x), 52.0);

        let cx = open(app, 600.0, || TopAppBar::new("bar", "Title"));
        let title = cx
            .debug_bounds("md3-app-bar-title")
            .expect("タイトルを描くこと");
        assert_eq!(f32::from(title.origin.x), 12.0);
    }

    #[cfg(debug_assertions)]
    #[gpui::test]
    fn 中央にそろえたタイトルはバーの幅の中央に置き_操作に重なればずらす(
        app: &mut TestAppContext,
    ) {
        let cx = open(app, 600.0, || {
            small().title_alignment(TopAppBarTitleAlignment::Center)
        });
        let title = cx
            .debug_bounds("md3-app-bar-title")
            .expect("タイトルを描くこと");
        assert_eq!(f32::from(title.center().x), 300.0);

        let cx = open(app, 320.0, || {
            TopAppBar::new("bar", "A long centered title")
                .title_alignment(TopAppBarTitleAlignment::Center)
                .child(button("first", ColorRole::Tertiary))
                .child(button("second", ColorRole::Tertiary))
                .child(button("third", ColorRole::Tertiary))
        });
        let title = cx
            .debug_bounds("md3-app-bar-title")
            .expect("タイトルを描くこと");
        let actions = filled(cx, ColorRole::Tertiary);
        let inset = (48.0 - actions[0].2) / 2.0;
        assert!(
            f32::from(title.right()) <= actions[0].0 - inset,
            "タイトルが操作に重ならないこと: {title:?} {actions:?}"
        );
        assert!(f32::from(title.origin.x) >= 12.0);
    }

    #[gpui::test]
    fn medium_と_large_は補足の文の有無で高さを変える(cx: &mut TestAppContext) {
        let cases: [(fn() -> TopAppBar, f32); 4] = [
            (
                || TopAppBar::new("bar", "Title").variant(TopAppBarVariant::Medium),
                112.0,
            ),
            (
                || {
                    TopAppBar::new("bar", "Title")
                        .variant(TopAppBarVariant::Medium)
                        .subtitle("Subtitle")
                },
                136.0,
            ),
            (
                || TopAppBar::new("bar", "Title").variant(TopAppBarVariant::Large),
                120.0,
            ),
            (
                || {
                    TopAppBar::new("bar", "Title")
                        .variant(TopAppBarVariant::Large)
                        .subtitle("Subtitle")
                },
                152.0,
            ),
        ];
        for (bar, height) in cases {
            let cx = open(cx, 600.0, bar);
            assert_eq!(
                filled(cx, ColorRole::Surface),
                vec![(0.0, 0.0, 600.0, height)]
            );
        }
    }

    /// 期待値は Compose の `placeTopAppBar`。最後の行の基準線から下端までを `distance` にし、
    /// 下の行の高さに収まらなければ収まるまで縮める
    // gpui は debug_assertions のあるビルドでだけ debug_selector の位置を記録する
    #[cfg(debug_assertions)]
    #[gpui::test]
    fn medium_と_large_は最後の行の基準線を下端から決まった距離に置き_収まらなければ縮める(
        app: &mut TestAppContext,
    ) {
        type Case = (fn() -> TopAppBar, f32, TypeScale, Option<TypeScale>, f32);
        let cases: [Case; 2] = [
            (
                || TopAppBar::new("bar", "Title").variant(TopAppBarVariant::Medium),
                112.0,
                app_bar_medium_flexible::TITLE_FONT,
                None,
                24.0,
            ),
            (
                || {
                    TopAppBar::new("bar", "Title")
                        .variant(TopAppBarVariant::Large)
                        .subtitle("Subtitle")
                },
                152.0,
                app_bar_large_flexible::TITLE_FONT,
                Some(app_bar_large_flexible::SUBTITLE_FONT),
                28.0,
            ),
        ];
        for (bar, height, title_font, subtitle_font, distance) in cases {
            let cx = open(app, 600.0, bar);
            let title = cx
                .debug_bounds("md3-app-bar-title")
                .expect("タイトルを描くこと");
            let expected_gap = cx.update(|window, cx| {
                let theme = cx.md3_theme().clone();
                let (title_baseline, title_line_height) =
                    super::baseline(&theme, title_font, &"Title".into(), window);
                let (last_baseline, last_line_height, block) = match subtitle_font {
                    Some(font) => {
                        let (baseline, line_height) =
                            super::baseline(&theme, font, &"Subtitle".into(), window);
                        (baseline, line_height, title_line_height + line_height)
                    }
                    None => (title_baseline, title_line_height, title_line_height),
                };
                let gap = f32::from(px(distance) - (last_line_height - last_baseline));
                gap.min(height - 64.0 - f32::from(block)).max(0.0)
            });
            assert_eq!(height - f32::from(title.bottom()), expected_gap);
        }
    }
}
