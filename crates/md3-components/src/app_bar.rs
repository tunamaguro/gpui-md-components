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
    AnimationExt, AnyElement, App, Div, ElementId, Hsla, InteractiveElement, IntoElement,
    ParentElement, Pixels, Refineable, RenderOnce, SharedString, SpringAnimation, StyleRefinement,
    Styled, Window, div, prelude::FluentBuilder, px,
};
use md3_theme::md3_tokens::component::{
    app_bar, app_bar_large_flexible, app_bar_medium_flexible, app_bar_small,
};
use md3_theme::md3_tokens::motion::MotionSchemeKey;
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
/// Tab で到達できるナビゲーションのアイコンと操作は、この順に並ぶ
#[derive(IntoElement)]
pub struct TopAppBar {
    id: ElementId,
    title: SharedString,
    subtitle: Option<SharedString>,
    variant: TopAppBarVariant,
    alignment: TopAppBarTitleAlignment,
    navigation_icon: Option<IconButton>,
    actions: Vec<AnyElement>,
    scrolled: bool,
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
            scrolled: false,
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

    /// バーの下に内容がスクロールして入り込んでいるかを設定する
    ///
    /// `true` のあいだ、容器をスクロール中の色で塗る。値が変わると色をバネで変え、
    /// モーションを減らす設定 ([`App::reduce_motion`]) ではすぐに変える。
    /// バーの高さはスクロールで変わらない
    pub fn scrolled(mut self, scrolled: bool) -> Self {
        self.scrolled = scrolled;
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
    /// 有効な `icon` は、[`IconButton::colors`] でアイコンの色を指定すれば、どのバリアントでもその色で描く。
    /// 指定しなければ、Standard と Outlined はナビゲーションのアイコンの色で、Filled と FilledTonal はバリアントの色で描く。
    /// 無効な `icon` は [`IconButton`] の無効時の色で描く
    pub fn navigation_icon(mut self, icon: IconButton) -> Self {
        self.navigation_icon = Some(icon);
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    ///
    /// ナビゲーションのアイコンは、[`IconButton::theme`] でテーマを指定していなければこのテーマで描く。
    /// 操作として置いた子の要素には渡らないため、子の要素ごとに指定すること
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

/// `scale` の書体で `text` を描いたときの、最後の行の基準線から行の下端までの距離
///
/// gpui は行の高さの中で文字の上下の余白を等しくして基準線を置く
fn descent_below_baseline(
    theme: &Theme,
    scale: TypeScale,
    text: &SharedString,
    window: &Window,
) -> Pixels {
    let mut style = window.text_style();
    style.refine(&theme.text_style(scale));
    let font_size = style.font_size.to_pixels(window.rem_size());
    let line_height = style.line_height_in_pixels(window.rem_size());
    // shape_line は改行を含む文字列を受け付けないため、最後の行だけを測る
    let last: SharedString = text.lines().last().unwrap_or_default().to_owned().into();
    let line =
        window
            .text_system()
            .shape_line(last.clone(), font_size, &[style.to_run(last.len())], None);
    line_height - ((line_height - line.ascent - line.descent) / 2.0 + line.ascent)
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
        let small = self.variant == TopAppBarVariant::Small;
        // Small は 1 行に収める。truncate は幅による折り返しだけを止めるため、改行も空白にする
        let one_line = |content: SharedString| -> SharedString {
            if small && content.contains('\n') {
                content.replace('\n', " ").into()
            } else {
                content
            }
        };
        let text = |font: TypeScale, color: Hsla, content: gpui::Text| {
            let mut element = div().text_color(color);
            element.text_style().refine(&theme.text_style(font));
            element
                .when(small, |this| this.truncate())
                .when(centered, |this| this.text_center())
                .child(content)
        };
        let title = div()
            .debug_selector(|| "md3-app-bar-title".into())
            .flex()
            .flex_col()
            .min_w_0()
            .px(edge)
            .when(centered, |this| this.items_center())
            // 文字列の ID は呼び出し位置から作られ、利用者が子の要素に付ける ID とぶつからない
            .child(text(
                title_font,
                theme.color(app_bar::TITLE_COLOR),
                gpui::text!(one_line(self.title.clone())),
            ))
            .when_some(self.subtitle.clone(), |this, subtitle| {
                this.child(text(
                    subtitle_font,
                    theme.color(app_bar::SUBTITLE_COLOR),
                    gpui::text!(one_line(subtitle)),
                ))
            });

        // Compose のアイコンボタンと同じ間隔で並べるため、アイコンボタンを 48dp 四方の配置枠の中央に置く。
        // 押せる範囲はアイコンボタン自身の大きさのまま
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
                            .default_theme(theme.clone())
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
        let bar = div().w_full().flex().flex_col();
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
                // 行はタイトルが収まる高さまで伸びる。タイトルの下の余白は行の高さに収まらなければ縮める
                let (last_font, last_text) = match &self.subtitle {
                    Some(subtitle) => (subtitle_font, subtitle),
                    None => (title_font, &self.title),
                };
                let gap = (dp(bottom_padding)
                    - descent_below_baseline(&theme, last_font, last_text, window))
                .max(px(0.0));
                bar.child(
                    row(leading(self.navigation_icon), None, trailing(self.actions))
                        .h(collapsed_height),
                )
                .child(
                    // 余白は高さ 0 から伸びるため、行の高さはタイトルの高さと行の最小の高さの大きいほうになる。
                    // 余った高さは余白が `gap` まで受け取り、残りはタイトルの上に空ける
                    div()
                        .flex()
                        .flex_col()
                        .justify_end()
                        .min_h(dp(expanded_height) - collapsed_height)
                        .child(row(leading(None), Some(title), trailing(Vec::new())))
                        .child(div().flex_grow_1().flex_basis(px(0.0)).max_h(gap)),
                )
            }
        };
        // Compose は内容が入り込むと色の目標を切り替え、DefaultEffects のバネで変える
        let container = theme.color(app_bar::CONTAINER_COLOR);
        let scrolled_container = theme.color(app_bar::ON_SCROLL_CONTAINER_COLOR);
        let style = self.style;
        // バネの状態は id ごとに保たれるため、バーの id をそのまま使い、子の id もその下に置く
        bar.with_spring(
            self.id,
            SpringAnimation::new(theme.spring(MotionSchemeKey::DefaultEffects)).to(self.scrolled),
            move |mut this, phase| {
                this =
                    this.bg(container.blend(scrolled_container.opacity(phase.0.clamp(0.0, 1.0))));
                this.style().refine(&style);
                this
            },
        )
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
    use gpui::{Context, Entity, Render, TestAppContext, VisualTestContext};
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
        // アイコンボタンの容器は 48dp の配置枠の中央にある
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

    /// 期待値は Compose の `placeTopAppBar`。下の行はタイトルが収まる高さまで伸び、最後の行の基準線を
    /// 下端から 24dp (Medium) か 28dp (Large) に置く。余白が行に収まらなければ収まるまで縮める。
    /// 既定の高さでは、どの書体でも余白は縮む
    // gpui は debug_assertions のあるビルドでだけ debug_selector の位置を記録する
    #[cfg(debug_assertions)]
    #[gpui::test]
    fn medium_と_large_の下の行はタイトルの下に収まるだけの余白を取る(
        app: &mut TestAppContext,
    ) {
        /// バー、バーの幅、バーの高さ、タイトルの下端
        type Case = (fn() -> TopAppBar, f32, f32, f32);
        let cases: [Case; 4] = [
            // 下の行 48dp に HeadlineMedium の 1 行 36dp を置き、残りの 12dp を余白にする
            (
                || TopAppBar::new("bar", "Title").variant(TopAppBarVariant::Medium),
                600.0,
                112.0,
                100.0,
            ),
            // 下の行 56dp に DisplaySmall の 1 行 44dp を置き、残りの 12dp を余白にする
            (
                || TopAppBar::new("bar", "Title").variant(TopAppBarVariant::Large),
                600.0,
                120.0,
                108.0,
            ),
            // 折り返して 2 行 72dp になると、行は 72dp に伸び、余白は無くなる
            (
                || {
                    TopAppBar::new("bar", "A medium title that wraps")
                        .variant(TopAppBarVariant::Medium)
                },
                300.0,
                136.0,
                136.0,
            ),
            // 改行を含む文も同じく 2 行になる
            (
                || TopAppBar::new("bar", "First\nSecond").variant(TopAppBarVariant::Medium),
                600.0,
                136.0,
                136.0,
            ),
        ];
        for (bar, width, height, bottom) in cases {
            let cx = open(app, width, bar);
            assert_eq!(
                filled(cx, ColorRole::Surface),
                vec![(0.0, 0.0, width, height)]
            );
            let title = cx
                .debug_bounds("md3-app-bar-title")
                .expect("タイトルを描くこと");
            assert_eq!(f32::from(title.bottom()), bottom);
        }
    }

    #[gpui::test]
    fn 補足の文に改行があっても描ける(app: &mut TestAppContext) {
        let cx = open(app, 600.0, || {
            TopAppBar::new("bar", "Title")
                .variant(TopAppBarVariant::Large)
                .subtitle("First\nSecond")
        });
        // 下の行 88dp に DisplaySmall の 44dp と TitleMedium の 2 行 48dp を置くと収まらず、行が 92dp に伸びる
        assert_eq!(
            filled(cx, ColorRole::Surface),
            vec![(0.0, 0.0, 600.0, 64.0 + 92.0)]
        );
    }

    #[gpui::test]
    fn small_は改行を含む文も_1_行に収める(app: &mut TestAppContext) {
        let cx = open(app, 600.0, || {
            TopAppBar::new("bar", "First\nSecond").subtitle("Sub\ntitle")
        });
        assert_eq!(
            filled(cx, ColorRole::Surface),
            vec![(0.0, 0.0, 600.0, 64.0)]
        );
    }

    /// `scrolled` を切り替えられるバーと、その下に置いた切り替えないバー
    struct ScrolledView {
        scrolled: bool,
    }

    impl Render for ScrolledView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .w(px(600.0))
                .child(TopAppBar::new("bar", "Title").scrolled(self.scrolled))
                .child(TopAppBar::new("other", "Other"))
        }
    }

    fn open_scrolled(cx: &mut TestAppContext) -> (Entity<ScrolledView>, &mut VisualTestContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            ScrolledView { scrolled: false }
        });
        cx.run_until_parked();
        (view, cx)
    }

    // gpui のバネは実時間 (`Instant::now`) で進み、テストの時計では進められないため、
    // 途中の色は確かめず、十分な時間の後に着くことだけを確かめる
    #[gpui::test]
    fn 内容が入り込むとスクロール中の色まで変わり_戻ると元の色に戻る(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = open_scrolled(cx);
        let bar = (0.0, 0.0, 600.0, 64.0);
        let other = (0.0, 64.0, 600.0, 64.0);
        assert_eq!(filled(cx, ColorRole::Surface), vec![bar, other]);
        // 切り替えないバーは元の色のまま
        for (scrolled, scrolled_rects, surface_rects) in [
            (true, vec![bar], vec![other]),
            (false, vec![], vec![bar, other]),
        ] {
            view.update(cx, |view, cx| {
                view.scrolled = scrolled;
                cx.notify();
            });
            cx.run_until_parked();
            // DefaultEffects のバネは 1 秒以内に収まる
            std::thread::sleep(std::time::Duration::from_secs(1));
            view.update(cx, |_, cx| cx.notify());
            cx.run_until_parked();
            assert_eq!(
                filled(cx, ColorRole::SurfaceContainer),
                scrolled_rects,
                "scrolled={scrolled}"
            );
            assert_eq!(
                filled(cx, ColorRole::Surface),
                surface_rects,
                "scrolled={scrolled}"
            );
        }
    }

    #[gpui::test]
    fn モーションを減らす設定ではスクロール中の色をすぐに描く(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = open_scrolled(cx);
        cx.update(|_, cx| cx.set_reduce_motion(true));
        view.update(cx, |view, cx| {
            view.scrolled = true;
            cx.notify();
        });
        cx.run_until_parked();
        assert_eq!(
            filled(cx, ColorRole::SurfaceContainer),
            vec![(0.0, 0.0, 600.0, 64.0)]
        );
        assert_eq!(
            filled(cx, ColorRole::Surface),
            vec![(0.0, 64.0, 600.0, 64.0)]
        );
    }
}
