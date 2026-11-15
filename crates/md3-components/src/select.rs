// 入力欄の並べ方・余白・色・書体と、一覧の余白・色・書体・形は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ExposedDropdownMenu.kt
//         (`ExposedDropdownMenuBox`、`ExposedDropdownMenu`、`ExposedDropdownMenuDefaults.TrailingIcon`)、
//         TextField.kt、OutlinedTextField.kt (`outlineCutout`)、
//         internal/TextFieldImpl.kt (`CommonDecorationBox`、`TextFieldLayout`、`TextFieldMeasurePolicy`、
//         `CutoutTextFieldLayout`、`OutlinedTextFieldMeasurePolicy`)、
//         Menu.kt (`DropdownMenuContent`、`DropdownMenuItemContent`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021, 2023 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
//
// hover での入力欄の色と、選ばれている項目の容器の色、内側に描く項目のフォーカスリングは Compose が扱わないため Material Web から移植している
// (https://github.com/material-components/material-web)
// Source: field/internal/_filled-field.scss、field/internal/_outlined-field.scss、
//         select/internal/selectoption/select-option.ts、menu/internal/menuitem/menu-item.ts (`md-focus-ring` の `inward`)
//         @ de5470f7ed6a653457ca8f5cc0e7f7602c7a8391
// Copyright 2023 Google LLC. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 のセレクト

use gpui::{
    AnyElement, App, Div, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, Radians,
    Refineable, RenderOnce, SharedString, StatefulInteractiveElement, StyleRefinement, Styled,
    Transformation, Window, div, prelude::FluentBuilder, px, size, svg,
};
use gpui_headless::select::{
    Select as HeadlessSelect, SelectListbox, SelectOption as HeadlessSelectOption,
};
use md3_icons::StyledIcon;

use crate::shape::Shaped;
use crate::util;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::{
    filled_text_field as filled, list, menu, outlined_text_field as outlined,
};
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::md3_tokens::{Dp, focus_ring, state, supplement};
use md3_theme::{SharedTheme, ThemeExt, dp};

/// 入力欄の層が入力欄の hover を知るための group 名
const GROUP: &str = "md3-select";
/// 項目の層が項目の hover を知るための group 名
const OPTION_GROUP: &str = "md3-select-option";

type ChangeHandler<T> = Box<dyn Fn(&T, &mut Window, &mut App)>;
type ExpandedChangeHandler = Box<dyn Fn(&bool, &mut Window, &mut App)>;

/// セレクトの入力欄の種類
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SelectVariant {
    /// 容器を塗り、下端に線を引く
    #[default]
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
struct FieldRoles {
    line: (ColorRole, f32),
    line_width: Dp,
    label: (ColorRole, f32),
    input: (ColorRole, f32),
    leading: (ColorRole, f32),
    trailing: (ColorRole, f32),
    supporting: (ColorRole, f32),
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
                },
                (Interaction::Rest, false) => FieldRoles {
                    line: opaque(t::$line),
                    line_width: t::$width,
                    label: opaque(t::LABEL_COLOR),
                    input: opaque(t::INPUT_COLOR),
                    leading: opaque(t::LEADING_ICON_COLOR),
                    trailing: opaque(t::TRAILING_ICON_COLOR),
                    supporting: opaque(t::SUPPORTING_COLOR),
                },
                (Interaction::Hover, false) => FieldRoles {
                    line: opaque(t::$hover),
                    line_width: t::$hover_width,
                    label: opaque(t::HOVER_LABEL_COLOR),
                    input: opaque(t::HOVER_INPUT_COLOR),
                    leading: opaque(t::HOVER_LEADING_ICON_COLOR),
                    trailing: opaque(t::HOVER_TRAILING_ICON_COLOR),
                    supporting: opaque(t::HOVER_SUPPORTING_COLOR),
                },
                (Interaction::Focus, false) => FieldRoles {
                    line: opaque(t::$focus),
                    line_width: t::$focus_width,
                    label: opaque(t::FOCUS_LABEL_COLOR),
                    input: opaque(t::FOCUS_INPUT_COLOR),
                    leading: opaque(t::FOCUS_LEADING_ICON_COLOR),
                    trailing: opaque(t::FOCUS_TRAILING_ICON_COLOR),
                    supporting: opaque(t::FOCUS_SUPPORTING_COLOR),
                },
                (Interaction::Rest, true) => FieldRoles {
                    line: opaque(t::$error),
                    line_width: t::$width,
                    label: opaque(t::ERROR_LABEL_COLOR),
                    input: opaque(t::ERROR_INPUT_COLOR),
                    leading: opaque(t::ERROR_LEADING_ICON_COLOR),
                    trailing: opaque(t::ERROR_TRAILING_ICON_COLOR),
                    supporting: opaque(t::ERROR_SUPPORTING_COLOR),
                },
                (Interaction::Hover, true) => FieldRoles {
                    line: opaque(t::$error_hover),
                    line_width: t::$hover_width,
                    label: opaque(t::ERROR_HOVER_LABEL_COLOR),
                    input: opaque(t::ERROR_HOVER_INPUT_COLOR),
                    leading: opaque(t::ERROR_HOVER_LEADING_ICON_COLOR),
                    trailing: opaque(t::ERROR_HOVER_TRAILING_ICON_COLOR),
                    supporting: opaque(t::ERROR_HOVER_SUPPORTING_COLOR),
                },
                (Interaction::Focus, true) => FieldRoles {
                    line: opaque(t::$error_focus),
                    line_width: t::$focus_width,
                    label: opaque(t::ERROR_FOCUS_LABEL_COLOR),
                    input: opaque(t::ERROR_FOCUS_INPUT_COLOR),
                    leading: opaque(t::ERROR_FOCUS_LEADING_ICON_COLOR),
                    trailing: opaque(t::ERROR_FOCUS_TRAILING_ICON_COLOR),
                    supporting: opaque(t::ERROR_FOCUS_SUPPORTING_COLOR),
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

/// MD3 のセレクト
///
/// 読み取り専用の入力欄に選ばれている項目の名前を表示し、押すと項目の一覧を開く。
/// 選ばれている値と開いているかどうかは呼び出し側が持ち、[`Select::new`] に毎回渡す。
/// セレクト自身は状態を変えず、選択は [`Select::on_change`] で、開閉は [`Select::on_expanded_change`] で求める。
/// 項目は [`Select::option`] で加え、加えた順に一覧に並ぶ
///
/// 閉じている間
///
/// - クリック・Enter・Space・下矢印キーで開き、選ばれている項目 (無ければ最初の項目) にフォーカスを移す
/// - 上矢印キーと Home で開いて最初の項目に、End で開いて最後の項目にフォーカスを移す
///
/// 開いている間の一覧
///
/// - 上下の矢印キーで前後の項目へ移り、端では止まる。Home と End では最初と最後の項目へ、
///   PageUp と PageDown では 10 個前と後の項目へ移る。無効な項目は飛ばす
/// - 項目にマウスが乗ると、その項目にフォーカスを移す
/// - 項目のクリック・Enter・Space・Alt+上矢印キーで、その項目を選ぶよう求めて閉じ、フォーカスをセレクトへ戻す
/// - Tab と Shift+Tab で、フォーカスのある項目を選ぶよう求めて閉じ、セレクトの次と前の要素へフォーカスを移す
/// - Escape で選ばずに閉じ、フォーカスをセレクトへ戻す
/// - 一覧とセレクトの外でマウスのボタンを押すと閉じる
///
/// 入力欄は高さ 56dp、幅 280dp 以上で、[`Styled`] で指定した幅に広がる。
/// [`Select::label`] のラベルは、値が無く閉じていてフォーカスも無い間は入力欄の中に、それ以外は入力欄の上端に小さく表示する。
/// 一覧は入力欄の下に入力欄以上の幅で開き、ウィンドウの左右の端から 8dp、上下の端から 48dp の内側に収まらなければ上に移す。
/// ウィンドウより大きければ縦にスクロールする
///
/// 支援技術には role `combobox` として、開いているかと、選ばれている項目の名前を値として公開し、
/// 一覧は role `listbox`、項目は role `option` として公開する。
/// 名前はラベルか [`Select::aria_label`] で与えること。補足の文は説明として公開する
#[derive(IntoElement)]
pub struct Select<T: PartialEq + 'static> {
    id: ElementId,
    selected: Option<T>,
    expanded: bool,
    variant: SelectVariant,
    label: Option<SharedString>,
    aria_label: Option<SharedString>,
    leading_icon: Option<StyledIcon>,
    supporting_text: Option<SharedString>,
    error: bool,
    disabled: bool,
    on_change: Option<ChangeHandler<T>>,
    on_expanded_change: Option<ExpandedChangeHandler>,
    options: Vec<SelectOption<T>>,
    style: StyleRefinement,
    theme: Option<SharedTheme>,
}

impl<T: PartialEq + 'static> Select<T> {
    /// `selected` の値を持つ項目が選ばれ、`expanded` なら一覧を開いた状態で表示する
    /// [`SelectVariant::Filled`] のセレクトを作る。`selected` が `None` ならどの項目も選ばれていない
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>, selected: Option<T>, expanded: bool) -> Self {
        Self {
            id: id.into(),
            selected,
            expanded,
            variant: SelectVariant::default(),
            label: None,
            aria_label: None,
            leading_icon: None,
            supporting_text: None,
            error: false,
            disabled: false,
            on_change: None,
            on_expanded_change: None,
            options: Vec::new(),
            style: StyleRefinement::default(),
            theme: None,
        }
    }

    /// 入力欄の種類を `variant` にする
    pub fn variant(mut self, variant: SelectVariant) -> Self {
        self.variant = variant;
        self
    }

    /// 入力欄のラベルを `label` にする。ラベルはアクセシブルな名前にもなる
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// ラベルの代わりに `label` をアクセシブルな名前にする
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// 入力欄の先頭に `icon` を表示する
    pub fn leading_icon(mut self, icon: impl Into<StyledIcon>) -> Self {
        self.leading_icon = Some(icon.into());
        self
    }

    /// 入力欄の下に補足の文 `text` を表示する
    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting_text = Some(text.into());
        self
    }

    /// `error` なら、入力欄をエラーの色で表示する
    ///
    /// エラーであることは支援技術に公開しない。理由は [`Select::supporting_text`] で伝えること
    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    /// セレクトを無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効なセレクトは押されても開かず、Tab でも到達できず、支援技術には disabled として公開される
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 項目を選ぶよう求められたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には選ぶよう求められた項目の値を渡す。選ばれている項目と無効な項目では呼ばない。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`Select::new`] に渡すこと
    pub fn on_change(mut self, handler: impl Fn(&T, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// 開くか閉じるよう求められたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には、開くよう求められたら `true` を、閉じるよう求められたら `false` を渡す。
    /// 項目を選ぶよう求めるときは、[`Select::on_change`] の後に呼ぶ。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`Select::new`] に渡すこと。
    /// 設定しない場合、セレクトは押されても何も呼ばない
    pub fn on_expanded_change(
        mut self,
        handler: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_expanded_change = Some(Box::new(handler));
        self
    }

    /// 末尾に項目を加える
    pub fn option(mut self, option: SelectOption<T>) -> Self {
        self.options.push(option);
        self
    }

    /// 末尾に `options` の項目を順に加える
    pub fn options(mut self, options: impl IntoIterator<Item = SelectOption<T>>) -> Self {
        self.options.extend(options);
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl<T: PartialEq + 'static> RenderOnce for Select<T> {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let focus_handle = util::keyed_focus_handle(&self.id, window, cx);
        // 一覧を開いている間はフォーカスが一覧にあるが、Compose と同じく入力欄にフォーカスがあるものとして表示する
        let focused = focus_handle.is_focused(window) || self.expanded;
        let roles = match self.variant {
            SelectVariant::Filled => filled_roles,
            SelectVariant::Outlined => outlined_roles,
        };
        let rest = roles(
            if self.disabled {
                Interaction::Disabled
            } else if focused {
                Interaction::Focus
            } else {
                Interaction::Rest
            },
            self.error,
        );
        // Material Web はフォーカスの色を hover の色より優先する
        let hover = (!self.disabled && !focused).then(|| roles(Interaction::Hover, self.error));
        let color = |(role, opacity): (ColorRole, f32)| theme.color(role).opacity(opacity);

        let value = self
            .options
            .iter()
            .find(|option| Some(&option.value) == self.selected.as_ref())
            .map(|option| option.label.clone());
        let floating = value.is_some() || focused;
        let horizontal_padding = dp(supplement::select::FIELD_HORIZONTAL_PADDING);
        let icon_box = dp(supplement::select::ICON_BOX_SIZE);
        // アイコンの箱は 48dp で、アイコンの左右に空く分だけ文の余白を減らす (`textFieldHorizontalIconPadding`)
        let icon_padding = (icon_box - dp(filled::LEADING_ICON_SIZE)) / 2.0;
        let start_padding = if self.leading_icon.is_some() {
            (horizontal_padding - icon_padding).max(px(0.))
        } else {
            horizontal_padding
        };
        // 末尾には常に開閉を示すアイコンを置く
        let end_padding = (horizontal_padding - icon_padding).max(px(0.));
        let input_style = theme.text_style(filled::INPUT_FONT);
        let small_label_style = theme.text_style(TypeScale::BodySmall);
        let label_style = if floating {
            small_label_style.clone()
        } else {
            theme.text_style(filled::LABEL_FONT)
        };
        let mut small_label = window.text_style();
        small_label.refine(&small_label_style);
        let small_label_height = small_label.line_height_in_pixels(window.rem_size());
        let mut input = window.text_style();
        input.refine(&input_style);
        let input_line_height = input.line_height_in_pixels(window.rem_size());

        let label_element = |label: SharedString| {
            let mut element = div()
                .debug_selector(|| "md3-select-label".into())
                .flex_none()
                .whitespace_nowrap()
                .overflow_hidden()
                .text_ellipsis()
                .text_color(color(rest.label))
                .when_some(hover, |this, hover| {
                    this.group_hover(GROUP, |style| style.text_color(color(hover.label)))
                })
                .child(label);
            element.text_style().refine(&label_style);
            element
        };
        // ラベルを浮かせている間は、値が無くても値の 1 行分の高さを取る (`MinTextLineHeight`)
        let value_element = (value.is_some() || floating).then(|| {
            div()
                .debug_selector(|| "md3-select-value".into())
                .min_h(input_line_height)
                .whitespace_nowrap()
                .overflow_hidden()
                .text_ellipsis()
                .text_color(color(rest.input))
                .when_some(hover, |this, hover| {
                    this.group_hover(GROUP, |style| style.text_color(color(hover.input)))
                })
                .children(value.clone())
        });
        // Filled はラベルを浮かせると上端から 8dp に置き、その下に値を置く。Outlined は浮かせたラベルを枠線の上に置く
        let in_field_label = self
            .label
            .clone()
            .filter(|_| self.variant == SelectVariant::Filled || !floating);
        let middle = div()
            .flex_grow_1()
            .min_w_0()
            .flex()
            .flex_col()
            .justify_center()
            .pl(start_padding)
            .pr(end_padding)
            .when(
                self.variant == SelectVariant::Filled && self.label.is_some() && floating,
                |this| {
                    this.justify_start()
                        .py(dp(supplement::select::FILLED_LABEL_VERTICAL_PADDING))
                },
            )
            .children(in_field_label.map(label_element))
            .children(value_element);

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
                                this.group_hover(GROUP, |style| style.text_color(hover_color))
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
        // 開いている間は向きを反転する (`ExposedDropdownMenuDefaults.TrailingIcon`)
        let trailing = icon(
            md3_icons::ARROW_DROP_DOWN.into(),
            (
                color(rest.trailing),
                hover.map(|hover| color(hover.trailing)),
            ),
            self.expanded,
        );

        // hover の線の太さはトークンで通常と同じため、hover では色だけを変える
        let line_color = color(rest.line);
        let line_hover = hover.map(|hover| color(hover.line));
        let line_width = dp(rest.line_width);
        let field_height = dp(supplement::select::FIELD_MIN_HEIGHT);
        let decoration: Vec<AnyElement> = match self.variant {
            SelectVariant::Filled => {
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
                        this.group_hover(GROUP, |style| style.bg(line_hover))
                    });
                vec![
                    Shaped::new(container, theme.clone(), filled::CONTAINER_SHAPE)
                        .into_any_element(),
                    indicator.into_any_element(),
                ]
            }
            SelectVariant::Outlined => {
                // 浮かせたラベルの左右 4dp までの範囲には枠線を描かない (`outlineCutout`)。
                // 上辺を切るため、枠線を上の 2 つの角、上辺の線、残りの 3 辺に分けて描く。
                // 入力欄の高さは決まっているため、角の半径はその高さから求める
                let radii =
                    theme.corner_radii(outlined::CONTAINER_SHAPE, size(field_height, field_height));
                let border = |piece: Div| {
                    piece.absolute().border_color(line_color).when_some(
                        line_hover,
                        |this, line_hover| {
                            this.group_hover(GROUP, |style| style.border_color(line_hover))
                        },
                    )
                };
                let segment = || {
                    div()
                        .absolute()
                        .top_0()
                        .h(line_width)
                        .bg(line_color)
                        .when_some(line_hover, |this, line_hover| {
                            this.group_hover(GROUP, |style| style.bg(line_hover))
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
                        let mut label_style = window.text_style();
                        label_style.refine(&small_label_style);
                        let font_size = label_style.font_size.to_pixels(window.rem_size());
                        let label_width = window
                            .text_system()
                            .shape_line(
                                label.clone(),
                                font_size,
                                &[label_style.to_run(label.len())],
                                None,
                            )
                            .width
                            .ceil();
                        let cutout = dp(supplement::select::OUTLINED_LABEL_CUTOUT_PADDING);
                        let cutout_left = (horizontal_padding - cutout).max(left);
                        pieces.push(
                            segment()
                                .left(left)
                                .w(cutout_left - left)
                                .into_any_element(),
                        );
                        pieces.push(
                            segment()
                                .left(horizontal_padding + label_width + cutout)
                                .right(right)
                                .into_any_element(),
                        );
                        // ラベルは枠線の上辺に中心をそろえて置く
                        let mut floating_label = div()
                            .debug_selector(|| "md3-select-label".into())
                            .absolute()
                            .left(horizontal_padding)
                            .top(-small_label_height / 2.0)
                            .whitespace_nowrap()
                            .text_color(color(rest.label))
                            .when_some(hover, |this, hover| {
                                this.group_hover(GROUP, |style| {
                                    style.text_color(color(hover.label))
                                })
                            })
                            .child(label);
                        floating_label.text_style().refine(&small_label_style);
                        pieces.push(floating_label.into_any_element());
                    }
                    None => pieces.push(segment().left(left).right(right).into_any_element()),
                }
                pieces
            }
        };

        let name = self.label.clone().or(self.aria_label.clone());
        let mut listbox = SelectListbox::new().window_margin(size(
            dp(supplement::menu::HORIZONTAL_MARGIN),
            dp(supplement::menu::VERTICAL_MARGIN),
        ));
        if let Some(name) = &name {
            listbox = listbox.aria_label(name.clone());
        }
        if self.expanded {
            listbox = fill_listbox(
                listbox,
                &self.id,
                self.selected.as_ref(),
                std::mem::take(&mut self.options),
                &theme,
                window,
                cx,
            );
        }
        let mut field = HeadlessSelect::new(self.id.clone(), self.selected, self.expanded)
            .listbox(listbox)
            .track_focus(&focus_handle.tab_index(0))
            .disabled(self.disabled)
            .group(GROUP)
            .relative()
            .flex()
            .items_center()
            .min_h(field_height)
            .children(decoration)
            .children(leading)
            .child(middle)
            .child(trailing);
        field.text_style().refine(&input_style);
        if let Some(name) = name {
            field = field.aria_label(name);
        }
        if let Some(on_change) = self.on_change {
            field = field.on_change(on_change);
        }
        if let Some(on_expanded_change) = self.on_expanded_change {
            field = field.on_expanded_change(on_expanded_change);
        }
        if let Some(text) = &self.supporting_text {
            field = field.aria_description(text.clone());
        }
        let supporting = self.supporting_text.map(|text| {
            let mut supporting = div()
                .debug_selector(|| "md3-select-supporting-text".into())
                .pt(dp(supplement::select::SUPPORTING_TOP_PADDING))
                .px(horizontal_padding)
                .text_color(color(rest.supporting))
                .when_some(hover, |this, hover| {
                    this.group_hover(GROUP, |style| style.text_color(color(hover.supporting)))
                })
                .child(text);
            supporting
                .text_style()
                .refine(&theme.text_style(filled::SUPPORTING_FONT));
            supporting
        });

        let mut outer = div()
            .flex()
            .flex_col()
            .min_w(dp(supplement::select::FIELD_MIN_WIDTH))
            // Outlined は浮かせたラベルが入力欄の上にはみ出す分だけ、上に余白を取る (`topPaddingForLabelCutout`)
            .when(
                self.variant == SelectVariant::Outlined && self.label.is_some(),
                |this| this.pt(small_label_height / 2.0),
            )
            .child(field)
            .children(supporting);
        outer.style().refine(&self.style);
        outer
    }
}

impl<T: PartialEq + 'static> Styled for Select<T> {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

/// `id` のセレクトの開いている一覧 `listbox` に、容器を描き `options` を並べて返す
fn fill_listbox<T: PartialEq + 'static>(
    mut listbox: SelectListbox<T>,
    id: &ElementId,
    selected: Option<&T>,
    options: Vec<SelectOption<T>>,
    theme: &SharedTheme,
    window: &mut Window,
    cx: &mut App,
) -> SelectListbox<T> {
    let shadow = theme.shadow(menu::CONTAINER_ELEVATION);
    let container = div()
        .absolute()
        .inset_0()
        .bg(theme.color(menu::CONTAINER_COLOR))
        .shadow(shadow.clone());
    listbox = listbox
        .paint_margin(util::shadow_margin(&shadow))
        .relative()
        .flex()
        .flex_col()
        .py(dp(supplement::select::LISTBOX_VERTICAL_PADDING))
        // 一覧はセレクトの子として描かれ、入力欄の文字のスタイルを受け継ぐため、折り返しを戻す
        .whitespace_normal()
        .child(Shaped::new(container, theme.clone(), menu::CONTAINER_SHAPE));
    // Compose は一覧の項目の文を labelLarge で描く (`DropdownMenuItemContent`)
    listbox
        .text_style()
        .refine(&theme.text_style(TypeScale::LabelLarge));

    let mut selected_found = false;
    for option in options {
        // 同じ値の項目が複数あっても、支援技術に公開する状態と同じく最初の項目だけを選ばれた見た目にする
        let is_selected = !selected_found && selected == Some(&option.value);
        selected_found |= is_selected;
        let focus_handle = window.with_element_namespace(id.clone(), |window| {
            window.with_element_namespace("options", |window| {
                util::keyed_focus_handle(&option.id, window, cx)
            })
        });
        let focus_visible = gpui_headless::is_focus_visible(&focus_handle, window, cx);
        let disabled = option.disabled;
        let label_color = if disabled {
            theme
                .color(list::ITEM_DISABLED_LABEL_TEXT_COLOR)
                .opacity(list::ITEM_DISABLED_LABEL_TEXT_OPACITY)
        } else if is_selected {
            theme.color(menu::LIST_ITEM_SELECTED_LABEL_TEXT_COLOR)
        } else {
            theme.color(list::ITEM_LABEL_TEXT_COLOR)
        };
        let container = (is_selected && !disabled)
            .then(|| theme.color(menu::LIST_ITEM_SELECTED_CONTAINER_COLOR));
        // 状態レイヤーは文の色を重ねる (Compose の項目の ripple は内容の色を使う)
        let state_layer = |opacity: f32| -> Hsla {
            let layer = label_color.opacity(opacity);
            match container {
                Some(container) => container.blend(layer),
                None => layer,
            }
        };
        let layer = div()
            .absolute()
            .inset_0()
            .when_some(container, |this, container| this.bg(container))
            .when(!disabled, |this| {
                let this = this.when(focus_visible, |this| {
                    this.bg(state_layer(state::FOCUS_STATE_LAYER_OPACITY))
                });
                // gpui は group_hover を自身のスタイルの後に重ねるため、hover がフォーカスより優先される
                this.group_hover(OPTION_GROUP, |style| {
                    style.bg(state_layer(state::HOVER_STATE_LAYER_OPACITY))
                })
            });
        // Material Web の項目はリングを内側に描く
        let ring = focus_visible.then(|| {
            div()
                .absolute()
                .inset(dp(focus_ring::INWARD_OFFSET))
                .border(dp(focus_ring::WIDTH))
                .border_color(theme.color(focus_ring::COLOR))
        });
        listbox = listbox.option(
            HeadlessSelectOption::new(option.id, option.value, option.label.clone())
                .disabled(disabled)
                .track_focus(&focus_handle)
                .group(OPTION_GROUP)
                .relative()
                .flex()
                .items_center()
                .min_h(dp(supplement::select::OPTION_MIN_HEIGHT))
                .px(dp(supplement::select::OPTION_HORIZONTAL_PADDING))
                .text_color(label_color)
                .child(layer)
                .child(div().relative().child(option.label))
                .children(ring),
        );
    }
    listbox
}

/// [`Select`] の一覧の項目
///
/// [`Select::option`] に渡して使う。項目が表す値が、セレクトの選ばれている値と等しいときに選ばれた状態になる
pub struct SelectOption<T> {
    id: ElementId,
    value: T,
    label: SharedString,
    disabled: bool,
}

impl<T> SelectOption<T> {
    /// `value` を表し、`label` を表示する項目を作る
    ///
    /// `label` は項目の名前と、選ばれたときに入力欄に表示する値になる。
    /// `id` と `value` は一覧の中で一意にし、`id` は描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>, value: T, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            value,
            label: label.into(),
            disabled: false,
        }
    }

    /// 項目を無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効な項目は選べず、矢印キーでも到達できない
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Context, Modifiers, Render, TestAppContext, VisualTestContext, point, px};

    use super::*;

    const OPTIONS: [&str; 3] = ["Apple", "Banana", "Cherry"];

    /// (100, 100) に置いた幅 300 のセレクト。前に 10x10 のボタンを置く
    struct SelectView {
        selected: Option<usize>,
        expanded: bool,
        variant: SelectVariant,
        leading_icon: bool,
        error: bool,
        disabled: bool,
        /// 指定した幅。`None` なら幅を指定しない
        width: Option<f32>,
    }

    impl Render for SelectView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div().size_full().child(
                Select::new("select", self.selected, self.expanded)
                    .variant(self.variant)
                    .label("Fruit")
                    .when(self.leading_icon, |this| {
                        this.leading_icon(md3_icons::NUTRITION)
                    })
                    .error(self.error)
                    .disabled(self.disabled)
                    .on_change(cx.listener(|this, value: &usize, _, cx| {
                        this.selected = Some(*value);
                        cx.notify();
                    }))
                    .on_expanded_change(cx.listener(|this, expanded: &bool, _, cx| {
                        this.expanded = *expanded;
                        cx.notify();
                    }))
                    .options(
                        OPTIONS
                            .iter()
                            .enumerate()
                            .map(|(ix, name)| SelectOption::new(*name, ix, *name)),
                    )
                    .absolute()
                    .left(px(100.))
                    .top(px(100.))
                    .when_some(self.width, |this, width| this.w(px(width))),
            )
        }
    }

    fn add_view(
        cx: &mut TestAppContext,
        f: impl FnOnce(&mut SelectView),
    ) -> (gpui::Entity<SelectView>, &mut VisualTestContext) {
        cx.update(gpui_headless::init);
        let (view, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            let mut view = SelectView {
                selected: None,
                expanded: false,
                variant: SelectVariant::Filled,
                leading_icon: false,
                error: false,
                disabled: false,
                width: Some(300.),
            };
            f(&mut view);
            view
        });
        cx.update(|window, _| window.activate_window());
        cx.simulate_resize(size(px(800.), px(600.)));
        // ウィンドウをアクティブにすると最初のマウスの移動の後に位置が (0, 0) に戻るため、済むまで待つ
        cx.run_until_parked();
        (view, cx)
    }

    fn color(cx: &mut VisualTestContext, role: ColorRole) -> Hsla {
        cx.update(|_, cx| cx.md3_theme().color(role))
    }

    /// 背景を `color` で塗った矩形の x, y, 幅, 高さ
    fn filled(cx: &mut VisualTestContext, color: Hsla) -> Vec<(f32, f32, f32, f32)> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            window
                .painted_quads()
                .iter()
                .filter(|quad| quad.background == color.into())
                .map(|quad| {
                    let b = quad.bounds;
                    (
                        b.origin.x.0 / scale,
                        b.origin.y.0 / scale,
                        b.size.width.0 / scale,
                        b.size.height.0 / scale,
                    )
                })
                .collect()
        })
    }

    /// 矩形の x, y, 幅, 高さと、上・右・下・左の枠線の太さ
    type BorderedRect = ((f32, f32, f32, f32), [f32; 4]);

    /// 枠線を `color` で描いた矩形。
    /// gpui は枠線だけの矩形を空の内側を避けて分けて描くため、同じ矩形は 1 つにまとめる
    fn bordered(cx: &mut VisualTestContext, color: Hsla) -> Vec<BorderedRect> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            window
                .painted_quads()
                .iter()
                .filter(|quad| quad.border_color == color)
                .map(|quad| {
                    let b = quad.bounds;
                    let w = quad.border_widths;
                    (
                        (
                            b.origin.x.0 / scale,
                            b.origin.y.0 / scale,
                            b.size.width.0 / scale,
                            b.size.height.0 / scale,
                        ),
                        [w.top.0, w.right.0, w.bottom.0, w.left.0].map(|w| w / scale),
                    )
                })
                .fold(Vec::new(), |mut rects, rect| {
                    if !rects.contains(&rect) {
                        rects.push(rect);
                    }
                    rects
                })
        })
    }

    // gpui は debug_assertions のあるビルドでだけ debug_selector の位置を記録する
    #[cfg(debug_assertions)]
    fn bounds(cx: &mut VisualTestContext, selector: &'static str) -> Option<(f32, f32, f32, f32)> {
        cx.debug_bounds(selector).map(|b| {
            (
                b.origin.x.into(),
                b.origin.y.into(),
                b.size.width.into(),
                b.size.height.into(),
            )
        })
    }

    #[gpui::test]
    fn filled_の入力欄は高さ_56_で容器を塗り下端に線を引く(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_view(cx, |_| {});
        let container = color(cx, filled::CONTAINER_COLOR);
        assert_eq!(filled(cx, container), [(100., 100., 300., 56.)]);
        let line = color(cx, filled::ACTIVE_INDICATOR_COLOR);
        assert_eq!(filled(cx, line), [(100., 155., 300., 1.)]);
    }

    #[gpui::test]
    fn 幅は_280_を下回らない(cx: &mut TestAppContext) {
        let (_, cx) = add_view(cx, |view| view.width = None);
        let container = color(cx, filled::CONTAINER_COLOR);
        assert_eq!(filled(cx, container), [(100., 100., 280., 56.)]);
    }

    #[gpui::test]
    #[cfg(debug_assertions)]
    fn 値が無く閉じている間はラベルを入力欄の中央に置く(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_view(cx, |_| {});
        let (x, y, _, height) = bounds(cx, "md3-select-label").unwrap();
        // bodyLarge の行の高さは 24
        assert_eq!((x, y, height), (116., 116., 24.));
        assert_eq!(bounds(cx, "md3-select-value"), None);
    }

    #[gpui::test]
    #[cfg(debug_assertions)]
    fn 値があればラベルを上端から_8_に浮かせその下に値を置く(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_view(cx, |view| view.selected = Some(1));
        let (x, y, _, height) = bounds(cx, "md3-select-label").unwrap();
        // bodySmall の行の高さは 16
        assert_eq!((x, y, height), (116., 108., 16.));
        let (x, y, _, height) = bounds(cx, "md3-select-value").unwrap();
        assert_eq!((x, y, height), (116., 124., 24.));
    }

    #[gpui::test]
    #[cfg(debug_assertions)]
    fn 先頭のアイコンがあれば文を_48_の箱と_4_の余白の後に置く(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_view(cx, |view| {
            view.selected = Some(1);
            view.leading_icon = true;
        });
        let (x, ..) = bounds(cx, "md3-select-value").unwrap();
        assert_eq!(x, 152.);
    }

    #[gpui::test]
    #[cfg(debug_assertions)]
    fn 開くとラベルを浮かせ線を_primary_で太くし一覧を下に開く(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = add_view(cx, |_| {});
        cx.simulate_click(point(px(200.), px(128.)), Modifiers::none());
        assert!(view.read_with(cx, |view, _| view.expanded));
        let (_, y, _, height) = bounds(cx, "md3-select-label").unwrap();
        assert_eq!((y, height), (108., 16.));
        let primary = color(cx, filled::FOCUS_ACTIVE_INDICATOR_COLOR);
        assert!(filled(cx, primary).contains(&(100., 154., 300., 2.)));
        // 一覧の容器は入力欄の幅で、上下 8 の余白の中に高さ 48 の項目を 3 つ並べる
        let listbox = color(cx, menu::CONTAINER_COLOR);
        assert_eq!(filled(cx, listbox), [(100., 156., 300., 160.)]);
    }

    #[gpui::test]
    fn 選ばれている項目を_secondary_container_で塗る(cx: &mut TestAppContext) {
        let (_, cx) = add_view(cx, |view| {
            view.selected = Some(1);
            view.expanded = true;
        });
        cx.run_until_parked();
        let selected = color(cx, menu::LIST_ITEM_SELECTED_CONTAINER_COLOR);
        assert_eq!(filled(cx, selected), [(100., 212., 300., 48.)]);
    }

    #[gpui::test]
    fn 項目を選ぶと値を表示する(cx: &mut TestAppContext) {
        let (view, cx) = add_view(cx, |_| {});
        cx.simulate_click(point(px(200.), px(128.)), Modifiers::none());
        // 3 つ目の項目は y が 260 から 308 の範囲になる
        cx.simulate_click(point(px(200.), px(284.)), Modifiers::none());
        assert_eq!(
            view.read_with(cx, |view, _| (view.selected, view.expanded)),
            (Some(2), false)
        );
    }

    #[gpui::test]
    fn hover_で線を_on_surface_にする(cx: &mut TestAppContext) {
        let (_, cx) = add_view(cx, |_| {});
        cx.simulate_mouse_move(point(px(200.), px(128.)), None, Modifiers::none());
        let hover = color(cx, filled::HOVER_ACTIVE_INDICATOR_COLOR);
        assert_eq!(filled(cx, hover), [(100., 155., 300., 1.)]);
    }

    #[gpui::test]
    fn エラーでは線とラベルを_error_の色にする(cx: &mut TestAppContext) {
        let (_, cx) = add_view(cx, |view| view.error = true);
        let error = color(cx, filled::ERROR_ACTIVE_INDICATOR_COLOR);
        assert_eq!(filled(cx, error), [(100., 155., 300., 1.)]);
    }

    #[gpui::test]
    fn 無効なセレクトは薄く描き開かない(cx: &mut TestAppContext) {
        let (view, cx) = add_view(cx, |view| view.disabled = true);
        let container = cx.update(|_, cx| {
            cx.md3_theme()
                .color(filled::DISABLED_CONTAINER_COLOR)
                .opacity(filled::DISABLED_CONTAINER_OPACITY)
        });
        assert_eq!(filled(cx, container), [(100., 100., 300., 56.)]);
        cx.simulate_click(point(px(200.), px(128.)), Modifiers::none());
        assert!(!view.read_with(cx, |view, _| view.expanded));
    }

    #[gpui::test]
    fn outlined_は枠線を上の角と残りの_3_辺に分けて描く(cx: &mut TestAppContext) {
        let (_, cx) = add_view(cx, |view| {
            view.variant = SelectVariant::Outlined;
            view.selected = Some(0);
        });
        let outline = color(cx, outlined::OUTLINE_COLOR);
        let mut pieces = bordered(cx, outline);
        pieces.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        // 入力欄は浮かせたラベルの半分の高さ 8 だけ下に置く。角の半径は 4
        assert_eq!(
            pieces,
            [
                ((100., 108., 4., 4.), [1., 0., 0., 1.]),
                ((100., 112., 300., 52.), [0., 1., 1., 1.]),
                ((396., 108., 4., 4.), [1., 1., 0., 0.]),
            ]
        );
    }

    #[gpui::test]
    #[cfg(debug_assertions)]
    fn outlined_は浮かせたラベルを上辺に置きその左右_4_までの線を空ける(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_view(cx, |view| {
            view.variant = SelectVariant::Outlined;
            view.selected = Some(0);
        });
        let (label_x, label_y, label_width, label_height) = bounds(cx, "md3-select-label").unwrap();
        assert_eq!((label_x, label_y, label_height), (116., 100., 16.));
        let outline = color(cx, outlined::OUTLINE_COLOR);
        let mut segments = filled(cx, outline);
        segments.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let right = label_x + label_width.ceil() + 4.;
        assert_eq!(
            segments,
            [(104., 108., 8., 1.), (right, 108., 396. - right, 1.)]
        );
    }

    #[gpui::test]
    #[cfg(debug_assertions)]
    fn outlined_の値が無く閉じている間は上辺を空けない(cx: &mut TestAppContext) {
        let (_, cx) = add_view(cx, |view| view.variant = SelectVariant::Outlined);
        let outline = color(cx, outlined::OUTLINE_COLOR);
        assert_eq!(filled(cx, outline), [(104., 108., 292., 1.)]);
        let (_, y, _, height) = bounds(cx, "md3-select-label").unwrap();
        assert_eq!((y, height), (124., 24.));
    }
}
