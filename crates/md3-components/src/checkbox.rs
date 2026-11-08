// 寸法と色の選択は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Checkbox.kt
//         (`CheckboxImpl`、`drawBox`、`CheckboxDefaults`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// フォーカスリングの大きさは Compose にないため Material Web から移植している
// (https://github.com/material-components/material-web)
// Source: checkbox/internal/_checkbox.scss @ de5470f7ed6a653457ca8f5cc0e7f7602c7a8391
// Copyright 2022 Google LLC. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 のチェックボックス

use gpui::{
    App, CornersRefinement, ElementId, FocusHandle, Hsla, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, size, svg,
};
use gpui_headless::choice::{Checkbox as HeadlessCheckbox, CheckedState};

use crate::{InteractionSource, util};
use md3_icons::IconStyle;
use md3_theme::md3_tokens::component::checkbox;
use md3_theme::md3_tokens::{focus_ring, state, supplement};
use md3_theme::{SharedTheme, Theme, ThemeExt, dp, shape_radii};

/// [`Checkbox::interaction_source`] を指定しないとき、状態レイヤーがチェックボックスの hover を知るための group 名
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
///
/// [`Checkbox::on_click`] を設定すると操作できるチェックボックスになる。押されるとハンドラに押した後の状態を渡すため、
/// ハンドラで呼び出し側の状態を更新すること。
/// 支援技術には role `checkbox` として公開され、[`Checkbox::aria_label`] がアクセシブルな名前になる。
/// ラベルは表示しないため、名前は必ず [`Checkbox::aria_label`] で与えること
///
/// 設定しない場合は見た目だけのチェックボックスになり、押せず、フォーカスを受けず、支援技術にも公開されない。
/// [`CheckboxRow`] の子にして、行全体を 1 つのチェックボックスとして操作させる場合に使う
///
/// 大きさは状態レイヤーの 40dp 四方の箱で、操作できる場合はこの箱が押せる範囲になる。
/// フォーカスリングは箱の外側にはみ出して描く
///
/// [`Styled`] で指定したスタイルはチェックボックスの外側の箱に付く
#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    state: CheckboxState,
    on_click: Option<ClickHandler>,
    aria_label: Option<SharedString>,
    disabled: bool,
    error: Option<SharedString>,
    focus_handle: Option<FocusHandle>,
    interaction_source: Option<InteractionSource>,
    theme: Option<SharedTheme>,
    outer: gpui::Div,
}

impl Checkbox {
    /// `state` を表示するチェックボックスを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。操作できるチェックボックスで
    /// [`Checkbox::track_focus`] も [`Checkbox::interaction_source`] も指定しない場合、
    /// `id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(id: impl Into<ElementId>, state: CheckboxState) -> Self {
        Self {
            id: id.into(),
            state,
            on_click: None,
            aria_label: None,
            disabled: false,
            error: None,
            focus_handle: None,
            interaction_source: None,
            theme: None,
            // 利用者が Styled で指定した display を render で上書きしないよう、既定はここで付ける
            outer: div().flex(),
        }
    }

    /// `label` をアクセシブルな名前にする。見た目だけのチェックボックスでは使わない
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// 押されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には押した後に求められる状態を渡す。[`CheckboxState::Checked`] なら
    /// [`CheckboxState::Unchecked`] を、それ以外なら [`CheckboxState::Checked`] を渡す。
    /// 設定しない場合は見た目だけのチェックボックスになる
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
    /// `Some` の場合、エラーの色で描かれる。操作できるチェックボックスでは、支援技術に invalid として公開され、
    /// `message` が説明の先頭に加わる。見た目だけのチェックボックスでは色だけが変わるため、
    /// 支援技術へは [`CheckboxRow::error`] で伝えること。
    /// `message` は表示しないため、エラーの内容は呼び出し側がテキストとしても表示すること
    pub fn error(mut self, message: Option<SharedString>) -> Self {
        self.error = message;
        self
    }

    /// `focus_handle` をこのチェックボックスのフォーカスとして使う
    ///
    /// 指定しない場合は、操作できるチェックボックスが内部でフォーカスを持つ。
    /// 見た目だけのチェックボックスと、[`Checkbox::interaction_source`] を指定した場合は使わない
    pub fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle.clone());
        self
    }

    /// 状態レイヤー・hover の枠線・フォーカスリングを、`source` を渡した部品の hover とフォーカスで描く
    ///
    /// 見た目だけのチェックボックスに、自身を子孫に含む [`CheckboxRow`] に渡したものと同じ `source` を渡すと、
    /// 行の hover とフォーカスがチェックボックスに表示される。
    /// 操作できるチェックボックスに渡すと、`source` のフォーカスをこのチェックボックスのフォーカスとして使う
    pub fn interaction_source(mut self, source: &InteractionSource) -> Self {
        self.interaction_source = Some(source.clone());
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
        let interactive = self.on_click.is_some();

        // hover を知る group と、フォーカスを表示するためのフォーカス。見た目だけで source もなければどちらも持たない
        let (group, focus_handle): (Option<SharedString>, Option<FocusHandle>) =
            match &self.interaction_source {
                Some(source) => (
                    Some(source.inner().group().clone()),
                    Some(source.focus_handle().clone()),
                ),
                None if interactive => (
                    Some(GROUP.into()),
                    Some(
                        self.focus_handle
                            .unwrap_or_else(|| util::keyed_focus_handle(&self.id, window, cx)),
                    ),
                ),
                None => (None, None),
            };
        let colors = Colors::resolve(
            &theme,
            self.state,
            disabled,
            self.error.is_some(),
            focus_handle
                .as_ref()
                .is_some_and(|focus_handle| focus_handle.is_focused(window)),
            focus_handle.as_ref().is_some_and(|focus_handle| {
                gpui_headless::is_focus_visible(focus_handle, window, cx)
            }),
        );

        let container_size = dp(checkbox::CONTAINER_SIZE);
        let mut container = div()
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .bottom_0()
            .when_some(colors.fill, |this, fill| this.bg(fill))
            .when_some(colors.outline, |this, outline| {
                this.border(dp(checkbox::UNSELECTED_OUTLINE_WIDTH))
                    .border_color(outline)
            })
            .when_some(
                group.clone().zip(colors.hover_outline),
                |this, (group, outline)| {
                    this.group_hover(group, |style| style.border_color(outline))
                },
            );
        let radii = shape_radii(
            checkbox::CONTAINER_SHAPE,
            size(container_size, container_size),
        );
        container.style().corner_radii = CornersRefinement {
            top_left: Some(radii.top_left.into()),
            top_right: Some(radii.top_right.into()),
            bottom_right: Some(radii.bottom_right.into()),
            bottom_left: Some(radii.bottom_left.into()),
        };

        // Compose は drawCheck で線を描くが、Material Symbols のアイコンで代える。形と線の太さは Compose と異なる
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
            .when_some(colors.focus_state_layer, |this, layer| this.bg(layer))
            .when_some(
                group.clone().zip(colors.hover_state_layer),
                |this, (group, layer)| this.group_hover(group, |style| style.bg(layer)),
            );

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
                    .size(container_size)
                    .child(container)
                    .when_some(icon, |this, icon| {
                        this.child(
                            svg()
                                .data(icon.svg(IconStyle::Outlined, false).as_bytes())
                                .absolute()
                                .top_0()
                                .left_0()
                                .size(dp(checkbox::ICON_SIZE))
                                .text_color(colors.checkmark),
                        )
                    }),
            )
            .when_some(colors.focus_ring, |this, ring| {
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
                        .border_color(ring),
                )
            });

        let Some(on_click) = self.on_click else {
            return self.outer.child(indicator);
        };
        let root = HeadlessCheckbox::new(self.id, self.state.into())
            .disabled(disabled)
            .error(self.error)
            .flex()
            .child(indicator)
            .map(|this| match (&self.interaction_source, focus_handle) {
                (Some(source), _) => this.interaction_source(source.inner()),
                (None, Some(focus_handle)) => this.track_focus(&focus_handle).group(GROUP),
                (None, None) => this,
            })
            .when_some(self.aria_label, |this, label| this.aria_label(label))
            .on_change(move |next, window, cx| on_click(&next.into(), window, cx));

        self.outer.child(root)
    }
}

/// 行全体を 1 つのチェックボックスとして操作させる部品
///
/// 子の内容を並べ、行のどこを押しても [`CheckboxRow::on_click`] のハンドラを呼ぶ。
/// 支援技術には role `checkbox` として公開され、子のうち支援技術に公開されるテキストがアクセシブルな名前になる。
/// 文字列を直接子にしても支援技術には公開されないため、名前にするテキストは [`gpui::text!`] で作ること。
/// 状態は呼び出し側が持ち、[`CheckboxRow::new`] に毎回渡す
///
/// 行自身は見た目を持たない。子に見た目だけの [`Checkbox`] を置き、行に渡したものと同じ [`InteractionSource`] を
/// [`Checkbox::interaction_source`] に渡すこと。行の hover とフォーカスがチェックボックスに表示される。
/// 無効とエラーは子のチェックボックスにも同じ値を渡すこと。
/// レイアウトは [`Styled`] で呼び出し側が付ける
///
/// ```ignore
/// CheckboxRow::new("row", state, &source)
///     .on_click(|next, _, _| { /* 状態を更新する */ })
///     .child(Checkbox::new("box", state).interaction_source(&source))
///     .child(gpui::text!("ラベル"))
/// ```
#[derive(IntoElement)]
pub struct CheckboxRow {
    inner: HeadlessCheckbox,
}

impl CheckboxRow {
    /// `state` を表す行を作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。
    /// 行は `source` のフォーカスを使い、`source` を通して行の hover とフォーカスを子に伝える
    pub fn new(id: impl Into<ElementId>, state: CheckboxState, source: &InteractionSource) -> Self {
        Self {
            inner: HeadlessCheckbox::new(id, state.into()).interaction_source(source.inner()),
        }
    }

    /// 押されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` に渡す状態は [`Checkbox::on_click`] と同じ。設定しない場合、行は押されても何も呼ばない
    pub fn on_click(
        mut self,
        handler: impl Fn(&CheckboxState, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.inner = self
            .inner
            .on_change(move |next, window, cx| handler(&next.into(), window, cx));
        self
    }

    /// 行を無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効な行は押されてもハンドラを呼ばず、Tab でも到達できない。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.inner = self.inner.disabled(disabled);
        self
    }

    /// 値にエラーがあることを示す。`None` を渡すとエラーのない状態に戻す。既定ではエラーなし
    ///
    /// `Some` の場合、支援技術には invalid として公開され、`message` が説明の先頭に加わる。
    /// `message` は表示しないため、エラーの内容は呼び出し側がテキストとしても表示すること
    pub fn error(mut self, message: Option<SharedString>) -> Self {
        self.inner = self.inner.error(message);
        self
    }
}

impl RenderOnce for CheckboxRow {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        self.inner
    }
}

impl Styled for CheckboxRow {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.inner.style()
    }
}

impl ParentElement for CheckboxRow {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.inner.extend(elements);
    }
}

/// 状態から決まる描画の色
struct Colors {
    /// 箱の塗り。`None` は塗らない
    fill: Option<Hsla>,
    /// 箱の枠線。`None` は枠線を付けない
    outline: Option<Hsla>,
    /// hover 中の枠線。`None` は hover で変えない
    hover_outline: Option<Hsla>,
    /// チェックマーク
    checkmark: Hsla,
    /// フォーカスの状態レイヤー。`None` は描かない
    focus_state_layer: Option<Hsla>,
    /// hover の状態レイヤー。`None` は描かない
    hover_state_layer: Option<Hsla>,
    /// フォーカスリング。`None` は描かない
    focus_ring: Option<Hsla>,
}

impl Colors {
    // `defaultCheckboxColors` と `isCheckboxStylingFixEnabled` が true のときの色の選択。
    // false は MD2 の見た目のため採らない。エラーの色は CheckboxTokens にあるが Compose は使っていない。
    // 選択時は箱と枠線が同じ色のため、drawBox と同じく塗りだけで描き、枠線は付けない
    fn resolve(
        theme: &Theme,
        state: CheckboxState,
        disabled: bool,
        error: bool,
        focused: bool,
        focus_visible: bool,
    ) -> Self {
        let selected = state != CheckboxState::Unchecked;
        let (fill, outline, checkmark) = match (selected, disabled) {
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
                } else if focused {
                    // Material Web はリングの有無にかかわらず、フォーカスがあればこの色にする (`:focus-within`)
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
        let hover_outline = (!selected && !disabled).then(|| {
            theme.color(if error {
                checkbox::UNSELECTED_ERROR_HOVER_OUTLINE_COLOR
            } else {
                checkbox::UNSELECTED_HOVER_OUTLINE_COLOR
            })
        });
        let (hover_layer, focus_layer) = if error {
            (
                supplement::checkbox::ERROR_HOVER_STATE_LAYER_COLOR,
                supplement::checkbox::ERROR_FOCUS_STATE_LAYER_COLOR,
            )
        } else if selected {
            (
                supplement::checkbox::SELECTED_HOVER_STATE_LAYER_COLOR,
                supplement::checkbox::SELECTED_FOCUS_STATE_LAYER_COLOR,
            )
        } else {
            (
                supplement::checkbox::UNSELECTED_HOVER_STATE_LAYER_COLOR,
                supplement::checkbox::UNSELECTED_FOCUS_STATE_LAYER_COLOR,
            )
        };
        Self {
            fill,
            outline,
            hover_outline,
            checkmark,
            focus_state_layer: (!disabled && focus_visible).then(|| {
                theme
                    .color(focus_layer)
                    .opacity(state::FOCUS_STATE_LAYER_OPACITY)
            }),
            hover_state_layer: (!disabled).then(|| {
                theme
                    .color(hover_layer)
                    .opacity(state::HOVER_STATE_LAYER_OPACITY)
            }),
            focus_ring: focus_visible.then(|| theme.color(focus_ring::COLOR)),
        }
    }
}

impl Styled for Checkbox {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.outer.style()
    }
}

/// 行全体を 1 つのチェックボックスとして操作させる構成 ([`CheckboxRow`] と見た目だけの [`Checkbox`]) の振る舞い
#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use gpui::{Context, Modifiers, Render, TestAppContext, VisualTestContext, point, px};

    use super::*;

    struct RowView {
        source: InteractionSource,
        changes: Rc<RefCell<Vec<CheckboxState>>>,
    }

    impl Render for RowView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let changes = self.changes.clone();
            // 箱は左端の 40px 四方、テキストはその右に並ぶ
            CheckboxRow::new("row", CheckboxState::Unchecked, &self.source)
                .on_click(move |state, _, _| changes.borrow_mut().push(*state))
                .w(px(200.))
                .child(
                    Checkbox::new("box", CheckboxState::Unchecked).interaction_source(&self.source),
                )
                .child(gpui::text!("Lettuce"))
        }
    }

    fn setup(
        cx: &mut TestAppContext,
    ) -> (
        Rc<RefCell<Vec<CheckboxState>>>,
        InteractionSource,
        &mut VisualTestContext,
    ) {
        cx.update(crate::init);
        let changes = Rc::<RefCell<Vec<CheckboxState>>>::default();
        let source = cx.update(InteractionSource::new);
        let (_, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            RowView {
                source: source.clone(),
                changes: changes.clone(),
            }
        });
        cx.update(|window, _| window.activate_window());
        (changes, source, cx)
    }

    #[gpui::test]
    fn テキストを押しても箱を押しても一度だけ切り替わる(
        cx: &mut TestAppContext,
    ) {
        let (changes, _, cx) = setup(cx);
        cx.simulate_click(point(px(100.), px(20.)), Modifiers::none());
        assert_eq!(
            *changes.borrow(),
            vec![CheckboxState::Checked],
            "テキスト側"
        );
        cx.simulate_click(point(px(20.), px(20.)), Modifiers::none());
        assert_eq!(
            *changes.borrow(),
            vec![CheckboxState::Checked, CheckboxState::Checked],
            "箱を押しても行のハンドラが一度だけ呼ばれること"
        );
    }

    #[gpui::test]
    fn tab_で止まるのは行だけ(cx: &mut TestAppContext) {
        let (_, source, cx) = setup(cx);
        cx.simulate_keystrokes("tab");
        cx.update(|window, _| assert!(source.focus_handle().is_focused(window)));
        // Tab で到達できるのが行だけなら、次の Tab でも行に戻る
        cx.simulate_keystrokes("tab");
        cx.update(|window, _| assert!(source.focus_handle().is_focused(window)));
    }

    struct DecorativeView;

    impl Render for DecorativeView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            Checkbox::new("box", CheckboxState::Checked)
        }
    }

    #[gpui::test]
    fn 見た目だけのチェックボックスはフォーカスを受けない(
        cx: &mut TestAppContext,
    ) {
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            DecorativeView
        });
        cx.update(|window, _| window.activate_window());
        cx.simulate_keystrokes("tab");
        cx.simulate_click(point(px(20.), px(20.)), Modifiers::none());
        cx.update(|window, cx| assert!(window.focused(cx).is_none()));
    }
}
