// Ported from Jetpack Compose (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TextField.kt @ b8256a52
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
//! テキストフィールドの外枠（コンテナ、ラベル、下線 / 枠線、アイコン、補助テキスト）。
//!
//! [`TextField`](super::TextField) と [`Select`](super::Select) が共有する。入力部分は呼び出し側が渡す。

use gpui::{
    AnimationExt, AnyElement, App, Bounds, Hsla, PathBuilder, Pixels, SharedString,
    SpringAnimation, SpringPlayback, Window, canvas, div, point, prelude::*, px,
};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::{
    filled_text_field as filled, outlined_text_field as outlined,
};
use md3_theme::md3_tokens::layout;
use md3_theme::{CornerRadius, Md3ThemeExt, MotionToken, Theme, TypeScale};

use crate::{Icon, IconName, StyledExt};

/// テキストフィールドの種類。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextFieldVariant {
    /// 塗りつぶしのコンテナと下線。
    #[default]
    Filled,
    /// 枠線。
    Outlined,
}

impl TextFieldVariant {
    /// すべての種類。
    pub const ALL: [TextFieldVariant; 2] = [TextFieldVariant::Filled, TextFieldVariant::Outlined];
}

/// 外枠の状態。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct FieldState {
    pub focused: bool,
    pub hovered: bool,
    pub error: bool,
    pub disabled: bool,
}

/// 状態ごとの色のロールと不透明度。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FieldColors {
    pub indicator: (ColorRole, f32),
    pub indicator_width: f32,
    pub label: (ColorRole, f32),
    pub input: (ColorRole, f32),
    pub leading_icon: (ColorRole, f32),
    pub trailing_icon: (ColorRole, f32),
    pub supporting: (ColorRole, f32),
    pub caret: ColorRole,
}

impl FieldColors {
    /// トークンから色を選ぶ。Filled と Outlined で色の組み合わせは同じで、下線と枠線のトークン名だけが違う。
    pub(crate) fn of(variant: TextFieldVariant, state: FieldState) -> Self {
        let opaque = |role| (role, 1.0);
        if state.disabled {
            let (indicator, indicator_width) = match variant {
                TextFieldVariant::Filled => (
                    (
                        filled::DISABLED_ACTIVE_INDICATOR_COLOR,
                        filled::DISABLED_ACTIVE_INDICATOR_OPACITY,
                    ),
                    filled::DISABLED_ACTIVE_INDICATOR_HEIGHT,
                ),
                TextFieldVariant::Outlined => (
                    (
                        outlined::DISABLED_OUTLINE_COLOR,
                        outlined::DISABLED_OUTLINE_OPACITY,
                    ),
                    outlined::DISABLED_OUTLINE_WIDTH,
                ),
            };
            return Self {
                indicator,
                indicator_width,
                label: (filled::DISABLED_LABEL_COLOR, filled::DISABLED_LABEL_OPACITY),
                input: (filled::DISABLED_INPUT_COLOR, filled::DISABLED_INPUT_OPACITY),
                leading_icon: (
                    filled::DISABLED_LEADING_ICON_COLOR,
                    filled::DISABLED_LEADING_ICON_OPACITY,
                ),
                trailing_icon: (
                    filled::DISABLED_TRAILING_ICON_COLOR,
                    filled::DISABLED_TRAILING_ICON_OPACITY,
                ),
                supporting: (
                    filled::DISABLED_SUPPORTING_COLOR,
                    filled::DISABLED_SUPPORTING_OPACITY,
                ),
                caret: filled::CARET_COLOR,
            };
        }
        let (indicator, indicator_width) =
            match (variant, state.error, state.focused, state.hovered) {
                (TextFieldVariant::Filled, true, true, _) => (
                    filled::ERROR_FOCUS_ACTIVE_INDICATOR_COLOR,
                    filled::FOCUS_ACTIVE_INDICATOR_HEIGHT,
                ),
                (TextFieldVariant::Filled, true, false, true) => (
                    filled::ERROR_HOVER_ACTIVE_INDICATOR_COLOR,
                    filled::HOVER_ACTIVE_INDICATOR_HEIGHT,
                ),
                (TextFieldVariant::Filled, true, false, false) => (
                    filled::ERROR_ACTIVE_INDICATOR_COLOR,
                    filled::ACTIVE_INDICATOR_HEIGHT,
                ),
                (TextFieldVariant::Filled, false, true, _) => (
                    filled::FOCUS_ACTIVE_INDICATOR_COLOR,
                    filled::FOCUS_ACTIVE_INDICATOR_HEIGHT,
                ),
                (TextFieldVariant::Filled, false, false, true) => (
                    filled::HOVER_ACTIVE_INDICATOR_COLOR,
                    filled::HOVER_ACTIVE_INDICATOR_HEIGHT,
                ),
                (TextFieldVariant::Filled, false, false, false) => (
                    filled::ACTIVE_INDICATOR_COLOR,
                    filled::ACTIVE_INDICATOR_HEIGHT,
                ),
                (TextFieldVariant::Outlined, true, true, _) => (
                    outlined::ERROR_FOCUS_OUTLINE_COLOR,
                    outlined::FOCUS_OUTLINE_WIDTH,
                ),
                (TextFieldVariant::Outlined, true, false, true) => (
                    outlined::ERROR_HOVER_OUTLINE_COLOR,
                    outlined::HOVER_OUTLINE_WIDTH,
                ),
                (TextFieldVariant::Outlined, true, false, false) => {
                    (outlined::ERROR_OUTLINE_COLOR, outlined::OUTLINE_WIDTH)
                }
                (TextFieldVariant::Outlined, false, true, _) => {
                    (outlined::FOCUS_OUTLINE_COLOR, outlined::FOCUS_OUTLINE_WIDTH)
                }
                (TextFieldVariant::Outlined, false, false, true) => {
                    (outlined::HOVER_OUTLINE_COLOR, outlined::HOVER_OUTLINE_WIDTH)
                }
                (TextFieldVariant::Outlined, false, false, false) => {
                    (outlined::OUTLINE_COLOR, outlined::OUTLINE_WIDTH)
                }
            };
        let (label, input, leading_icon, trailing_icon, supporting, caret) =
            match (state.error, state.focused, state.hovered) {
                (true, true, _) => (
                    filled::ERROR_FOCUS_LABEL_COLOR,
                    filled::ERROR_FOCUS_INPUT_COLOR,
                    filled::ERROR_FOCUS_LEADING_ICON_COLOR,
                    filled::ERROR_FOCUS_TRAILING_ICON_COLOR,
                    filled::ERROR_FOCUS_SUPPORTING_COLOR,
                    filled::ERROR_FOCUS_CARET_COLOR,
                ),
                (true, false, true) => (
                    filled::ERROR_HOVER_LABEL_COLOR,
                    filled::ERROR_HOVER_INPUT_COLOR,
                    filled::ERROR_HOVER_LEADING_ICON_COLOR,
                    filled::ERROR_HOVER_TRAILING_ICON_COLOR,
                    filled::ERROR_HOVER_SUPPORTING_COLOR,
                    filled::ERROR_FOCUS_CARET_COLOR,
                ),
                (true, false, false) => (
                    filled::ERROR_LABEL_COLOR,
                    filled::ERROR_INPUT_COLOR,
                    filled::ERROR_LEADING_ICON_COLOR,
                    filled::ERROR_TRAILING_ICON_COLOR,
                    filled::ERROR_SUPPORTING_COLOR,
                    filled::ERROR_FOCUS_CARET_COLOR,
                ),
                (false, true, _) => (
                    filled::FOCUS_LABEL_COLOR,
                    filled::FOCUS_INPUT_COLOR,
                    filled::FOCUS_LEADING_ICON_COLOR,
                    filled::FOCUS_TRAILING_ICON_COLOR,
                    filled::FOCUS_SUPPORTING_COLOR,
                    filled::CARET_COLOR,
                ),
                (false, false, true) => (
                    filled::HOVER_LABEL_COLOR,
                    filled::HOVER_INPUT_COLOR,
                    filled::HOVER_LEADING_ICON_COLOR,
                    filled::HOVER_TRAILING_ICON_COLOR,
                    filled::HOVER_SUPPORTING_COLOR,
                    filled::CARET_COLOR,
                ),
                (false, false, false) => (
                    filled::LABEL_COLOR,
                    filled::INPUT_COLOR,
                    filled::LEADING_ICON_COLOR,
                    filled::TRAILING_ICON_COLOR,
                    filled::SUPPORTING_COLOR,
                    filled::CARET_COLOR,
                ),
            };
        Self {
            indicator: opaque(indicator),
            indicator_width,
            label: opaque(label),
            input: opaque(input),
            leading_icon: opaque(leading_icon),
            trailing_icon: opaque(trailing_icon),
            supporting: opaque(supporting),
            caret,
        }
    }
}

/// ロールと不透明度から色を作る。
pub(crate) fn tint(theme: &Theme, (role, opacity): (ColorRole, f32)) -> Hsla {
    theme.color(role).opacity(opacity)
}

/// 外枠を組み立てるための値。
pub(crate) struct FieldFrame {
    pub variant: TextFieldVariant,
    pub state: FieldState,
    pub label: Option<SharedString>,
    /// ラベルを上に小さく出すか（フォーカス中か、値があるとき）。
    pub floated: bool,
    pub leading_icon: Option<IconName>,
    pub trailing: Option<AnyElement>,
    /// 入力行（接頭辞・入力・接尾辞）。1 行の欄はラベルが上に出ていないときは描かない。
    pub content: AnyElement,
    /// 複数行の入力か。入力を上に寄せ、ラベルの有無にかかわらず入力の高さを確保する。
    pub multi_line: bool,
    pub supporting_text: Option<SharedString>,
    pub supporting_a11y: Option<(gpui::ElementId, gpui_headless::A11yNode)>,
    pub counter: Option<SharedString>,
}

impl FieldFrame {
    /// コンテナ（最小 56dp の行）と補助テキストの行を返す。コンテナは呼び出し側で操作要素にする。
    pub(crate) fn render(self, cx: &App) -> (gpui::Div, Option<gpui::Div>) {
        let theme = cx.md3();
        let colors = FieldColors::of(self.variant, self.state);
        let corner = theme.corner(filled::CONTAINER_SHAPE);
        let label_large = theme.text_style(filled::LABEL_FONT);
        let label_small = theme.text_style(filled::SUPPORTING_FONT);
        let space = px(layout::TEXT_FIELD_HORIZONTAL_SPACE);
        // アイコンは 48dp の操作領域の中央に置き、文字はその外側から TextFieldPadding − 12dp 離す
        // （Compose の textFieldHorizontalIconPadding）。
        let icon_slot = px(layout::MIN_INTERACTIVE_SIZE);
        let icon_padding = (icon_slot - px(filled::LEADING_ICON_SIZE)) / 2.0;
        let text_start = if self.leading_icon.is_some() {
            (space - icon_padding).max(px(0.0))
        } else {
            space
        };
        let text_end = if self.trailing.is_some() {
            (space - icon_padding).max(px(0.0))
        } else {
            space
        };
        let label_color = tint(theme, colors.label);
        // Filled のトークンには高さがないため Outlined の値を使う（どちらも 56dp。TextFieldDefaults.MinHeight と同じ）。
        let height = px(outlined::CONTAINER_HEIGHT);
        // ラベルが浮いても高さが変わらないよう、Filled はその分を先に確保する（Compose の TextFieldImpl）。
        let height = if self.variant == TextFieldVariant::Filled && self.label.is_some() {
            height.max(label_small.line_height + theme.text_style(filled::INPUT_FONT).line_height)
        } else {
            height
        };

        // 中央の列（ラベルと入力行）。
        let label_in_column = self.variant == TextFieldVariant::Filled && self.label.is_some();
        let floated_label_in_column = label_in_column && self.floated;
        let mut column = div()
            .relative()
            .flex_1()
            .min_w_0()
            .h_full()
            .flex()
            .flex_col()
            .pl(text_start)
            .pr(text_end);
        if self.multi_line {
            // Compose の contentPaddingWithLabel / contentPaddingWithoutLabel。
            let vertical = if label_in_column {
                px(layout::TEXT_FIELD_WITH_LABEL_VERTICAL_SPACE)
            } else {
                space
            };
            column = column.justify_start().pt(vertical).pb(vertical);
            if label_in_column && !self.floated {
                // ラベルが浮いても高さが変わらないよう、その分を空ける。
                column = column.child(div().flex_none().h(label_small.line_height));
            }
            if !self.floated
                && let Some(label) = self.label.clone()
            {
                // 空の入力の上に重ねる。位置は TextFieldMeasurePolicy の labelStartY
                // （Filled は上の余白 + 小さいラベルの高さの半分）と startY（Outlined は上の余白）。
                let top = if label_in_column {
                    vertical + label_small.line_height / 2.0
                } else {
                    vertical
                };
                column = column.child(
                    div()
                        .absolute()
                        .top(top)
                        .left(text_start)
                        .right(text_end)
                        .type_style(&label_large)
                        .text_color(label_color)
                        .whitespace_nowrap()
                        .overflow_hidden()
                        .text_ellipsis()
                        .child(label),
                );
            }
        } else {
            column = column.justify_center();
        }
        if floated_label_in_column {
            column = column.child(
                div()
                    .type_style(&label_small)
                    .text_color(label_color)
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .text_ellipsis()
                    .children(self.label.clone()),
            );
        }
        if self.multi_line || self.floated || self.label.is_none() {
            column = column.child(
                div()
                    .type_style(&theme.text_style(filled::INPUT_FONT))
                    .text_color(tint(theme, colors.input))
                    .child(self.content),
            );
        } else if let Some(label) = self.label.clone() {
            column = column.child(
                div()
                    .type_style(&label_large)
                    .text_color(label_color)
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(label),
            );
        }

        let mut container = div()
            .relative()
            .w_full()
            .min_h(height)
            .flex()
            .items_center()
            .when_some(self.leading_icon, |this, icon| {
                this.child(
                    div()
                        .flex_none()
                        .size(icon_slot)
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            Icon::new(icon)
                                .size(px(filled::LEADING_ICON_SIZE))
                                .tint(tint(theme, colors.leading_icon)),
                        ),
                )
            })
            .child(column)
            .when_some(self.trailing, |this, trailing| {
                this.child(
                    div()
                        .flex_none()
                        .size(icon_slot)
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(tint(theme, colors.trailing_icon))
                        .child(trailing),
                )
            });

        let indicator = tint(theme, colors.indicator);
        let indicator_width = px(colors.indicator_width);
        match self.variant {
            TextFieldVariant::Filled => {
                let background = if self.state.disabled {
                    theme
                        .color(filled::DISABLED_CONTAINER_COLOR)
                        .opacity(filled::DISABLED_CONTAINER_OPACITY)
                } else {
                    theme.color(filled::CONTAINER_COLOR)
                };
                let radius = match corner {
                    CornerRadius::Fixed(radius) => radius,
                    CornerRadius::Full => height / 2.0,
                };
                container = container.bg(background).rounded_t(radius).child(
                    div()
                        .absolute()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .bg(indicator)
                        .with_spring(
                            "field-indicator-thickness",
                            SpringAnimation::new(theme.spring(MotionToken::FastSpatial))
                                .to(indicator_width)
                                .playback(if self.state.disabled {
                                    SpringPlayback::Completed
                                } else {
                                    SpringPlayback::Running
                                }),
                            |indicator, width| indicator.h(width),
                        ),
                );
            }
            TextFieldVariant::Outlined => {
                let radius = match corner {
                    CornerRadius::Fixed(radius) => radius,
                    CornerRadius::Full => height / 2.0,
                };
                let label = self.label.clone().filter(|_| self.floated);
                let gap_start = text_start
                    + if self.leading_icon.is_some() {
                        icon_slot
                    } else {
                        px(0.0)
                    }
                    - px(layout::OUTLINED_TEXT_FIELD_LABEL_GAP);
                // 枠線は上辺のラベルの部分を切り欠いて描く。ラベルの幅は描画時に測る。
                let font_size = label_small.font_size;
                let small = label_small.clone();
                container = container.child(
                    div().absolute().inset_0().type_style(&small).child(
                        canvas(
                            move |_, window, _| {
                                label.as_ref().map(|label| {
                                    let style = window.text_style();
                                    let run = style.to_run(label.len());
                                    window
                                        .text_system()
                                        .shape_line(label.clone(), font_size, &[run], None)
                                        .width
                                })
                            },
                            move |bounds, label_width, window, _| {
                                let gap = label_width.map(|width| {
                                    (
                                        gap_start,
                                        gap_start
                                            + width
                                            + px(layout::OUTLINED_TEXT_FIELD_LABEL_GAP) * 2.0,
                                    )
                                });
                                paint_outline(
                                    bounds,
                                    radius,
                                    indicator_width,
                                    gap,
                                    indicator,
                                    window,
                                );
                            },
                        )
                        .size_full(),
                    ),
                );
                let line_height = label_small.line_height;
                // 枠線にかかるラベルの分を上に空ける（Compose の minimizedLabelHalfHeight）。
                if self.label.is_some() {
                    container = container.mt(line_height / 2.0);
                }
                if let Some(label) = self.label.clone().filter(|_| self.floated) {
                    container = container.child(
                        div()
                            .absolute()
                            .top(-line_height / 2.0)
                            .left(gap_start + px(layout::OUTLINED_TEXT_FIELD_LABEL_GAP))
                            .type_style(&label_small)
                            .text_color(label_color)
                            .whitespace_nowrap()
                            .child(label),
                    );
                }
            }
        }

        use gpui_headless::A11yExt;
        let has_supporting_text = self.supporting_text.is_some();
        let supporting_text = div().flex_1();
        let supporting_text = match self.supporting_a11y {
            Some((id, node)) => supporting_text
                .id(id)
                .a11y(node)
                .children(self.supporting_text)
                .into_any_element(),
            None => supporting_text
                .children(self.supporting_text)
                .into_any_element(),
        };
        let supporting = (has_supporting_text || self.counter.is_some()).then(|| {
            div()
                .w_full()
                .flex()
                .gap(space)
                .pt(px(layout::TEXT_FIELD_SUPPORTING_TOP_SPACE))
                .px(space)
                .type_style(&theme.text_style(TypeScale::BodySmall))
                .text_color(tint(theme, colors.supporting))
                .child(supporting_text)
                .children(self.counter.map(|counter| div().flex_none().child(counter)))
        });
        (container, supporting)
    }
}

/// 枠線（角丸の長方形）を描く。`gap` は上辺の切り欠き（左端からの距離の範囲）。
fn paint_outline(
    bounds: Bounds<Pixels>,
    radius: Pixels,
    width: Pixels,
    gap: Option<(Pixels, Pixels)>,
    color: Hsla,
    window: &mut Window,
) {
    // 線の中心を内側に半分ずらし、線全体が bounds に収まるようにする。
    let inset = width / 2.0;
    let left = bounds.left() + inset;
    let right = bounds.right() - inset;
    let top = bounds.top() + inset;
    let bottom = bounds.bottom() - inset;
    let r = (radius - inset).max(px(0.0));
    let radii = point(r, r);
    let mut path = PathBuilder::stroke(width);
    let (start, end) = match gap {
        Some((from, to)) => (bounds.left() + to, bounds.left() + from),
        None => (left + r, left + r),
    };
    path.move_to(point(start.max(left + r), top));
    path.line_to(point(right - r, top));
    path.arc_to(radii, px(0.0), false, true, point(right, top + r));
    path.line_to(point(right, bottom - r));
    path.arc_to(radii, px(0.0), false, true, point(right - r, bottom));
    path.line_to(point(left + r, bottom));
    path.arc_to(radii, px(0.0), false, true, point(left, bottom - r));
    path.line_to(point(left, top + r));
    path.arc_to(radii, px(0.0), false, true, point(left + r, top));
    path.line_to(point(end.max(left + r), top));
    if let Ok(path) = path.build() {
        window.paint_path(path, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focused_field_uses_primary_and_thick_indicator() {
        let state = FieldState {
            focused: true,
            ..FieldState::default()
        };
        for variant in TextFieldVariant::ALL {
            let colors = FieldColors::of(variant, state);
            assert_eq!(colors.indicator.0, ColorRole::Primary);
            assert_eq!(colors.indicator_width, 2.0);
            assert_eq!(colors.label.0, ColorRole::Primary);
        }
    }

    #[test]
    fn error_overrides_focus_colors() {
        let state = FieldState {
            focused: true,
            error: true,
            ..FieldState::default()
        };
        let colors = FieldColors::of(TextFieldVariant::Outlined, state);
        assert_eq!(colors.indicator.0, ColorRole::Error);
        assert_eq!(colors.supporting.0, ColorRole::Error);
        assert_eq!(colors.caret, ColorRole::Error);
    }

    #[test]
    fn disabled_field_is_translucent() {
        let state = FieldState {
            disabled: true,
            focused: true,
            ..FieldState::default()
        };
        let colors = FieldColors::of(TextFieldVariant::Filled, state);
        assert_eq!(colors.input, (ColorRole::OnSurface, 0.38));
    }
}
