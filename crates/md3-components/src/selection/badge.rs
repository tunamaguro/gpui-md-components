// Ported from Jetpack Compose (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Badge.kt @ b8256a52
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
//! Badge。

use gpui::{App, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, px, relative};
use md3_theme::Md3ThemeExt;
use md3_theme::md3_tokens::component::badge as tokens;
use md3_theme::md3_tokens::layout;

use crate::StyledExt;

/// 件数や新着を示す小さな印。アイコンなどに重ねて置く。
///
/// 読み上げられないので、件数は重ねる先の要素のラベルなどに含めること。
/// `NavigationItem::badge` は項目の説明に文字を渡す。
#[derive(Clone, Debug, PartialEq, Eq, IntoElement)]
pub struct Badge {
    label: Option<SharedString>,
}

impl Badge {
    pub(crate) fn label(&self) -> Option<&SharedString> {
        self.label.as_ref()
    }

    /// 文字のない小さな点。
    pub fn dot() -> Self {
        Self { label: None }
    }

    /// 件数などの文字を持つバッジ。
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: Some(label.into()),
        }
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.md3();
        match self.label {
            None => div()
                .size(px(tokens::SIZE))
                .corner_radius(theme.corner(tokens::SHAPE))
                .bg(theme.color(tokens::COLOR)),
            Some(label) => div()
                .min_h(px(tokens::LARGE_SIZE))
                .min_w(px(tokens::LARGE_SIZE))
                .px(px(layout::BADGE_HORIZONTAL_SPACE))
                .flex()
                .items_center()
                .justify_center()
                .corner_radius(theme.corner(tokens::LARGE_SHAPE))
                .bg(theme.color(tokens::LARGE_COLOR))
                .text_color(theme.color(tokens::LARGE_LABEL_TEXT_COLOR))
                .type_style(&theme.text_style(tokens::LARGE_LABEL_TEXT_FONT))
                .child(label),
        }
    }
}

/// アンカーの大きさを変えず、右上にバッジを重ねる。
///
/// ブロックの親で外側の箱が親の幅に広がっても、配置の基準はアンカーとする。
/// アンカーは自身の大きさが決まる要素にする。`.w_full()` のように親に対する割合で幅を決める要素は幅が 0 になる。
#[derive(IntoElement)]
pub struct BadgedBox {
    anchor: gpui::AnyElement,
    badge: Badge,
}

impl BadgedBox {
    /// アンカーとバッジから作る。
    pub fn new(anchor: impl IntoElement, badge: Badge) -> Self {
        Self {
            anchor: anchor.into_any_element(),
            badge,
        }
    }
}

impl RenderOnce for BadgedBox {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let (horizontal, vertical) = if self.badge.label().is_some() {
            (
                layout::BADGE_WITH_CONTENT_HORIZONTAL_OFFSET,
                layout::BADGE_WITH_CONTENT_VERTICAL_OFFSET,
            )
        } else {
            (layout::BADGE_OFFSET, layout::BADGE_OFFSET)
        };
        div().flex().child(
            div()
                .relative()
                .flex_none()
                .debug_selector(|| "md3-badged-box".into())
                .child(self.anchor)
                .child(
                    div()
                        .absolute()
                        .left(relative(1.0))
                        .top(px(0.0))
                        .size(px(0.0))
                        .child(
                            div()
                                .absolute()
                                .debug_selector(|| "md3-badged-box-badge".into())
                                .left(px(-horizontal))
                                .bottom(px(-vertical))
                                .child(self.badge),
                        ),
                ),
        )
    }
}
