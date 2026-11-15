// 入力欄の並べ方・余白・色・書体は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TextField.kt、
//         OutlinedTextField.kt (`outlineCutout`)、
//         internal/TextFieldImpl.kt (`CommonDecorationBox`、`TextFieldLayout`、`TextFieldMeasurePolicy`、
//         `CutoutTextFieldLayout`、`OutlinedTextFieldMeasurePolicy`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021, 2023 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
//
// hover での入力欄の色は Compose が扱わないため Material Web から移植している
// (https://github.com/material-components/material-web)
// Source: field/internal/_filled-field.scss、field/internal/_outlined-field.scss
//         @ de5470f7ed6a653457ca8f5cc0e7f7602c7a8391
// Copyright 2023 Google LLC. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! テキストフィールドとセレクトが共有する、MD3 の入力欄の見た目

use gpui::{
    AnyElement, Div, Hsla, InteractiveElement, IntoElement, ParentElement, Radians, Refineable,
    SharedString, Styled, TextStyleRefinement, Transformation, Window, div, prelude::FluentBuilder,
    px, size, svg,
};
use md3_icons::StyledIcon;

use crate::shape::Shaped;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::{
    filled_text_field as filled, outlined_text_field as outlined,
};
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::md3_tokens::{Dp, supplement::text_field as supplement};
use md3_theme::{SharedTheme, dp};

/// 入力欄の種類
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum FieldVariant {
    /// 容器を塗り、下端に線を引く
    Filled,
    /// 容器を塗らず、枠線で囲む
    Outlined,
}

/// 入力欄の操作の状態
#[derive(Clone, Copy, PartialEq, Eq)]
enum Interaction {
    Disabled,
    Rest,
    Hover,
    Focus,
}

/// 入力欄の色のロールと不透明度、線の太さ
#[derive(Clone, Copy)]
#[cfg_attr(
    not(feature = "text-field"),
    expect(
        dead_code,
        reason = "プレースホルダーとカーソルの色はテキストフィールドだけが使う"
    )
)]
struct FieldRoles {
    line: (ColorRole, f32),
    line_width: Dp,
    label: (ColorRole, f32),
    input: (ColorRole, f32),
    leading: (ColorRole, f32),
    trailing: (ColorRole, f32),
    supporting: (ColorRole, f32),
    affix: (ColorRole, f32),
    placeholder: (ColorRole, f32),
    caret: ColorRole,
}

/// `$tokens` の入力欄のトークンから、操作の状態とエラーかどうかに応じた [`FieldRoles`] を返す関数 `$name` を作る。
/// 線のトークンの名前は Filled と Outlined で異なるため、`line` と `width` で渡す
macro_rules! field_roles {
    (
        $name:ident, $tokens:ident,
        line: [$line:ident, $hover:ident, $focus:ident, $error:ident, $error_hover:ident,
            $error_focus:ident, $disabled:ident, $disabled_opacity:ident],
        width: [$width:ident, $hover_width:ident, $focus_width:ident, $disabled_width:ident]
    ) => {
        fn $name(interaction: Interaction, error: bool) -> FieldRoles {
            use $tokens as t;
            let opaque = |role| (role, 1.0);
            match (interaction, error) {
                (Interaction::Disabled, _) => FieldRoles {
                    line: (t::$disabled, t::$disabled_opacity),
                    line_width: t::$disabled_width,
                    label: (t::DISABLED_LABEL_COLOR, t::DISABLED_LABEL_OPACITY),
                    input: (t::DISABLED_INPUT_COLOR, t::DISABLED_INPUT_OPACITY),
                    leading: (
                        t::DISABLED_LEADING_ICON_COLOR,
                        t::DISABLED_LEADING_ICON_OPACITY,
                    ),
                    trailing: (
                        t::DISABLED_TRAILING_ICON_COLOR,
                        t::DISABLED_TRAILING_ICON_OPACITY,
                    ),
                    supporting: (t::DISABLED_SUPPORTING_COLOR, t::DISABLED_SUPPORTING_OPACITY),
                    // トークンに無効な接辞とプレースホルダーの値は無く、Compose は入力の色を使う
                    affix: (t::DISABLED_INPUT_COLOR, t::DISABLED_INPUT_OPACITY),
                    placeholder: (t::DISABLED_INPUT_COLOR, t::DISABLED_INPUT_OPACITY),
                    caret: t::CARET_COLOR,
                },
                (Interaction::Rest, false) => FieldRoles {
                    line: opaque(t::$line),
                    line_width: t::$width,
                    label: opaque(t::LABEL_COLOR),
                    input: opaque(t::INPUT_COLOR),
                    leading: opaque(t::LEADING_ICON_COLOR),
                    trailing: opaque(t::TRAILING_ICON_COLOR),
                    supporting: opaque(t::SUPPORTING_COLOR),
                    affix: opaque(t::INPUT_PREFIX_COLOR),
                    placeholder: opaque(t::INPUT_PLACEHOLDER_COLOR),
                    caret: t::CARET_COLOR,
                },
                (Interaction::Hover, false) => FieldRoles {
                    line: opaque(t::$hover),
                    line_width: t::$hover_width,
                    label: opaque(t::HOVER_LABEL_COLOR),
                    input: opaque(t::HOVER_INPUT_COLOR),
                    leading: opaque(t::HOVER_LEADING_ICON_COLOR),
                    trailing: opaque(t::HOVER_TRAILING_ICON_COLOR),
                    supporting: opaque(t::HOVER_SUPPORTING_COLOR),
                    affix: opaque(t::INPUT_PREFIX_COLOR),
                    placeholder: opaque(t::INPUT_PLACEHOLDER_COLOR),
                    caret: t::CARET_COLOR,
                },
                (Interaction::Focus, false) => FieldRoles {
                    line: opaque(t::$focus),
                    line_width: t::$focus_width,
                    label: opaque(t::FOCUS_LABEL_COLOR),
                    input: opaque(t::FOCUS_INPUT_COLOR),
                    leading: opaque(t::FOCUS_LEADING_ICON_COLOR),
                    trailing: opaque(t::FOCUS_TRAILING_ICON_COLOR),
                    supporting: opaque(t::FOCUS_SUPPORTING_COLOR),
                    affix: opaque(t::INPUT_PREFIX_COLOR),
                    placeholder: opaque(t::INPUT_PLACEHOLDER_COLOR),
                    caret: t::CARET_COLOR,
                },
                (Interaction::Rest, true) => FieldRoles {
                    line: opaque(t::$error),
                    line_width: t::$width,
                    label: opaque(t::ERROR_LABEL_COLOR),
                    input: opaque(t::ERROR_INPUT_COLOR),
                    leading: opaque(t::ERROR_LEADING_ICON_COLOR),
                    trailing: opaque(t::ERROR_TRAILING_ICON_COLOR),
                    supporting: opaque(t::ERROR_SUPPORTING_COLOR),
                    affix: opaque(t::INPUT_PREFIX_COLOR),
                    placeholder: opaque(t::INPUT_PLACEHOLDER_COLOR),
                    caret: t::ERROR_FOCUS_CARET_COLOR,
                },
                (Interaction::Hover, true) => FieldRoles {
                    line: opaque(t::$error_hover),
                    line_width: t::$hover_width,
                    label: opaque(t::ERROR_HOVER_LABEL_COLOR),
                    input: opaque(t::ERROR_HOVER_INPUT_COLOR),
                    leading: opaque(t::ERROR_HOVER_LEADING_ICON_COLOR),
                    trailing: opaque(t::ERROR_HOVER_TRAILING_ICON_COLOR),
                    supporting: opaque(t::ERROR_HOVER_SUPPORTING_COLOR),
                    affix: opaque(t::INPUT_PREFIX_COLOR),
                    placeholder: opaque(t::INPUT_PLACEHOLDER_COLOR),
                    caret: t::ERROR_FOCUS_CARET_COLOR,
                },
                (Interaction::Focus, true) => FieldRoles {
                    line: opaque(t::$error_focus),
                    line_width: t::$focus_width,
                    label: opaque(t::ERROR_FOCUS_LABEL_COLOR),
                    input: opaque(t::ERROR_FOCUS_INPUT_COLOR),
                    leading: opaque(t::ERROR_FOCUS_LEADING_ICON_COLOR),
                    trailing: opaque(t::ERROR_FOCUS_TRAILING_ICON_COLOR),
                    supporting: opaque(t::ERROR_FOCUS_SUPPORTING_COLOR),
                    affix: opaque(t::INPUT_PREFIX_COLOR),
                    placeholder: opaque(t::INPUT_PLACEHOLDER_COLOR),
                    caret: t::ERROR_FOCUS_CARET_COLOR,
                },
            }
        }
    };
}

field_roles!(
    filled_roles, filled,
    line: [ACTIVE_INDICATOR_COLOR, HOVER_ACTIVE_INDICATOR_COLOR, FOCUS_ACTIVE_INDICATOR_COLOR,
        ERROR_ACTIVE_INDICATOR_COLOR, ERROR_HOVER_ACTIVE_INDICATOR_COLOR,
        ERROR_FOCUS_ACTIVE_INDICATOR_COLOR, DISABLED_ACTIVE_INDICATOR_COLOR,
        DISABLED_ACTIVE_INDICATOR_OPACITY],
    width: [ACTIVE_INDICATOR_HEIGHT, HOVER_ACTIVE_INDICATOR_HEIGHT, FOCUS_ACTIVE_INDICATOR_HEIGHT,
        DISABLED_ACTIVE_INDICATOR_HEIGHT]
);

field_roles!(
    outlined_roles, outlined,
    line: [OUTLINE_COLOR, HOVER_OUTLINE_COLOR, FOCUS_OUTLINE_COLOR, ERROR_OUTLINE_COLOR,
        ERROR_HOVER_OUTLINE_COLOR, ERROR_FOCUS_OUTLINE_COLOR, DISABLED_OUTLINE_COLOR,
        DISABLED_OUTLINE_OPACITY],
    width: [OUTLINE_WIDTH, HOVER_OUTLINE_WIDTH, FOCUS_OUTLINE_WIDTH, DISABLED_OUTLINE_WIDTH]
);

/// 入力欄の内容と状態
pub(crate) struct Field {
    pub variant: FieldVariant,
    /// 入力欄とその子孫の hover を結ぶ group の名前。[`Field::render`] に渡す容器にこの名前の group を付けること
    pub group: &'static str,
    pub label: Option<SharedString>,
    pub leading_icon: Option<StyledIcon>,
    pub trailing_icon: Option<StyledIcon>,
    /// 末尾のアイコンを上下に反転して描くか
    pub flip_trailing_icon: bool,
    pub prefix: Option<SharedString>,
    pub suffix: Option<SharedString>,
    pub supporting_text: Option<SharedString>,
    pub error: bool,
    pub disabled: bool,
    pub focused: bool,
    /// 値を持つか
    pub populated: bool,
    pub theme: SharedTheme,
}

/// テキストフィールドの文の色
#[cfg(feature = "text-field")]
pub(crate) struct InputColors {
    pub placeholder: Hsla,
    pub caret: Hsla,
    pub selection: Hsla,
}

impl Field {
    /// 今の状態と、hover の間の色のロール。hover の色が今の色と変わらなければ hover は `None`
    fn roles(&self) -> (FieldRoles, Option<FieldRoles>) {
        let roles = match self.variant {
            FieldVariant::Filled => filled_roles,
            FieldVariant::Outlined => outlined_roles,
        };
        let rest = roles(
            if self.disabled {
                Interaction::Disabled
            } else if self.focused {
                Interaction::Focus
            } else {
                Interaction::Rest
            },
            self.error,
        );
        // Material Web はフォーカスの色を hover の色より優先する
        let hover =
            (!self.disabled && !self.focused).then(|| roles(Interaction::Hover, self.error));
        (rest, hover)
    }

    /// ラベルを入力欄の上端に小さく表示するか
    pub fn label_floating(&self) -> bool {
        self.populated || self.focused
    }

    /// 文・接辞・プレースホルダーを表示するか。ラベルを入力欄の中に大きく表示している間は表示しない
    pub fn input_visible(&self) -> bool {
        self.label.is_none() || self.label_floating()
    }

    pub fn input_style(&self) -> TextStyleRefinement {
        self.theme.text_style(filled::INPUT_FONT)
    }

    #[cfg(feature = "text-field")]
    pub fn input_colors(&self) -> InputColors {
        let (rest, _) = self.roles();
        let color = |(role, opacity): (ColorRole, f32)| self.theme.color(role).opacity(opacity);
        InputColors {
            placeholder: color(rest.placeholder),
            caret: self.theme.color(rest.caret),
            selection: self
                .theme
                .color(rest.caret)
                .opacity(supplement::SELECTION_OPACITY),
        }
    }

    /// `container` を入力欄の容器にし、補足の文と合わせた要素を返す
    ///
    /// `input` は文を置く欄に置く。[`Field::input_visible`] が偽の間は、大きく表示したラベルの裏に大きさ 0 で置く。
    /// 入力欄は高さ 56dp 以上、幅 280dp 以上で、返した要素の幅に広がる
    pub fn render<C: ParentElement + Styled + IntoElement>(
        self,
        mut container: C,
        input: Option<AnyElement>,
        window: &mut Window,
    ) -> Div {
        let theme = &self.theme;
        let (rest, hover) = self.roles();
        let color = |(role, opacity): (ColorRole, f32)| theme.color(role).opacity(opacity);
        let group = self.group;
        let floating = self.label_floating();
        let input_visible = self.input_visible();

        let horizontal_padding = dp(supplement::HORIZONTAL_PADDING);
        let icon_box = dp(supplement::ICON_BOX_SIZE);
        // アイコンの箱は 48dp で、アイコンの左右に空く分だけ文の余白を減らす (`textFieldHorizontalIconPadding`)
        let icon_padding = (icon_box - dp(filled::LEADING_ICON_SIZE)) / 2.0;
        let start_padding = if self.leading_icon.is_some() {
            (horizontal_padding - icon_padding).max(px(0.))
        } else {
            horizontal_padding
        };
        let end_padding = if self.trailing_icon.is_some() {
            (horizontal_padding - icon_padding).max(px(0.))
        } else {
            horizontal_padding
        };
        let input_style = self.input_style();
        let small_label_style = theme.text_style(TypeScale::BodySmall);
        let label_style = if floating {
            small_label_style.clone()
        } else {
            theme.text_style(filled::LABEL_FONT)
        };
        let mut small_label = window.text_style();
        small_label.refine(&small_label_style);
        let small_label_height = small_label.line_height_in_pixels(window.rem_size());
        let mut input_text = window.text_style();
        input_text.refine(&input_style);
        let input_line_height = input_text.line_height_in_pixels(window.rem_size());

        let label_element = |label: SharedString| {
            let mut element = div()
                .debug_selector(|| "md3-field-label".into())
                .flex_none()
                .whitespace_nowrap()
                .overflow_hidden()
                .text_ellipsis()
                .text_color(color(rest.label))
                .when_some(hover, |this, hover| {
                    this.group_hover(group, |style| style.text_color(color(hover.label)))
                })
                .child(label);
            element.text_style().refine(&label_style);
            element
        };
        let affix = |text: SharedString| {
            div()
                .flex_none()
                .whitespace_nowrap()
                .text_color(color(rest.affix))
                .child(text)
        };
        // ラベルを浮かせている間は、値が無くても文の 1 行分の高さを取る (`MinTextLineHeight`)
        let input_row = if input_visible {
            div()
                .debug_selector(|| "md3-field-input".into())
                .min_h(input_line_height)
                .flex()
                .items_center()
                .whitespace_nowrap()
                .overflow_hidden()
                .text_ellipsis()
                .text_color(color(rest.input))
                .when_some(hover, |this, hover| {
                    this.group_hover(group, |style| style.text_color(color(hover.input)))
                })
                .children(self.prefix.map(|prefix| {
                    affix(prefix)
                        .debug_selector(|| "md3-field-prefix".into())
                        .pr(dp(supplement::PREFIX_SUFFIX_PADDING))
                }))
                .child(
                    div()
                        .debug_selector(|| "md3-field-text".into())
                        .flex_grow_1()
                        .min_w_0()
                        .overflow_hidden()
                        .children(input),
                )
                .children(self.suffix.map(|suffix| {
                    affix(suffix)
                        .debug_selector(|| "md3-field-suffix".into())
                        .pl(dp(supplement::PREFIX_SUFFIX_PADDING))
                }))
        } else {
            div().absolute().size_0().overflow_hidden().children(input)
        };
        // Filled はラベルを浮かせると上端から 8dp に置き、その下に文を置く。Outlined は浮かせたラベルを枠線の上に置く
        let in_field_label = self
            .label
            .clone()
            .filter(|_| self.variant == FieldVariant::Filled || !floating);
        let middle = div()
            .flex_grow_1()
            .min_w_0()
            .flex()
            .flex_col()
            .justify_center()
            .pl(start_padding)
            .pr(end_padding)
            .when(
                self.variant == FieldVariant::Filled && self.label.is_some() && floating,
                |this| {
                    this.justify_start()
                        .py(dp(supplement::FILLED_LABEL_VERTICAL_PADDING))
                },
            )
            .children(in_field_label.map(label_element))
            .child(input_row);

        let icon =
            |icon: StyledIcon, (rest_color, hover_color): (Hsla, Option<Hsla>), flip: bool| {
                div()
                    .flex_none()
                    .size(icon_box)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        svg()
                            .data(icon.svg().as_bytes())
                            .size(dp(filled::LEADING_ICON_SIZE))
                            .text_color(rest_color)
                            .when_some(hover_color, |this, hover_color| {
                                this.group_hover(group, |style| style.text_color(hover_color))
                            })
                            .when(flip, |this| {
                                this.with_transformation(Transformation::rotate(Radians(
                                    std::f32::consts::PI,
                                )))
                            }),
                    )
            };
        let leading = self.leading_icon.map(|leading| {
            icon(
                leading,
                (color(rest.leading), hover.map(|hover| color(hover.leading))),
                false,
            )
        });
        let trailing = self.trailing_icon.map(|trailing| {
            icon(
                trailing,
                (
                    color(rest.trailing),
                    hover.map(|hover| color(hover.trailing)),
                ),
                self.flip_trailing_icon,
            )
        });

        // hover の線の太さはトークンで通常と同じため、hover では色だけを変える
        let line_color = color(rest.line);
        let line_hover = hover.map(|hover| color(hover.line));
        let line_width = dp(rest.line_width);
        let field_height = dp(supplement::MIN_HEIGHT);
        let decoration: Vec<AnyElement> = match self.variant {
            FieldVariant::Filled => {
                let container = div().absolute().inset_0().bg(if self.disabled {
                    theme
                        .color(filled::DISABLED_CONTAINER_COLOR)
                        .opacity(filled::DISABLED_CONTAINER_OPACITY)
                } else {
                    theme.color(filled::CONTAINER_COLOR)
                });
                let indicator = div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .bottom_0()
                    .h(line_width)
                    .bg(line_color)
                    .when_some(line_hover, |this, line_hover| {
                        this.group_hover(group, |style| style.bg(line_hover))
                    });
                vec![
                    Shaped::new(container, theme.clone(), filled::CONTAINER_SHAPE)
                        .into_any_element(),
                    indicator.into_any_element(),
                ]
            }
            FieldVariant::Outlined => {
                // 浮かせたラベルの左右 4dp までの範囲には枠線を描かない (`outlineCutout`)。
                // 上辺を切るため、枠線を上の 2 つの角、上辺の線、残りの 3 辺に分けて描く。
                // 入力欄の高さは決まっているため、角の半径はその高さから求める
                let radii =
                    theme.corner_radii(outlined::CONTAINER_SHAPE, size(field_height, field_height));
                let border = |piece: Div| {
                    piece.absolute().border_color(line_color).when_some(
                        line_hover,
                        |this, line_hover| {
                            this.group_hover(group, |style| style.border_color(line_hover))
                        },
                    )
                };
                let segment = || {
                    div()
                        .h(line_width)
                        .bg(line_color)
                        .when_some(line_hover, |this, line_hover| {
                            this.group_hover(group, |style| style.bg(line_hover))
                        })
                };
                let top = radii.top_left.max(radii.top_right);
                let mut pieces = vec![
                    border(div())
                        .top(top)
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .border_l(line_width)
                        .border_r(line_width)
                        .border_b(line_width)
                        .rounded_bl(radii.bottom_left)
                        .rounded_br(radii.bottom_right)
                        .into_any_element(),
                    border(div())
                        .top_0()
                        .left_0()
                        .w(radii.top_left.max(line_width))
                        .h(top)
                        .border_t(line_width)
                        .border_l(line_width)
                        .rounded_tl(radii.top_left)
                        .into_any_element(),
                    border(div())
                        .top_0()
                        .right_0()
                        .w(radii.top_right.max(line_width))
                        .h(top)
                        .border_t(line_width)
                        .border_r(line_width)
                        .rounded_tr(radii.top_right)
                        .into_any_element(),
                ];
                let left = radii.top_left.max(line_width);
                let right = radii.top_right.max(line_width);
                match self.label.clone().filter(|_| floating) {
                    Some(label) => {
                        let font_size = small_label.font_size.to_pixels(window.rem_size());
                        let label_width = window
                            .text_system()
                            .shape_line(
                                label.clone(),
                                font_size,
                                &[small_label.to_run(label.len())],
                                None,
                            )
                            .width
                            .ceil();
                        let cutout = dp(supplement::OUTLINED_LABEL_CUTOUT_PADDING);
                        let cutout_left = (horizontal_padding - cutout).max(left);
                        // ラベルは左右の余白の内側に収め、収まらない分は省く。空ける範囲も同じだけ縮め、
                        // 右の線は空ける範囲の右端から角までの幅を下回らないようにする
                        pieces.push(
                            div()
                                .absolute()
                                .top_0()
                                .left_0()
                                .right_0()
                                .flex()
                                .pl(left)
                                .pr(right)
                                .child(segment().flex_none().w(cutout_left - left))
                                .child(
                                    div()
                                        .w(horizontal_padding - cutout_left + label_width + cutout),
                                )
                                .child(
                                    segment()
                                        .flex_grow_1()
                                        .min_w((horizontal_padding - cutout - right).max(px(0.))),
                                )
                                .into_any_element(),
                        );
                        // ラベルは枠線の上辺に中心をそろえて置く
                        let mut floating_label = div()
                            .debug_selector(|| "md3-field-label".into())
                            .min_w_0()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .text_color(color(rest.label))
                            .when_some(hover, |this, hover| {
                                this.group_hover(group, |style| {
                                    style.text_color(color(hover.label))
                                })
                            })
                            .child(label);
                        floating_label.text_style().refine(&small_label_style);
                        pieces.push(
                            div()
                                .absolute()
                                .left(horizontal_padding)
                                .right(horizontal_padding)
                                .top(-small_label_height / 2.0)
                                .flex()
                                .child(floating_label)
                                .into_any_element(),
                        );
                    }
                    None => pieces.push(
                        segment()
                            .absolute()
                            .top_0()
                            .left(left)
                            .right(right)
                            .into_any_element(),
                    ),
                }
                pieces
            }
        };

        container = container
            .relative()
            .flex()
            .items_center()
            .min_h(field_height)
            .children(decoration)
            .children(leading)
            .child(middle)
            .children(trailing);
        container.text_style().refine(&input_style);

        let supporting = self.supporting_text.map(|text| {
            let mut supporting = div()
                .debug_selector(|| "md3-field-supporting-text".into())
                .pt(dp(supplement::SUPPORTING_TOP_PADDING))
                .px(horizontal_padding)
                .text_color(color(rest.supporting))
                .when_some(hover, |this, hover| {
                    this.group_hover(group, |style| style.text_color(color(hover.supporting)))
                })
                .child(text);
            supporting
                .text_style()
                .refine(&theme.text_style(filled::SUPPORTING_FONT));
            supporting
        });

        div()
            .flex()
            .flex_col()
            .min_w(dp(supplement::MIN_WIDTH))
            // Outlined は浮かせたラベルが入力欄の上にはみ出す分だけ、上に余白を取る (`topPaddingForLabelCutout`)
            .when(
                self.variant == FieldVariant::Outlined && self.label.is_some(),
                |this| this.pt(small_label_height / 2.0),
            )
            .child(container)
            .children(supporting)
    }
}
