// Ported from Jetpack Compose (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/RadioButton.kt @ b8256a52
// Copyright 2020 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
//! RadioButton（RadioGroup としてまとめて使う）。

use gpui::{
    AnimationExt, App, ElementId, IntoElement, RenderOnce, SharedString, SpringAnimation, Window,
    div, prelude::*, px,
};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::radio_button as tokens;
use md3_theme::md3_tokens::layout;
use md3_theme::{CornerRadius, Md3ThemeExt, MotionToken, TypeScale};

use crate::{FocusRing, StateLayer, Text};

/// ラジオボタンのグループ。選択肢ごとにラジオボタンとラベルを縦に並べる。
///
/// APG の Radio Group に従い、Tab で止まるのは選択中の項目（無ければ最初の項目）だけで、
/// 矢印キーでグループ内を移ると同時に選択する。
#[derive(IntoElement)]
pub struct RadioGroup {
    inner: gpui_headless::RadioGroup,
    horizontal: bool,
}
impl RadioGroup {
    /// ID とグループのラベル（読み上げ用の名前）から作る。
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            inner: gpui_headless::RadioGroup::new(id, label)
                .reveal_margin(crate::focus::reveal_margin())
                .group_prefix("md3"),
            horizontal: false,
        }
    }
    /// 選択肢を加える。
    pub fn option(mut self, label: impl Into<SharedString>) -> Self {
        self.inner = self.inner.option(label);
        self
    }
    /// 無効な選択肢を加える。
    pub fn disabled_option(mut self, label: impl Into<SharedString>) -> Self {
        self.inner = self.inner.disabled_option(label);
        self
    }
    /// 選択中の項目。
    pub fn selected(mut self, selected: Option<usize>) -> Self {
        self.inner = self.inner.selected(selected);
        self
    }
    /// 横に並べる。
    pub fn horizontal(mut self, horizontal: bool) -> Self {
        self.horizontal = horizontal;
        self
    }
    /// 選択が変わったときの処理。
    pub fn on_change(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.inner = self.inner.on_change(handler);
        self
    }
}

/// 選択状態ごとの円の色。
pub(crate) fn icon_color(selected: bool) -> (ColorRole, ColorRole) {
    if selected {
        (
            tokens::SELECTED_ICON_COLOR,
            tokens::SELECTED_HOVER_ICON_COLOR,
        )
    } else {
        (
            tokens::UNSELECTED_ICON_COLOR,
            tokens::UNSELECTED_HOVER_ICON_COLOR,
        )
    }
}

impl RenderOnce for RadioGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let horizontal = self.horizontal;
        let group = self.inner.build(window, cx, |item, _window, cx| {
            let gpui_headless::RadioItem {
                base,
                state: control,
                label,
                selected,
                ..
            } = item;
            let disabled = control.disabled;
            let theme = cx.md3();
            let (role, hovered_role) = icon_color(selected);
            let color = if disabled {
                let opacity = if selected {
                    tokens::DISABLED_SELECTED_ICON_OPACITY
                } else {
                    tokens::DISABLED_UNSELECTED_ICON_OPACITY
                };
                theme
                    .color(tokens::DISABLED_SELECTED_ICON_COLOR)
                    .opacity(opacity)
            } else {
                theme.color(role)
            };
            let hovered = theme.color(hovered_role);
            let state_size = theme.dense(tokens::STATE_LAYER_SIZE);
            let radius = CornerRadius::Full;

            let mut ring = div()
                .size(px(tokens::ICON_SIZE))
                .rounded_full()
                .border(px(layout::RADIO_BUTTON_STROKE_WIDTH))
                .border_color(color)
                .flex()
                .items_center()
                .justify_center();
            if !disabled {
                ring = ring.group_hover(control.group.clone(), move |s| s.border_color(hovered));
            }
            let mut dot = div().rounded_full().bg(color);
            if !disabled {
                dot = dot.group_hover(control.group.clone(), move |s| s.bg(hovered));
            }
            let dot = dot.with_spring(
                "radio-dot-size",
                SpringAnimation::new(theme.spring(MotionToken::FastSpatial)).to(if selected {
                    layout::RADIO_BUTTON_DOT_SIZE
                } else {
                    0.0
                }),
                |dot, size| dot.size(px(size.max(0.0))),
            );
            ring = ring.child(dot);
            let radio = div()
                .relative()
                .size(state_size)
                .flex()
                .items_center()
                .justify_center()
                .child(ring)
                .child(
                    StateLayer::new(
                        SharedString::from(format!("{}-layer", control.group)),
                        control.group.clone(),
                        role,
                    )
                    .radius(radius)
                    .focused(control.focus_visible)
                    .disabled(disabled),
                )
                .child(FocusRing::new(radius).visible(control.focus_visible));

            let label_color = if disabled {
                theme
                    .color(ColorRole::OnSurface)
                    .opacity(tokens::DISABLED_UNSELECTED_ICON_OPACITY)
            } else {
                theme.color(ColorRole::OnSurface)
            };
            let item = base
                .flex()
                .items_center()
                .pr_2()
                .text_color(label_color)
                .child(radio)
                .child(Text::new(TypeScale::BodyLarge, label));
            item.into_any_element()
        });
        if horizontal {
            group.flex().flex_row().flex_wrap().gap_4()
        } else {
            group.flex().flex_col()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selected_uses_primary() {
        assert_eq!(icon_color(true).0, ColorRole::Primary);
        assert_eq!(icon_color(false).0, ColorRole::OnSurfaceVariant);
    }
}
