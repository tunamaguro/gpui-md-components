// 寸法・余白・色は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SearchBar.kt
//         (`SearchBar`、`SearchBarDefaults.InputField`、`SearchBarDefaults.inputFieldColors`、
//         `SearchBarMinWidth`、`SearchBarMaxWidth`、`SearchBarIconOffsetX`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2022 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 の検索バー

use gpui::{
    AnyElement, App, CursorStyle, ElementId, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Refineable, RenderOnce, SharedString, StatefulInteractiveElement,
    StyleRefinement, Styled, Window, div, prelude::FluentBuilder, svg,
};
use gpui_headless::text_field::TextField as HeadlessTextField;
use md3_icons::StyledIcon;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::search_bar as tokens;
use md3_theme::md3_tokens::{Dp, supplement::text_field as supplement};
use md3_theme::{SharedTheme, ThemeExt, dp};

use crate::shape::Shaped;
use crate::util;

/// 検索バーの最小の幅 (Compose の `SearchBarMinWidth`)
const MIN_WIDTH: Dp = Dp(360.0);
/// 検索バーの最大の幅 (Compose の `SearchBarMaxWidth`)
const MAX_WIDTH: Dp = Dp(720.0);
/// 先頭のアイコンの大きさ (Compose の `Icon` の既定)
const ICON_SIZE: Dp = Dp(24.0);
/// 先頭のアイコンの前の余白。Compose の 48dp のアイコンの枠の余白 12dp と、枠をずらす 4dp の和
const LEADING_SPACE: Dp = Dp(16.0);
/// 先頭のアイコンと文の間の余白。Compose のアイコンの枠の余白 12dp と、文の前の余白 4dp の和から、枠をずらす 4dp を引いた値
const ICON_TEXT_SPACE: Dp = Dp(12.0);
/// 末尾の要素の後ろの余白と、文と末尾の要素の間の余白 (Compose の文の後ろの余白と、末尾の枠をずらす 4dp)
const TRAILING_SPACE: Dp = Dp(4.0);
/// 末尾の要素が無いときの文の後ろの余白 (Compose の `TextFieldDefaults.contentPaddingWithoutLabel` の 16dp)
const TEXT_END_SPACE: Dp = Dp(16.0);

type ChangeHandler = Box<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// MD3 の検索バー (Compose の `SearchBar` と `SearchBarDefaults.InputField`)
///
/// 角の丸い容器に、検索のアイコン、1 行の入力欄、子の要素を左から並べる。
/// 子の要素 (アイコンボタンやアバターなど) は末尾に並び、文字の色は OnSurfaceVariant になる。
/// 検索バーは高さ 56dp、幅 360dp から 720dp で、[`Styled`] で指定した大きさに従う。
/// 文字列は呼び出し側が持ち、[`SearchBar::new`] に毎回渡す。検索バー自身は文字列を変えず、
/// 編集した後の文字列を [`SearchBar::on_change`] で、確定した文字列を [`SearchBar::on_search`] で求める
///
/// - 入力欄の操作は [`crate::TextField`] と同じ
/// - 入力欄にフォーカスがある間、IME で変換中でなければ、修飾キーなしの Enter で検索を求める
/// - 容器の入力欄と子の要素の外を押すと、入力欄にフォーカスを移す
///
/// [`SearchBar::placeholder`] のプレースホルダーは、文字列が空の間だけ入力欄に表示する。
/// 検索の候補を表示する画面 (Compose の `ExpandedFullScreenSearchBar` や `ExpandedDockedSearchBar`) は持たない
///
/// 支援技術には、入力欄が [`SearchBar::new`] の `label` を名前とする role `textbox` として、文字列と選択とともに公開される。
/// プレースホルダーはプレースホルダーとして公開する
#[derive(IntoElement)]
pub struct SearchBar {
    id: ElementId,
    value: SharedString,
    label: SharedString,
    placeholder: Option<SharedString>,
    leading_icon: StyledIcon,
    children: Vec<AnyElement>,
    on_change: Option<ChangeHandler>,
    on_search: Option<ChangeHandler>,
    style: StyleRefinement,
    theme: Option<SharedTheme>,
}

impl SearchBar {
    /// `value` を入力欄に表示する検索バーを作る
    ///
    /// `label` は入力欄のアクセシブルな名前にする。`label` は画面には表示しない。
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。
    /// `id` が変わるか 1 フレームでも描画されないと、フォーカス・カーソル・選択が失われる
    pub fn new(
        id: impl Into<ElementId>,
        value: impl Into<SharedString>,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            value: value.into(),
            label: label.into(),
            placeholder: None,
            leading_icon: md3_icons::SEARCH.into(),
            children: Vec::new(),
            on_change: None,
            on_search: None,
            style: StyleRefinement::default(),
            theme: None,
        }
    }

    /// 文字列が空の間に、入力欄に `placeholder` を表示する
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// 先頭のアイコンを `icon` にする。既定では検索のアイコン
    ///
    /// アイコンは操作を受けない。押せる操作 (戻るボタンなど) を置く場合は子の要素として末尾に置くこと
    pub fn leading_icon(mut self, icon: impl Into<StyledIcon>) -> Self {
        self.leading_icon = icon.into();
        self
    }

    /// 文字列を編集する操作をされたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には編集した後の文字列を渡す。表示を変えるには、`handler` で呼び出し側の状態を更新し、
    /// 次の描画で [`SearchBar::new`] に渡すこと。渡さなかった場合、編集は取り消される。
    /// 設定しない場合、入力欄は文字列を編集させず、支援技術には read-only として公開される
    pub fn on_change(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// 入力欄で Enter を押して検索を求められたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には最後に編集した後の文字列 ([`SearchBar::on_change`] に渡したものと同じ) を渡す
    pub fn on_search(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_search = Some(Box::new(handler));
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl ParentElement for SearchBar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Styled for SearchBar {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for SearchBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let focus_handle = util::keyed_focus_handle(&self.id, window, cx);

        let mut input = HeadlessTextField::new(self.id.clone(), self.value.clone())
            .track_focus(&focus_handle.clone().tab_index(0))
            .caret_color(theme.color(ColorRole::Primary))
            .selection_color(
                theme
                    .color(ColorRole::Primary)
                    .opacity(supplement::SELECTION_OPACITY),
            )
            .aria_label(self.label)
            .w_full();
        if let Some(placeholder) = &self.placeholder {
            input = input.placeholder(placeholder.clone());
        }
        if let Some(on_change) = self.on_change {
            input = input.on_change(on_change);
        }
        if let Some(on_search) = self.on_search {
            input = input.on_submit(on_search);
        }
        let placeholder = self
            .placeholder
            .filter(|_| self.value.is_empty())
            .map(|placeholder| {
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .overflow_hidden()
                    .text_color(theme.color(tokens::SUPPORTING_TEXT_COLOR))
                    .child(placeholder)
            });

        let has_children = !self.children.is_empty();
        // 角丸は描画時の大きさで決まるため、背景は Shaped で包んだ層に描く
        let container = Shaped::new(
            div()
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .bottom_0()
                .bg(theme.color(tokens::CONTAINER_COLOR)),
            theme.clone(),
            tokens::CONTAINER_SHAPE,
        );
        let mut text_style = theme.text_style(tokens::INPUT_TEXT_FONT);
        text_style.color = Some(theme.color(tokens::INPUT_TEXT_COLOR));
        let mut bar = div()
            .relative()
            .flex()
            .items_center()
            .h(dp(tokens::CONTAINER_HEIGHT))
            .min_w(dp(MIN_WIDTH))
            .max_w(dp(MAX_WIDTH))
            .pl(dp(LEADING_SPACE))
            .pr(dp(if has_children {
                TRAILING_SPACE
            } else {
                TEXT_END_SPACE
            }))
            .cursor(CursorStyle::IBeam)
            // 文の外を押してもフォーカスを移す。文と子の要素の上の押下はそれぞれが受ける
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                window.focus(&focus_handle, cx);
            })
            .child(container)
            .child(
                svg()
                    .data(self.leading_icon.svg().as_bytes())
                    .relative()
                    .flex_none()
                    .size(dp(ICON_SIZE))
                    .text_color(theme.color(tokens::LEADING_ICON_COLOR)),
            )
            .child(
                div()
                    .debug_selector(|| "md3-search-bar-input".into())
                    .relative()
                    .flex_1()
                    .min_w_0()
                    .ml(dp(ICON_TEXT_SPACE))
                    .children(placeholder)
                    .child(input),
            )
            .when(has_children, |this| {
                this.child(
                    div()
                        .debug_selector(|| "md3-search-bar-trailing".into())
                        .relative()
                        .flex()
                        .flex_none()
                        .items_center()
                        .ml(dp(TRAILING_SPACE))
                        .text_color(theme.color(tokens::TRAILING_ICON_COLOR))
                        .cursor(CursorStyle::Arrow)
                        .children(self.children),
                )
            });
        bar.text_style().refine(&text_style);
        bar.style().refine(&self.style);
        bar
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use gpui::{Context, Modifiers, Render, TestAppContext, VisualTestContext, point, px};

    use super::*;

    /// (100, 100) に幅 400 の検索バーを置き、文字列と検索を求められた文字列を持つビュー
    struct BarView {
        value: SharedString,
        searches: Rc<RefCell<Vec<SharedString>>>,
        trailing: bool,
    }

    impl Render for BarView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let searches = self.searches.clone();
            div().size_full().child(
                SearchBar::new("search", self.value.clone(), "Search")
                    .placeholder("Search mail")
                    .on_change(cx.listener(|this, value: &SharedString, _, cx| {
                        this.value = value.clone();
                        cx.notify();
                    }))
                    .on_search(move |value, _, _| searches.borrow_mut().push(value.clone()))
                    .when(self.trailing, |this| this.child(div().size(px(48.))))
                    .absolute()
                    .left(px(100.))
                    .top(px(100.))
                    .w(px(400.)),
            )
        }
    }

    fn setup(
        cx: &mut TestAppContext,
        trailing: bool,
    ) -> (gpui::Entity<BarView>, &mut VisualTestContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            BarView {
                value: SharedString::default(),
                searches: Rc::default(),
                trailing,
            }
        });
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        (view, cx)
    }

    #[cfg(debug_assertions)]
    fn bounds(cx: &mut VisualTestContext, selector: &'static str) -> (f32, f32, f32, f32) {
        let b = cx
            .debug_bounds(selector)
            .expect("描画した要素の境界があること");
        (
            f32::from(b.origin.x),
            f32::from(b.origin.y),
            f32::from(b.size.width),
            f32::from(b.size.height),
        )
    }

    // gpui は debug_assertions のあるビルドでだけ debug_selector の位置を記録する
    #[cfg(debug_assertions)]
    #[gpui::test]
    fn アイコン_入力欄_末尾の要素を_compose_と同じ余白で並べる(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx, true);
        let container = cx.update(|window, cx| {
            let scale = window.scale_factor();
            let color: gpui::Background = cx.md3_theme().color(tokens::CONTAINER_COLOR).into();
            window
                .painted_quads()
                .into_iter()
                .filter(|quad| quad.background == color)
                .map(|quad| {
                    let b = quad.bounds;
                    (
                        b.origin.x.0 / scale,
                        b.origin.y.0 / scale,
                        b.size.width.0 / scale,
                        b.size.height.0 / scale,
                    )
                })
                .collect::<Vec<_>>()
        });
        assert_eq!(container, [(100., 100., 400., 56.)]);
        // 先頭の余白 16、アイコン 24、アイコンと文の間 12
        let (x, _, _, _) = bounds(cx, "md3-search-bar-input");
        assert_eq!(x, 152.);
        // 末尾の要素の後ろの余白 4、文と末尾の要素の間 4
        let (x, y, w, h) = bounds(cx, "md3-search-bar-trailing");
        assert_eq!((x, y, w, h), (448., 104., 48., 48.));
        let (input_x, _, input_w, _) = bounds(cx, "md3-search-bar-input");
        assert_eq!(input_x + input_w, 444.);
    }

    // gpui は debug_assertions のあるビルドでだけ debug_selector の位置を記録する
    #[cfg(debug_assertions)]
    #[gpui::test]
    fn 末尾の要素が無ければ文の後ろに_16_の余白を空ける(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx, false);
        let (x, _, w, _) = bounds(cx, "md3-search-bar-input");
        assert_eq!(x + w, 484.);
    }

    #[gpui::test]
    fn 容器を押すと入力欄にフォーカスを移し_enter_で検索を求める(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, false);
        // 先頭のアイコンの上を押す
        cx.simulate_click(point(px(120.), px(128.)), Modifiers::none());
        cx.simulate_input("rust");
        cx.simulate_keystrokes("enter");
        let (value, searches) = view.read_with(cx, |view, _| {
            (view.value.clone(), view.searches.borrow().clone())
        });
        assert_eq!(value, "rust");
        assert_eq!(searches, ["rust"]);
    }
}
