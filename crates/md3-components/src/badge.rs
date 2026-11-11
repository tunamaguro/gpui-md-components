// 寸法・色・書体・アンカーに対する位置は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Badge.kt
//         (`BadgedBox`、`Badge`、`BadgeDefaults`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 のバッジ

use crate::shape::Shaped;
use gpui::{
    AnyElement, App, Div, ElementId, InteractiveElement, IntoElement, ParentElement, Refineable,
    RenderOnce, SharedString, StatefulInteractiveElement, StyleRefinement, Styled, Window,
    accesskit, div, prelude::FluentBuilder, relative,
};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::badge;
use md3_theme::md3_tokens::supplement;
use md3_theme::{SharedTheme, ThemeExt, dp};

/// バッジの色の上書き。`None` の項目は既定の色のまま
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
    /// `id` は同じ親の中で一意にすること。[`Badge::aria_label`] で公開するノードの id になる
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

    /// バッジを、名前が `label` のテキストとして支援技術に公開する
    ///
    /// 子の内容とは別に、内容の意味が伝わる名前 (「新着 3 件」など) を与える。
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
        let (size, color) = if has_content {
            (badge::LARGE_SIZE, badge::LARGE_COLOR)
        } else {
            (badge::SIZE, badge::COLOR)
        };
        let mut container = div()
            .flex()
            .flex_none()
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
        container.style().refine(&self.style);
        let shape = if has_content {
            badge::LARGE_SHAPE
        } else {
            badge::SHAPE
        };
        let container = Shaped::new(container, theme, shape);
        match self.aria_label {
            Some(label) => div()
                .id(self.id)
                .flex()
                .flex_none()
                .role(accesskit::Role::Label)
                // gpui のテキスト (`gpui::text!`) と同じく、Label の文字は名前ではなく値に書く
                .aria_value(label)
                .child(container)
                .into_any_element(),
            None => container.into_any_element(),
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
/// 子の内容をアンカーとして置き、箱の大きさはアンカーの大きさになる。バッジは箱の外にはみ出し、レイアウトには影響しない。
/// 箱は flex の親の中に置き、交差軸に伸ばさない配置 (`items_center` など) にすること。
/// flex でない親の中や交差軸に伸ばす配置では、箱が親いっぱいに広がり、バッジもその右上に移る。
/// 内容の無いバッジはアンカーの右上の角から 6dp 内側に、内容のあるバッジは左端をアンカーの右端から 12dp 内側、
/// 下端をアンカーの上端から 14dp 下に置く。
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
            anchor: div().relative().flex().flex_none(),
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
    use gpui::{Context, Element, Render, TestAppContext, px};

    use super::*;

    struct ElementView(Box<dyn Fn() -> AnyElement>);

    impl Render for ElementView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            (self.0)()
        }
    }

    /// 矩形の論理ピクセルの x, y, 幅, 高さと塗り
    type Quad = ((f32, f32, f32, f32), gpui::Background);

    /// 24×24 のアンカーに `badge` を重ね、ウィンドウの (100, 100) に置いて描いたときの矩形と、Error の色
    fn painted_quads(
        cx: &mut TestAppContext,
        badge: fn() -> Badge,
    ) -> (Vec<Quad>, gpui::Background) {
        let (_, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            ElementView(Box::new(move || {
                div()
                    .flex()
                    .items_start()
                    .pt(px(100.))
                    .pl(px(100.))
                    .child(BadgedBox::new(badge()).child(div().size(px(24.))))
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
            (quads, cx.md3_theme().color(ColorRole::Error).into())
        })
    }

    /// 期待値は Compose の `BadgedBox` の式 (x = アンカーの幅 - 横のずれ、y = -バッジの高さ + 縦のずれ) に
    /// 24×24 のアンカーと `BadgeOffset` 6dp を手で代入して求めた
    #[gpui::test]
    fn 内容の無いバッジはアンカーの右上の角の内側に点を描く(
        cx: &mut TestAppContext,
    ) {
        let (quads, error) = painted_quads(cx, || Badge::new("badge"));
        assert_eq!(quads, vec![((118.0, 100.0, 6.0, 6.0), error)]);
    }

    /// 横のずれ 12dp、縦のずれ 14dp、高さ 16dp から、x = 24 - 12、y = -16 + 14
    #[gpui::test]
    fn 内容のあるバッジは高さ_16dp_で右上にはみ出す(cx: &mut TestAppContext) {
        let (quads, error) = painted_quads(cx, || Badge::new("badge").child("3"));
        let ((x, y, width, height), color) = quads[0];
        assert_eq!((x, y, height, color), (112.0, 98.0, 16.0, error));
        assert!(width >= 16.0, "幅は最小 16dp: {width}");
    }

    #[gpui::test]
    fn 長い内容は左右に余白を空けて横に伸びる(cx: &mut TestAppContext) {
        let (quads, _) = painted_quads(cx, || Badge::new("badge").child("999+"));
        let ((x, _, width, _), _) = quads[0];
        assert_eq!(x, 112.0, "左端の位置は内容の長さで変わらない");
        assert!(width > 16.0, "内容に合わせて伸びる: {width}");
    }

    #[gpui::test]
    fn 名前を与えるとテキストとして公開する(cx: &mut TestAppContext) {
        let (_, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            ElementView(Box::new(|| div().into_any_element()))
        });
        let node = cx.update(|window, cx| {
            let mut element = Badge::new("badge")
                .child("3")
                .aria_label("3 new messages")
                .render(window, cx)
                .into_any_element();
            let element = element
                .downcast_mut::<gpui::Stateful<Div>>()
                .expect("名前を与えたバッジは id を持つ要素になること");
            let mut node = accesskit::Node::new(element.a11y_role().expect("role を持つこと"));
            element.write_a11y_info(&mut node);
            node
        });
        assert_eq!(node.role(), accesskit::Role::Label);
        assert_eq!(node.value(), Some("3 new messages"));
    }
}
