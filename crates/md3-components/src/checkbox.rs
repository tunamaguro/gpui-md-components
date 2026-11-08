// 寸法と色の選択は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Checkbox.kt
//         (`CheckboxImpl`、`drawBox`、`CheckboxDefaults`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// 状態レイヤーの色とフォーカスリングの大きさは Compose にないため Material Web から移植している
// (https://github.com/material-components/material-web)
// Source: checkbox/internal/_checkbox.scss、tokens/versions/v0_192/_md-comp-checkbox.scss
//         @ de5470f7ed6a653457ca8f5cc0e7f7602c7a8391
// _checkbox.scss: Copyright 2022 Google LLC. _md-comp-checkbox.scss: Copyright 2023 Google LLC.
// Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 のチェックボックス

use gpui::{
    App, ElementId, FocusHandle, Hsla, InteractiveElement, IntoElement, ParentElement, Refineable,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, svg,
};
use gpui_headless::choice::{Checkbox as HeadlessCheckbox, CheckedState};

use md3_icons::IconStyle;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::checkbox;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::md3_tokens::{Dp, focus_ring, state};
use md3_theme::{SharedTheme, ThemeExt, dp};

/// `RadiusSize`。`CheckboxTokens.ContainerShape` と同じ 2dp
const CORNER_RADIUS: Dp = Dp(2.0);

/// 状態レイヤーがチェックボックスの hover を知るための group 名
const GROUP: &str = "md3-checkbox";

/// チェックボックスの状態
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum CheckboxState {
    /// チェックされていない
    #[default]
    Unchecked,
    /// チェックされている
    Checked,
    /// 一部だけチェックされている
    ///
    /// まとめて操作するチェックボックスで、配下の一部だけがチェックされていることを表す
    Indeterminate,
}

impl From<CheckedState> for CheckboxState {
    fn from(value: CheckedState) -> Self {
        match value {
            CheckedState::Unchecked => CheckboxState::Unchecked,
            CheckedState::Checked => CheckboxState::Checked,
            CheckedState::Mixed => CheckboxState::Indeterminate,
        }
    }
}

impl From<CheckboxState> for CheckedState {
    fn from(value: CheckboxState) -> Self {
        match value {
            CheckboxState::Unchecked => CheckedState::Unchecked,
            CheckboxState::Checked => CheckedState::Checked,
            CheckboxState::Indeterminate => CheckedState::Mixed,
        }
    }
}

type ClickHandler = Box<dyn Fn(&CheckboxState, &mut Window, &mut App)>;

/// MD3 のチェックボックス
///
/// 状態は呼び出し側が持ち、[`Checkbox::new`] に毎回渡す。部品自身は状態を変えない。
/// 押されると [`Checkbox::on_click`] のハンドラに押した後の状態を渡すため、ハンドラで呼び出し側の状態を更新すること
///
/// 支援技術には role `checkbox` として公開され、[`Checkbox::label`] か [`Checkbox::aria_label`] が
/// アクセシブルな名前になる。どちらかを必ず与えること
///
/// 大きさは状態レイヤーの 40dp 四方の箱で、ラベルを付けるとその右に並ぶ。押せる範囲は箱とラベルの全体。
/// フォーカスリングは箱の外側にはみ出して描く
///
/// [`Styled`] で指定したスタイルはチェックボックスの外側の箱に付く
#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    state: CheckboxState,
    label: Option<SharedString>,
    on_click: Option<ClickHandler>,
    aria_label: Option<SharedString>,
    disabled: bool,
    error: Option<SharedString>,
    focus_handle: Option<FocusHandle>,
    theme: Option<SharedTheme>,
    outer: gpui::Div,
}

impl Checkbox {
    /// `state` を表示するチェックボックスを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。[`Checkbox::track_focus`] を指定しない場合、
    /// `id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(id: impl Into<ElementId>, state: CheckboxState) -> Self {
        Self {
            id: id.into(),
            state,
            label: None,
            on_click: None,
            aria_label: None,
            disabled: false,
            error: None,
            focus_handle: None,
            theme: None,
            // 利用者が Styled で指定した display を render で上書きしないよう、既定はここで付ける
            outer: div().flex(),
        }
    }

    /// チェックボックスの右に `label` を表示し、アクセシブルな名前にする
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// ラベルの代わりに `label` をアクセシブルな名前にする
    ///
    /// ラベルを表示しない場合に使う
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// 押されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には押した後に求められる状態を渡す。[`CheckboxState::Checked`] なら
    /// [`CheckboxState::Unchecked`] を、それ以外なら [`CheckboxState::Checked`] を渡す。
    /// 設定しない場合、チェックボックスは押されても何も呼ばない
    pub fn on_click(
        mut self,
        handler: impl Fn(&CheckboxState, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// チェックボックスを無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効なチェックボックスは無効の色で描かれ、押されてもハンドラを呼ばず、Tab でも到達できない。
    /// エラーの色より無効の色を優先する。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 値にエラーがあることを示す。`None` を渡すとエラーのない状態に戻す。既定ではエラーなし
    ///
    /// `Some` の場合、エラーの色で描かれ、支援技術には invalid として公開され、`message` が説明の先頭に加わる。
    /// `message` は表示しないため、エラーの内容は呼び出し側がテキストとしても表示すること
    pub fn error(mut self, message: Option<SharedString>) -> Self {
        self.error = message;
        self
    }

    /// `focus_handle` をこのチェックボックスのフォーカスとして使う
    ///
    /// 指定しない場合はチェックボックスが内部でフォーカスを持つ
    pub fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle.clone());
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl RenderOnce for Checkbox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let disabled = self.disabled;
        let error = self.error.is_some();
        let selected = self.state != CheckboxState::Unchecked;

        let focus_handle = self.focus_handle.unwrap_or_else(|| {
            window
                .use_keyed_state(self.id.clone(), cx, |_, cx| cx.focus_handle())
                .read(cx)
                .clone()
        });
        let focus_visible = gpui_headless::is_focus_visible(&focus_handle, window, cx);

        // `defaultCheckboxColors` と `isCheckboxStylingFixEnabled` が true のときの色の選択。
        // false は MD2 の見た目のため採らない。エラーの色は CheckboxTokens にあるが Compose は使っていない。
        // 選択時は箱と枠線が同じ色のため、drawBox と同じく塗りだけで描き、枠線は付けない
        let (fill, outline, checkmark): (Option<Hsla>, Option<Hsla>, Hsla) =
            match (selected, disabled) {
                (true, false) => {
                    let (container, icon) = if error {
                        (
                            checkbox::SELECTED_ERROR_CONTAINER_COLOR,
                            checkbox::SELECTED_ERROR_ICON_COLOR,
                        )
                    } else {
                        (
                            checkbox::SELECTED_CONTAINER_COLOR,
                            checkbox::SELECTED_ICON_COLOR,
                        )
                    };
                    (Some(theme.color(container)), None, theme.color(icon))
                }
                (true, true) => (
                    Some(
                        theme
                            .color(checkbox::SELECTED_DISABLED_CONTAINER_COLOR)
                            .opacity(checkbox::SELECTED_DISABLED_CONTAINER_OPACITY),
                    ),
                    None,
                    theme.color(checkbox::SELECTED_DISABLED_ICON_COLOR),
                ),
                (false, false) => {
                    let role = if error {
                        checkbox::UNSELECTED_ERROR_OUTLINE_COLOR
                    } else if focus_visible {
                        checkbox::UNSELECTED_FOCUS_OUTLINE_COLOR
                    } else {
                        checkbox::UNSELECTED_OUTLINE_COLOR
                    };
                    (None, Some(theme.color(role)), gpui::transparent_black())
                }
                (false, true) => (
                    None,
                    Some(
                        theme
                            .color(checkbox::UNSELECTED_DISABLED_OUTLINE_COLOR)
                            .opacity(checkbox::UNSELECTED_DISABLED_CONTAINER_OPACITY),
                    ),
                    gpui::transparent_black(),
                ),
            };
        let hover_outline = theme.color(if error {
            checkbox::UNSELECTED_ERROR_HOVER_OUTLINE_COLOR
        } else {
            checkbox::UNSELECTED_HOVER_OUTLINE_COLOR
        });
        // Material Web の `*-state-layer-color`。Compose は状態レイヤーの色をトークンに持たない
        let state_layer = theme.color(if error {
            ColorRole::Error
        } else if selected {
            ColorRole::Primary
        } else {
            ColorRole::OnSurface
        });

        let container = div()
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .bottom_0()
            .rounded(dp(CORNER_RADIUS))
            .when_some(fill, |this, fill| this.bg(fill))
            .when_some(outline, |this, outline| {
                this.border(dp(checkbox::UNSELECTED_OUTLINE_WIDTH))
                    .border_color(outline)
                    .when(!disabled, |this| {
                        this.group_hover(GROUP, |style| style.border_color(hover_outline))
                    })
            });

        // Compose は線を描いてチェックマークを作るが、同じ形のアイコンで代える
        let icon = match self.state {
            CheckboxState::Unchecked => None,
            CheckboxState::Checked => Some(md3_icons::CHECK),
            CheckboxState::Indeterminate => Some(md3_icons::CHECK_INDETERMINATE_SMALL),
        };

        let state_size = dp(checkbox::STATE_LAYER_SIZE);
        let state_layer = div()
            .absolute()
            .top_0()
            .left_0()
            .size(state_size)
            .rounded_full()
            .when(!disabled, |this| {
                this.when(focus_visible, |this| {
                    this.bg(state_layer.opacity(state::FOCUS_STATE_LAYER_OPACITY))
                })
                .group_hover(GROUP, |style| {
                    style.bg(state_layer.opacity(state::HOVER_STATE_LAYER_OPACITY))
                })
            });

        let indicator = div()
            .relative()
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .size(state_size)
            .child(state_layer)
            .child(
                div()
                    .relative()
                    .size(dp(checkbox::CONTAINER_SIZE))
                    .child(container)
                    .when_some(icon, |this, icon| {
                        this.child(
                            svg()
                                .data(icon.svg(IconStyle::Outlined, false).as_bytes())
                                .absolute()
                                .top_0()
                                .left_0()
                                .size(dp(checkbox::ICON_SIZE))
                                .text_color(checkmark),
                        )
                    }),
            )
            .when(focus_visible, |this| {
                // Material Web は 44px の円 (状態レイヤーから外側へ 2px) の外側に、太さ 3px のリングを描く
                let outset = dp(focus_ring::OUTWARD_OFFSET) + dp(focus_ring::WIDTH);
                this.child(
                    div()
                        .absolute()
                        .top(-outset)
                        .left(-outset)
                        .size(state_size + outset * 2.0)
                        .rounded_full()
                        .border(dp(focus_ring::WIDTH))
                        .border_color(theme.color(focus_ring::COLOR)),
                )
            });

        let label_color = if disabled {
            theme
                .color(ColorRole::OnSurface)
                .opacity(checkbox::UNSELECTED_DISABLED_CONTAINER_OPACITY)
        } else {
            theme.color(ColorRole::OnSurface)
        };

        let on_click = self.on_click;
        let mut root = HeadlessCheckbox::new(self.id, self.state.into())
            .track_focus(&focus_handle)
            .disabled(disabled)
            .error(self.error)
            .group(GROUP)
            .flex()
            .items_center()
            .text_color(label_color)
            .child(indicator)
            .when_some(self.label, |this, label| this.child(label))
            .when_some(self.aria_label, |this, label| this.aria_label(label))
            .when_some(on_click, |this, on_click| {
                this.on_change(move |next, window, cx| on_click(&next.into(), window, cx))
            });
        root.text_style()
            .refine(&theme.text_style(TypeScale::BodyLarge));

        self.outer.child(root)
    }
}

impl Styled for Checkbox {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.outer.style()
    }
}
