// 寸法・色・書体・アンカーに対する位置は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Badge.kt
//         (`BadgedBox`、`Badge`、`BadgeDefaults`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 のバッジ

use crate::shape::Shaped;
use gpui::{
    AnyElement, App, Div, ElementId, IntoElement, ParentElement, Refineable, RenderOnce,
    SharedString, StyleRefinement, Styled, Window, div, prelude::FluentBuilder, relative,
};
use gpui_headless::text::StaticText;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::badge;
use md3_theme::md3_tokens::supplement;
use md3_theme::{SharedTheme, ThemeExt, dp};

/// バッジの色の上書き。`None` の項目は既定の色のまま
///
/// 内容の色は容器の色に追従しないため、容器の色を変えるときは、読める組み合わせになる内容の色
/// (容器が [`ColorRole::Primary`] なら [`ColorRole::OnPrimary`] など) も指定すること
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BadgeColors {
    /// 容器の色。既定は [`ColorRole::Error`]
    pub container: Option<ColorRole>,
    /// 内容の色。既定は [`ColorRole::OnError`]
    pub content: Option<ColorRole>,
}

/// MD3 のバッジ
///
/// 通知の有無や件数のような、アンカーに付く状態を示す。
/// 子の内容が無ければ直径 6dp の点を、あれば高さ 16dp の容器に内容を描く。内容は短い数字や文字にする。
/// アンカーに重ねるときは [`BadgedBox`] に渡す
///
/// [`Styled`] で指定したスタイルはバッジの外側の箱に付く。位置や外側の余白、flex の指定に使い、
/// 箱を広げるとバッジも箱いっぱいに広がる。色や形は変えられないため、[`Badge::colors`] を使う。
/// 単独で置くときは、flex の親の中で交差軸に伸ばさない配置 (`items_center` など) にすること。
/// flex でない親の中では親の幅いっぱいに、交差軸に伸ばす配置では交差軸の方向に広がる
///
/// 既定では支援技術に公開しない。バッジが伝える情報は、アンカーの名前に含めるか、
/// [`Badge::aria_label`] で与えること
#[derive(IntoElement)]
pub struct Badge {
    id: ElementId,
    children: Vec<AnyElement>,
    style: StyleRefinement,
    colors: BadgeColors,
    aria_label: Option<SharedString>,
    theme: Option<SharedTheme>,
}

impl Badge {
    /// 内容の無いバッジを作る。子を足すと内容を持つバッジになる
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。[`Badge::aria_label`] で公開するノードの id になり、
    /// 変わると支援技術にはノードの削除と追加として伝わる
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            children: Vec::new(),
            style: StyleRefinement::default(),
            colors: BadgeColors::default(),
            aria_label: None,
            theme: None,
        }
    }

    /// 既定の色を `colors` で上書きする
    pub fn colors(mut self, colors: BadgeColors) -> Self {
        self.colors = colors;
        self
    }

    /// バッジを、`label` の文字列のテキストとして支援技術に公開する
    ///
    /// 子の内容とは別に、内容の意味が伝わる文字列 (「新着 3 件」など) を与える。
    /// 子の内容に `gpui::text!` のような id を持つテキストを置くと、それも別のノードとして公開されるため、
    /// このときは `&str` などのテキストをそのまま置くこと
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    fn has_content(&self) -> bool {
        !self.children.is_empty()
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let has_content = self.has_content();
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let (size, color, shape) = if has_content {
            (badge::LARGE_SIZE, badge::LARGE_COLOR, badge::LARGE_SHAPE)
        } else {
            (badge::SIZE, badge::COLOR, badge::SHAPE)
        };
        let mut container = div()
            .flex()
            .flex_grow_1()
            .items_center()
            .justify_center()
            .min_w(dp(size))
            .min_h(dp(size))
            .bg(theme.color(self.colors.container.unwrap_or(color)))
            .when(has_content, |this| {
                this.px(dp(supplement::badge::WITH_CONTENT_HORIZONTAL_PADDING))
                    .text_color(
                        theme.color(self.colors.content.unwrap_or(badge::LARGE_LABEL_TEXT_COLOR)),
                    )
                    .whitespace_nowrap()
            })
            .children(self.children);
        if has_content {
            container
                .text_style()
                .refine(&theme.text_style(badge::LARGE_LABEL_TEXT_FONT));
        }
        let container = Shaped::new(container, theme, shape);
        // 名前の有無で外側の箱の型が変わっても、Styled の効き方を変えないよう、どちらも同じ既定に重ねる
        match self.aria_label {
            Some(label) => {
                let mut outer = StaticText::new(self.id, label).flex().flex_none();
                outer.style().refine(&self.style);
                outer.child(container).into_any_element()
            }
            None => {
                let mut outer = div().flex().flex_none();
                outer.style().refine(&self.style);
                outer.child(container).into_any_element()
            }
        }
    }
}

impl Styled for Badge {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for Badge {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

/// [`Badge`] をアンカーの右上に重ねる箱
///
/// 子の内容をアンカーとして中央に置く。アンカーには要素を 1 つ渡すこと (複数渡すと横に並ぶ)。
/// 大きさを [`Styled`] で指定しなければ、箱の大きさはアンカーの大きさになる。バッジはレイアウトに影響しない。
/// 箱は flex の親の中に置き、交差軸に伸ばさない配置 (`items_center` など) にすること。
/// flex でない親の中や交差軸に伸ばす配置では、箱が親いっぱいに広がり、バッジもその右上に移る。
/// 内容の無いバッジ (点) は、右端と上端を箱の右上の角に合わせて箱の内側に置く。
/// 内容のあるバッジは、左端を箱の右端から 12dp 内側、下端を箱の上端から 14dp 下に置き、箱の右上の外にはみ出す。
/// [`Styled`] で指定したスタイルは箱に付く
#[derive(IntoElement)]
pub struct BadgedBox {
    anchor: Div,
    badge: Badge,
}

impl BadgedBox {
    /// `badge` を重ねる箱を作る
    pub fn new(badge: Badge) -> Self {
        Self {
            // Compose はアンカーを `Box(contentAlignment = Alignment.Center)` で置く
            anchor: div()
                .relative()
                .flex()
                .flex_none()
                .items_center()
                .justify_center(),
            badge,
        }
    }
}

impl RenderOnce for BadgedBox {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let (horizontal, vertical) = if self.badge.has_content() {
            (
                supplement::badge::WITH_CONTENT_HORIZONTAL_OFFSET,
                supplement::badge::WITH_CONTENT_VERTICAL_OFFSET,
            )
        } else {
            (supplement::badge::OFFSET, supplement::badge::OFFSET)
        };
        // 左端をアンカーの右端に、下端をアンカーの上端に合わせてから、外側の余白を負にして内側へずらす
        self.anchor.child(
            div()
                .absolute()
                .left(relative(1.0))
                .bottom(relative(1.0))
                .ml(-dp(horizontal))
                .mb(-dp(vertical))
                .child(self.badge),
        )
    }
}

impl Styled for BadgedBox {
    fn style(&mut self) -> &mut StyleRefinement {
        self.anchor.style()
    }
}

impl ParentElement for BadgedBox {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.anchor.extend(elements);
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Context, Element, Render, TestAppContext, VisualTestContext, px};
    use md3_theme::md3_tokens::typography::TypeScale;

    use super::*;

    struct ElementView(Box<dyn Fn(&App) -> AnyElement>);

    impl Render for ElementView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            (self.0)(cx)
        }
    }

    /// 矩形の論理ピクセルの x, y, 幅, 高さと、左上の角丸の半径と塗り
    type Quad = ((f32, f32, f32, f32), f32, gpui::Background);

    fn add_view(
        cx: &mut TestAppContext,
        element: impl Fn(&App) -> AnyElement + 'static,
    ) -> &mut VisualTestContext {
        let (_, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            ElementView(Box::new(element))
        });
        cx
    }

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

    fn error_color(cx: &mut VisualTestContext) -> gpui::Background {
        cx.update(|_, cx| cx.md3_theme().color(ColorRole::Error).into())
    }

    /// 24×24 のアンカーに `badge` を重ね、ウィンドウの (100, 100) に置く
    fn add_badged_box(cx: &mut TestAppContext, badge: fn() -> Badge) -> &mut VisualTestContext {
        add_view(cx, move |_| {
            div()
                .flex()
                .items_start()
                .pt(px(100.))
                .pl(px(100.))
                .child(BadgedBox::new(badge()).child(div().size(px(24.))))
                .into_any_element()
        })
    }

    /// 期待値は Compose の `BadgedBox` の式 (x = アンカーの幅 - 横のずれ、y = -バッジの高さ + 縦のずれ) に
    /// 24×24 のアンカーと `BadgeOffset` 6dp を手で代入して求めた。半径は CornerFull の短辺の半分
    #[gpui::test]
    fn 内容の無いバッジはアンカーの右上の角の内側に点を描く(
        cx: &mut TestAppContext,
    ) {
        let cx = add_badged_box(cx, || Badge::new("badge"));
        let error = error_color(cx);
        assert_eq!(
            painted_quads(cx),
            vec![((118.0, 100.0, 6.0, 6.0), 3.0, error)]
        );
    }

    /// 横のずれ 12dp、縦のずれ 14dp、高さ 16dp から、x = 24 - 12、y = -16 + 14
    #[gpui::test]
    fn 内容のあるバッジは高さ_16dp_で右上にはみ出す(cx: &mut TestAppContext) {
        let cx = add_badged_box(cx, || Badge::new("badge").child("3"));
        let error = error_color(cx);
        let ((x, y, width, height), radius, color) = painted_quads(cx)[0];
        assert_eq!(
            (x, y, height, radius, color),
            (112.0, 98.0, 16.0, 8.0, error)
        );
        assert!(width >= 16.0, "幅は最小 16dp: {width}");
    }

    /// 期待値は 48×48 の箱の中央に 24×24 のアンカーを置いた位置 (Compose の `Alignment.Center`) と、
    /// 箱の幅 48 から `BadgeOffset` 6dp を引いた点の位置
    #[gpui::test]
    fn 箱を広げるとアンカーを中央に置き_バッジは箱の右上に付く(
        cx: &mut TestAppContext,
    ) {
        let cx = add_view(cx, |_| {
            div()
                .flex()
                .items_start()
                .child(
                    BadgedBox::new(Badge::new("badge"))
                        .size(px(48.))
                        .child(div().size(px(24.)).bg(gpui::black())),
                )
                .into_any_element()
        });
        let rects: Vec<_> = painted_quads(cx)
            .into_iter()
            .map(|(rect, _, _)| rect)
            .collect();
        assert_eq!(rects, vec![(12.0, 12.0, 24.0, 24.0), (42.0, 0.0, 6.0, 6.0)]);
    }

    /// 期待値の左右の余白 4dp は Compose の `BadgeWithContentHorizontalPadding`
    #[gpui::test]
    fn 長い内容は左右に余白を空けて横に伸びる(cx: &mut TestAppContext) {
        let cx = add_view(cx, |cx| {
            let mut probe = div().flex().flex_none().bg(gpui::black()).child("999+");
            probe
                .text_style()
                .refine(&cx.md3_theme().text_style(TypeScale::LabelSmall));
            div()
                .flex()
                .items_start()
                .gap(px(100.))
                .child(Badge::new("badge").child("999+"))
                .child(probe)
                .into_any_element()
        });
        let quads = painted_quads(cx);
        let ((x, _, badge_width, _), _, _) = quads[0];
        let ((_, _, text_width, _), _, _) = quads[1];
        assert_eq!(x, 0.0);
        assert!(
            text_width > 16.0,
            "内容が最小の幅より広いこと: {text_width}"
        );
        assert_eq!(badge_width, text_width + 8.0);
    }

    #[gpui::test]
    fn 名前の有無によらず_styled_のスタイルは外側の箱に付く(
        cx: &mut TestAppContext,
    ) {
        for labelled in [false, true] {
            let cx = add_view(cx, move |_| {
                let badge = Badge::new("badge").child("3").absolute().top_0().right_0();
                let badge = if labelled {
                    badge.aria_label("3 new messages")
                } else {
                    badge
                };
                div()
                    .flex()
                    .items_start()
                    .pl(px(100.))
                    .child(div().relative().flex().size(px(100.)).child(badge))
                    .into_any_element()
            });
            let ((x, y, width, _), _, _) = painted_quads(cx)[0];
            assert_eq!((x + width, y), (200.0, 0.0), "labelled={labelled}");
        }
    }

    // 名前の a11y ノードの属性は gpui-headless の StaticText のテストで確かめる。
    // StaticText の描画は外から取り出せないため、ここでは名前を与えると StaticText で包むことだけを確かめる
    #[gpui::test]
    fn 名前を与えると_static_text_で公開する(cx: &mut TestAppContext) {
        let cx = add_view(cx, |_| div().into_any_element());
        let uses_static_text = cx.update(|window, cx| {
            let mut element = Badge::new("badge")
                .child("3")
                .aria_label("3 new messages")
                .render(window, cx)
                .into_any_element();
            element
                .downcast_mut::<gpui::ViewElement<StaticText>>()
                .is_some()
        });
        assert!(uses_static_text);
    }

    #[gpui::test]
    fn 名前を与えなければ支援技術に公開しない(cx: &mut TestAppContext) {
        let cx = add_view(cx, |_| div().into_any_element());
        let role = cx.update(|window, cx| {
            let mut element = Badge::new("badge")
                .child("3")
                .render(window, cx)
                .into_any_element();
            element
                .downcast_mut::<Div>()
                .expect("名前を与えないバッジは id の無い要素になること")
                .a11y_role()
        });
        assert_eq!(role, None);
    }
}
