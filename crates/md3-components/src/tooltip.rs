// 寸法と色は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Tooltip.kt
//         (`PlainTooltip`、`RichTooltip`、`TooltipDefaults`、`textVerticalPadding`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2023 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 のツールチップ

use gpui::{
    FocusHandle, ParentElement, Pixels, Refineable, SharedString, StatefulInteractiveElement,
    Styled, Window, div, px,
};
use gpui_headless::tooltip::{Tooltip as HeadlessTooltip, TooltipRelation};

use crate::shape::Shaped;
use md3_theme::md3_tokens::component::{plain_tooltip, rich_tooltip};
use md3_theme::md3_tokens::supplement;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{SharedTheme, dp};

/// ツールチップの種類
#[derive(Clone, Debug, Default, PartialEq)]
pub enum TooltipVariant {
    /// 短い文で、トリガーが何かを示す。アイコンだけのボタンの名前を見せるのに使う
    #[default]
    Plain,
    /// トリガーについての補足を、見出しを付けて長めの文で示す
    Rich {
        /// 本文の上に置く見出し
        title: Option<SharedString>,
    },
}

/// MD3 のツールチップ
///
/// [`IconButton::tooltip`](crate::IconButton::tooltip) などに渡し、トリガーにマウスが乗っている間と、
/// キーボードでトリガーにフォーカスを移した間に、トリガーの上に表示する。
/// 表示中のツールチップにマウスを動かしても表示を続け、Escape で閉じる。
/// トリガーのテーマで描き、テーマの密度 ([`ThemeSettings::density`](md3_theme::ThemeSettings::density)) は反映しない
///
/// 支援技術には、文字列をトリガーの説明として公開する。見出しを持つ場合は、見出しと本文を改行でつないだ文字列にする。
/// 文字列がトリガーのアクセシブルな名前と同じ場合は、重ねて読まれないよう説明にしない。
/// ツールチップはフォーカスを受け取らないため、押せる要素 (Rich のアクション) は持てない
#[derive(Clone, Debug, PartialEq)]
pub struct Tooltip {
    text: SharedString,
    variant: TooltipVariant,
}

impl Tooltip {
    /// `text` を示す [`TooltipVariant::Plain`] のツールチップを作る
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            variant: TooltipVariant::default(),
        }
    }

    /// ツールチップの種類を設定する
    pub fn variant(mut self, variant: TooltipVariant) -> Self {
        self.variant = variant;
        self
    }

    /// `trigger` にこのツールチップを付けて返す
    ///
    /// `focus_handle` は `trigger` のフォーカス、`label` は `trigger` のアクセシブルな名前にすること
    pub(crate) fn attach<E: StatefulInteractiveElement + ParentElement>(
        self,
        trigger: E,
        focus_handle: &FocusHandle,
        label: &SharedString,
        theme: &SharedTheme,
        window: &Window,
    ) -> E {
        let text_style = |scale: TypeScale| {
            let mut element = div();
            element.text_style().refine(&theme.text_style(scale));
            element
        };
        // Compose の `paddingFromBaseline` と同じく、上端から最初の基準線までを `distance` にする上の余白。
        // gpui は行の高さの中で文字の上下の余白を等しくして基準線を置く
        let padding_from_baseline = |scale: TypeScale, text: &SharedString, distance: Pixels| {
            let mut style = window.text_style();
            style.refine(&theme.text_style(scale));
            let font_size = style.font_size.to_pixels(window.rem_size());
            let line_height = style.line_height_in_pixels(window.rem_size());
            let line = window.text_system().shape_line(
                text.clone(),
                font_size,
                &[style.to_run(text.len())],
                None,
            );
            let baseline = (line_height - line.ascent - line.descent) / 2.0 + line.ascent;
            (distance - baseline).max(px(0.))
        };

        let (description, content) = match self.variant {
            TooltipVariant::Plain => {
                let container = text_style(plain_tooltip::SUPPORTING_TEXT_FONT)
                    .min_w(dp(supplement::tooltip::MIN_WIDTH))
                    .max_w(dp(supplement::tooltip::PLAIN_MAX_WIDTH))
                    .min_h(dp(supplement::tooltip::MIN_HEIGHT))
                    .px(dp(supplement::tooltip::PLAIN_HORIZONTAL_PADDING))
                    .py(dp(supplement::tooltip::PLAIN_VERTICAL_PADDING))
                    .bg(theme.color(plain_tooltip::CONTAINER_COLOR))
                    .text_color(theme.color(plain_tooltip::SUPPORTING_TEXT_COLOR))
                    .child(self.text.clone());
                (
                    self.text,
                    Shaped::new(container, theme.clone(), plain_tooltip::CONTAINER_SHAPE),
                )
            }
            TooltipVariant::Rich { title } => {
                // `textVerticalPadding`。見出しもアクションも無ければ Plain と同じ上下の余白を取る
                let body = text_style(rich_tooltip::SUPPORTING_TEXT_FONT)
                    .text_color(theme.color(rich_tooltip::SUPPORTING_TEXT_COLOR));
                let body = if title.is_some() {
                    body.pt(padding_from_baseline(
                        rich_tooltip::SUPPORTING_TEXT_FONT,
                        &self.text,
                        dp(supplement::tooltip::HEIGHT_FROM_SUBHEAD_TO_TEXT_FIRST_LINE),
                    ))
                    .pb(dp(supplement::tooltip::TEXT_BOTTOM_PADDING))
                } else {
                    body.py(dp(supplement::tooltip::PLAIN_VERTICAL_PADDING))
                }
                .child(self.text.clone());
                let heading = title.as_ref().map(|title| {
                    text_style(rich_tooltip::SUBHEAD_FONT)
                        .pt(padding_from_baseline(
                            rich_tooltip::SUBHEAD_FONT,
                            title,
                            dp(supplement::tooltip::HEIGHT_TO_SUBHEAD_FIRST_LINE),
                        ))
                        .text_color(theme.color(rich_tooltip::SUBHEAD_COLOR))
                        .child(title.clone())
                });
                let container = div()
                    .flex()
                    .flex_col()
                    .min_w(dp(supplement::tooltip::MIN_WIDTH))
                    .max_w(dp(supplement::tooltip::RICH_MAX_WIDTH))
                    .min_h(dp(supplement::tooltip::MIN_HEIGHT))
                    .px(dp(supplement::tooltip::RICH_HORIZONTAL_PADDING))
                    .bg(theme.color(rich_tooltip::CONTAINER_COLOR))
                    .shadow(theme.shadow(rich_tooltip::CONTAINER_ELEVATION))
                    .children(heading)
                    .child(body);
                let description = match title {
                    Some(title) => format!("{title}\n{}", self.text).into(),
                    None => self.text,
                };
                (
                    description,
                    Shaped::new(container, theme.clone(), rich_tooltip::CONTAINER_SHAPE),
                )
            }
        };
        let relation = if description == *label {
            TooltipRelation::Label
        } else {
            TooltipRelation::Description
        };
        HeadlessTooltip::new(focus_handle, description, content)
            .relation(relation)
            .spacing(dp(supplement::tooltip::SPACING_BETWEEN_TOOLTIP_AND_ANCHOR))
            .attach(trigger)
    }
}

#[cfg(test)]
mod tests {
    use gpui::{
        Context, Element, Hsla, InteractiveElement, IntoElement, Modifiers, Render, TestAppContext,
        VisualTestContext, accesskit, point, size,
    };
    use md3_theme::ThemeExt;
    use md3_theme::md3_color::ColorRole;

    use super::*;
    use crate::{IconButton, IconToggleButton};

    /// 左上を (100, 100) に置いた Small のアイコンボタン (40dp 四方) と、比べる用にツールチップの文字列を
    /// Plain の書体で描いた黒い `probe`。`toggle` なら toggle のアイコンボタンを置く
    struct TooltipView {
        tooltip: Tooltip,
        probe: &'static str,
        toggle: bool,
    }

    impl Render for TooltipView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let mut probe = div().flex_none().bg(gpui::black()).child(self.probe);
            probe.text_style().refine(
                &cx.md3_theme()
                    .text_style(plain_tooltip::SUPPORTING_TEXT_FONT),
            );
            div()
                .flex()
                .flex_col()
                .items_start()
                .pl(px(100.))
                .pt(px(100.))
                .gap(px(100.))
                .child(if self.toggle {
                    IconToggleButton::new("button", md3_icons::FAVORITE, "Favorite", false)
                        .tooltip(self.tooltip.clone())
                        .into_any_element()
                } else {
                    IconButton::new("button", md3_icons::FAVORITE, "Favorite")
                        .tooltip(self.tooltip.clone())
                        .into_any_element()
                })
                .child(probe)
        }
    }

    fn add_view<'a>(
        cx: &'a mut TestAppContext,
        tooltip: Tooltip,
        probe: &'static str,
        toggle: bool,
    ) -> &'a mut VisualTestContext {
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            TooltipView {
                tooltip,
                probe,
                toggle,
            }
        });
        cx
    }

    /// `background` で塗った矩形の x, y, 幅, 高さと、左上の角丸の半径
    fn filled(cx: &mut VisualTestContext, background: Hsla) -> Vec<((f32, f32, f32, f32), f32)> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            window
                .painted_quads()
                .iter()
                .filter(|quad| quad.background == background.into())
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
                    )
                })
                .collect()
        })
    }

    fn color(cx: &mut VisualTestContext, role: ColorRole) -> Hsla {
        cx.update(|_, cx| cx.md3_theme().color(role))
    }

    fn hover_button(cx: &mut VisualTestContext) {
        cx.simulate_mouse_move(point(px(120.), px(120.)), None, Modifiers::none());
    }

    /// 期待値は Compose の `PlainTooltip` の左右 8dp・上下 4dp の余白と BodySmall の行の高さ 16sp、
    /// PlainTooltipTokens の InverseSurface・CornerExtraSmall (4dp)、
    /// `TooltipDefaults.rememberTooltipPositionProvider` の上に置く位置と `SpacingBetweenTooltipAndAnchor` の 4dp。
    /// ボタンの左右中央は 120、上端は 100
    #[gpui::test]
    fn plain_はボタンの上に余白を取って描く(cx: &mut TestAppContext) {
        for toggle in [false, true] {
            let cx = add_view(
                cx,
                Tooltip::new("Add to favorites"),
                "Add to favorites",
                toggle,
            );
            let inverse_surface = color(cx, ColorRole::InverseSurface);
            assert_eq!(filled(cx, inverse_surface), vec![], "toggle: {toggle}");
            hover_button(cx);
            let probe = filled(cx, gpui::black())[0].0.2;
            let width = probe + 16.0;
            assert_eq!(
                filled(cx, inverse_surface),
                vec![((120.0 - width / 2.0, 72.0, width, 24.0), 4.0)],
                "toggle: {toggle}"
            );
        }
    }

    /// 期待値は Compose の `TooltipMinWidth` (40dp) と `TooltipDefaults.plainTooltipMaxWidth` (200dp)
    #[gpui::test]
    fn plain_の幅は最小と最大の間に収める(cx: &mut TestAppContext) {
        for (text, width) in [
            ("A", 40.0),
            (
                "A tooltip text long enough to wrap onto more than one line",
                200.0,
            ),
        ] {
            let cx = add_view(cx, Tooltip::new(text), "A", false);
            hover_button(cx);
            let inverse_surface = color(cx, ColorRole::InverseSurface);
            let rects = filled(cx, inverse_surface);
            assert_eq!(rects.len(), 1, "{text}");
            assert_eq!(rects[0].0.2, width, "{text}");
        }
    }

    fn rich_tooltip() -> Tooltip {
        Tooltip::new(
            "Rich tooltips bring attention to a particular element or feature that warrants \
             the user's focus, and can be longer than plain tooltips",
        )
        .variant(TooltipVariant::Rich {
            title: Some("Rich tooltip".into()),
        })
    }

    /// 期待値は Compose の `TooltipDefaults.richTooltipMaxWidth` (320dp) と、
    /// RichTooltipTokens の SurfaceContainer・CornerMedium (12dp)。
    /// ボタンの左右中央 120 にそろえると左端が -40 になるため、ウィンドウの左端の 0 にずらす
    #[gpui::test]
    fn rich_は最大の幅で折り返し容器を角の大きい形で描く(
        cx: &mut TestAppContext,
    ) {
        let cx = add_view(cx, rich_tooltip(), "A", false);
        hover_button(cx);
        let container = color(cx, ColorRole::SurfaceContainer);
        let rects = filled(cx, container);
        assert_eq!(rects.len(), 1);
        let ((x, _, width, _), radius) = rects[0];
        assert_eq!((x, width, radius), (0.0, 320.0, 12.0));
    }

    #[gpui::test]
    fn ウィンドウが最大の幅より狭ければウィンドウの幅で折り返す(
        cx: &mut TestAppContext,
    ) {
        let cx = add_view(cx, rich_tooltip(), "A", false);
        cx.simulate_resize(size(px(240.), px(600.)));
        hover_button(cx);
        let container = color(cx, ColorRole::SurfaceContainer);
        let rects = filled(cx, container);
        assert_eq!(rects.len(), 1);
        let ((x, _, width, _), _) = rects[0];
        assert_eq!((x, width), (0.0, 240.0));
    }

    /// 文字列がボタンの名前と同じなら名前のままにし、違えば説明として公開する
    #[gpui::test]
    fn ボタンの名前と同じ文字列は説明にしない(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            TooltipView {
                tooltip: Tooltip::new(""),
                probe: "",
                toggle: false,
            }
        });
        for (text, description) in [
            ("Favorite", None),
            ("Add to favorites", Some("Add to favorites")),
        ] {
            let node = cx.update(|window, cx| {
                let focus_handle = cx.focus_handle();
                let theme = cx.md3_theme().clone();
                // Element にも id があるため、InteractiveElement の id を指定して呼ぶ
                let trigger = Tooltip::new(text).attach(
                    InteractiveElement::id(div(), "trigger")
                        .role(accesskit::Role::Button)
                        .aria_label("Favorite"),
                    &focus_handle,
                    &"Favorite".into(),
                    &theme,
                    window,
                );
                let mut node = accesskit::Node::new(trigger.a11y_role().expect("role を持つこと"));
                trigger.write_a11y_info(&mut node);
                node
            });
            assert_eq!(node.label(), Some("Favorite"), "{text}");
            assert_eq!(node.description(), description, "{text}");
        }
    }
}
