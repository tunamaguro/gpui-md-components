// 全画面の検索の構成と色は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SearchBar.kt
//         (`ExpandedFullScreenSearchBar`、`FullScreenSearchBarLayout`、`SearchBarDefaults.colors`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2022 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 の全画面の検索

use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, MouseButton, ParentElement,
    Refineable, RenderOnce, SharedString, StatefulInteractiveElement, StyleRefinement, Styled,
    Window, div,
};
use gpui_headless::dialog::Dialog as HeadlessDialog;
use gpui_headless::text_field::TextField as HeadlessTextField;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::search_view as tokens;
use md3_theme::md3_tokens::{Dp, supplement::text_field as supplement};
use md3_theme::{SharedTheme, ThemeExt, dp};

use crate::{IconButton, SearchBar, util};

/// 戻るボタンの前の余白。検索バーの先頭のアイコンの前の余白 16dp から、48dp のボタンの枠の余白 12dp を引いた値
const BACK_BUTTON_SPACE: Dp = Dp(4.0);
/// 文の後ろの余白 (Compose の `TextFieldDefaults.contentPaddingWithoutLabel` の 16dp)
const TEXT_END_SPACE: Dp = Dp(16.0);
/// 区切り線の太さ (Compose の `HorizontalDivider` の既定)
const DIVIDER_THICKNESS: Dp = Dp(1.0);

type ChangeHandler = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;
type ExpandedChangeHandler = Rc<dyn Fn(&bool, &mut Window, &mut App)>;

/// 検索バーと、開くとウィンドウ全体に広がる検索の画面 (Compose の `SearchBar` と `ExpandedFullScreenSearchBar`)
///
/// 閉じている間は [`SearchBar`] を描く。開いている間は、ウィンドウ全体を覆うモーダルのダイアログに、
/// 戻るボタンと入力欄を並べた高さ 72dp の見出し、区切り線、子の要素 (検索の候補や結果) を縦に並べる。
/// 子の要素は見出しの下の残りの高さに収まるよう置かれ、スクロールはしない。
///
/// 開いているかどうかと文字列は呼び出し側が持ち、[`SearchView::new`] に毎回渡す。
/// 次の操作で、[`SearchView::on_expanded_change`] のハンドラに開閉を求める
///
/// - 閉じている間に検索バーを押すか、検索バーで文字列を編集する操作をすると、開くよう求める
/// - 開いている間に、戻るボタンを押すか、修飾キーなしの Escape を押すと、閉じるよう求める
///
/// 開いたときは見出しの入力欄にフォーカスを移し、閉じたときは開く前にフォーカスのあった要素へ戻す。
/// 入力欄の操作は [`SearchBar`] と同じで、編集した後の文字列を [`SearchView::on_change`] で、
/// Enter で確定した文字列を [`SearchView::on_search`] で求める。
/// [`Styled`] は閉じている間の検索バーに適用する
///
/// 支援技術には、開いている間はダイアログが `label` を名前とする modal の `dialog` として、
/// 入力欄が `label` を名前とする role `textbox` として公開される
#[derive(IntoElement)]
pub struct SearchView {
    id: ElementId,
    value: SharedString,
    label: SharedString,
    expanded: bool,
    placeholder: Option<SharedString>,
    on_change: Option<ChangeHandler>,
    on_search: Option<ChangeHandler>,
    on_expanded_change: Option<ExpandedChangeHandler>,
    children: Vec<AnyElement>,
    style: StyleRefinement,
    theme: Option<SharedTheme>,
}

impl SearchView {
    /// `value` を入力欄に表示し、`expanded` なら検索の画面を開いて描く検索を作る
    ///
    /// `label` は入力欄と検索の画面のアクセシブルな名前にする。`label` は画面には表示しない。
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。
    /// `id` が変わるか 1 フレームでも描画されないと、フォーカス・カーソル・選択が失われる
    pub fn new(
        id: impl Into<ElementId>,
        value: impl Into<SharedString>,
        label: impl Into<SharedString>,
        expanded: bool,
    ) -> Self {
        Self {
            id: id.into(),
            value: value.into(),
            label: label.into(),
            expanded,
            placeholder: None,
            on_change: None,
            on_search: None,
            on_expanded_change: None,
            children: Vec::new(),
            style: StyleRefinement::default(),
            theme: None,
        }
    }

    /// 文字列が空の間に、入力欄に `placeholder` を表示する
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// 文字列を編集する操作をされたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には編集した後の文字列を渡す。表示を変えるには、`handler` で呼び出し側の状態を更新し、
    /// 次の描画で [`SearchView::new`] に渡すこと。設定しない場合、入力欄は文字列を編集させない
    pub fn on_change(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// 入力欄で Enter を押して検索を求められたときに、その時点の文字列で `handler` を呼ぶ
    pub fn on_search(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_search = Some(Rc::new(handler));
        self
    }

    /// 開くか閉じるよう求められたときに、求める状態 (開くなら `true`) で `handler` を呼ぶ
    pub fn on_expanded_change(
        mut self,
        handler: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_expanded_change = Some(Rc::new(handler));
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl ParentElement for SearchView {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Styled for SearchView {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for SearchView {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let expand = self.on_expanded_change.clone();
        let request = move |expanded: bool, window: &mut Window, cx: &mut App| {
            if let Some(handler) = &expand {
                handler(&expanded, window, cx);
            }
        };

        // 検索バーと見出しの入力欄は同時に描くため、フォーカスを分ける。
        // 閉じたときはダイアログが検索バーの入力欄へフォーカスを戻す
        let view_focus = util::keyed_focus_handle(&(self.id.clone(), "view").into(), window, cx);
        let mut bar = SearchBar::new(
            (self.id.clone(), "bar"),
            self.value.clone(),
            self.label.clone(),
        )
        .theme(theme.clone());
        if let Some(placeholder) = &self.placeholder {
            bar = bar.placeholder(placeholder.clone());
        }
        if let Some(on_change) = self.on_change.clone() {
            let request = request.clone();
            bar = bar.on_change(move |value, window, cx| {
                on_change(value, window, cx);
                request(true, window, cx);
            });
        }
        if let Some(on_search) = self.on_search.clone() {
            bar = bar.on_search(move |value, window, cx| on_search(value, window, cx));
        }
        bar.style().refine(&self.style);
        let collapsed = div()
            .id(self.id.clone())
            // 入力欄は押下を親に伝えないため、捕捉の段階で受ける
            .capture_any_mouse_down({
                let request = request.clone();
                let expanded = self.expanded;
                move |event, window, cx| {
                    if !expanded && event.button == MouseButton::Left {
                        request(true, window, cx);
                    }
                }
            })
            .child(bar);

        let dismiss = request.clone();
        let mut dialog = HeadlessDialog::new((self.id.clone(), "dialog"), self.expanded)
            .on_dismiss(move |_, window, cx| dismiss(false, window, cx))
            .initial_focus(&view_focus);
        if self.expanded {
            let mut input = HeadlessTextField::new("input", self.value.clone())
                .track_focus(&view_focus.clone().tab_index(0))
                .caret_color(theme.color(ColorRole::Primary))
                .selection_color(
                    theme
                        .color(ColorRole::Primary)
                        .opacity(supplement::SELECTION_OPACITY),
                )
                .aria_label(self.label.clone())
                .w_full();
            if let Some(placeholder) = &self.placeholder {
                input = input.placeholder(placeholder.clone());
            }
            if let Some(on_change) = self.on_change {
                input = input.on_change(move |value, window, cx| on_change(value, window, cx));
            }
            if let Some(on_search) = self.on_search {
                input = input.on_submit(move |value, window, cx| on_search(value, window, cx));
            }
            let placeholder =
                self.placeholder
                    .filter(|_| self.value.is_empty())
                    .map(|placeholder| {
                        div()
                            .absolute()
                            .top_0()
                            .left_0()
                            .right_0()
                            .truncate()
                            .text_color(theme.color(tokens::HEADER_SUPPORTING_TEXT_COLOR))
                            // 改行ごとに行を作らないよう、1 行にする
                            .child(SharedString::from(placeholder.replace(['\r', '\n'], " ")))
                    });
            let mut text_style = theme.text_style(tokens::HEADER_INPUT_TEXT_FONT);
            text_style.color = Some(theme.color(tokens::HEADER_INPUT_TEXT_COLOR));
            let back = {
                let request = request.clone();
                IconButton::new("back", md3_icons::ARROW_BACK, "Back")
                    .theme(theme.clone())
                    .on_click(move |_, window, cx| request(false, window, cx))
            };
            let mut header = div()
                .id("header")
                .flex()
                .flex_none()
                .items_center()
                .h(dp(tokens::FULL_SCREEN_HEADER_CONTAINER_HEIGHT))
                .pl(dp(BACK_BUTTON_SPACE))
                .pr(dp(TEXT_END_SPACE))
                .text_color(theme.color(tokens::HEADER_LEADING_ICON_COLOR))
                .child(back)
                .child(
                    div()
                        .relative()
                        .flex_1()
                        .min_w_0()
                        .cursor_text()
                        .children(placeholder)
                        .child(input),
                );
            header.text_style().refine(&text_style);
            dialog = dialog
                .scrim(div().size_full())
                .aria_label(self.label)
                .size_full()
                .flex()
                .flex_col()
                .bg(theme.color(tokens::CONTAINER_COLOR))
                .text_color(theme.color(ColorRole::OnSurface))
                .child(header)
                .child(
                    div()
                        .flex_none()
                        .h(dp(DIVIDER_THICKNESS))
                        .bg(theme.color(tokens::DIVIDER_COLOR)),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_h_0()
                        .overflow_hidden()
                        .children(self.children),
                );
        }
        div().child(collapsed).child(dialog)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use gpui::{Context, Modifiers, Render, TestAppContext, VisualTestContext, point, px};

    use super::*;

    /// (100, 100) に検索を置き、文字列と開いているかと、検索を求められた文字列を持つビュー
    struct SearchViewView {
        value: SharedString,
        expanded: bool,
        searches: Rc<RefCell<Vec<SharedString>>>,
    }

    impl Render for SearchViewView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let searches = self.searches.clone();
            div().size_full().child(
                div().absolute().left(px(100.)).top(px(100.)).child(
                    SearchView::new("search", self.value.clone(), "Search", self.expanded)
                        .placeholder("Search mail")
                        .on_change(cx.listener(|this, value: &SharedString, _, cx| {
                            this.value = value.clone();
                            cx.notify();
                        }))
                        .on_search(move |value, _, _| searches.borrow_mut().push(value.clone()))
                        .on_expanded_change(cx.listener(|this, expanded: &bool, _, cx| {
                            this.expanded = *expanded;
                            cx.notify();
                        }))
                        .child(div().h(px(50.)).bg(gpui::red())),
                ),
            )
        }
    }

    fn setup(cx: &mut TestAppContext) -> (gpui::Entity<SearchViewView>, &mut VisualTestContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            SearchViewView {
                value: SharedString::default(),
                expanded: false,
                searches: Rc::default(),
            }
        });
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        (view, cx)
    }

    fn state(view: &gpui::Entity<SearchViewView>, cx: &mut VisualTestContext) -> (String, bool) {
        view.read_with(cx, |view, _| (view.value.to_string(), view.expanded))
    }

    /// 赤く塗った子の要素の (x, y, 幅)
    fn content(cx: &mut VisualTestContext) -> Option<(f32, f32, f32)> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            window
                .painted_quads()
                .into_iter()
                .find(|quad| quad.background == gpui::red().into())
                .map(|quad| {
                    let b = quad.bounds;
                    (
                        b.origin.x.0 / scale,
                        b.origin.y.0 / scale,
                        b.size.width.0 / scale,
                    )
                })
        })
    }

    #[gpui::test]
    fn 検索バーを押すと開いて見出しの入力欄で入力できる(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        assert_eq!(content(cx), None, "閉じている間は子を描かないこと");
        cx.simulate_click(point(px(200.), px(128.)), Modifiers::none());
        assert_eq!(state(&view, cx), (String::new(), true));
        // 見出しの高さ 72 と区切り線の 1 の下に、ウィンドウの幅いっぱいに子を置く
        let (x, y, width) = content(cx).expect("開いている間は子を描くこと");
        assert_eq!((x, y), (0., 73.));
        assert!(width > 1000., "{width}");
        cx.simulate_keystrokes("a b");
        assert_eq!(state(&view, cx), ("ab".into(), true));
        cx.simulate_keystrokes("enter");
        assert_eq!(
            *view.read_with(cx, |view, _| view.searches.clone()).borrow(),
            ["ab"]
        );
    }

    #[gpui::test]
    fn 検索バーで入力すると開く(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        cx.simulate_keystrokes("tab x");
        assert_eq!(state(&view, cx), ("x".into(), true));
        cx.simulate_keystrokes("y");
        assert_eq!(
            state(&view, cx),
            ("xy".into(), true),
            "開いた後は見出しの入力欄で入力できること"
        );
    }

    #[gpui::test]
    fn escape_と戻るボタンで閉じて検索バーへフォーカスを戻す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        cx.simulate_click(point(px(200.), px(128.)), Modifiers::none());
        cx.simulate_keystrokes("escape");
        assert_eq!(state(&view, cx), (String::new(), false));
        assert_eq!(content(cx), None);
        // 検索バーの入力欄にフォーカスが戻り、入力すると開く
        cx.simulate_keystrokes("c");
        assert_eq!(state(&view, cx), ("c".into(), true));
        // 戻るボタンは見出しの左端から 4 の位置の 48 の枠の中央にある
        cx.simulate_click(point(px(28.), px(36.)), Modifiers::none());
        assert_eq!(state(&view, cx), ("c".into(), false));
    }
}
