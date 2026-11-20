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
    AnyElement, App, CursorStyle, ElementId, FocusHandle, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Refineable, RenderOnce, SharedString, StatefulInteractiveElement,
    StyleRefinement, Styled, Window, div, prelude::FluentBuilder, svg,
};
use gpui_headless::text_field::TextField as HeadlessTextField;
use md3_icons::StyledIcon;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::search_bar as tokens;
use md3_theme::md3_tokens::{Dp, focus_ring, supplement::text_field as supplement};
use md3_theme::{SharedTheme, ThemeExt, dp};

use crate::shape::Shaped;
use crate::util;

/// 検索バーの既定の幅 (Compose の `SearchBarMinWidth`)
const DEFAULT_WIDTH: Dp = Dp(360.0);
/// 末尾の要素を置く枠の最小の大きさ (Compose の `TextFieldImplKt.MinIconSize` の 48dp)
const TRAILING_MIN_SIZE: Dp = Dp(48.0);
/// 先頭のアイコンの大きさ (Compose の `Icon` の既定)
const ICON_SIZE: Dp = Dp(24.0);
/// 先頭のアイコンの前の余白。Compose の 48dp のアイコンの枠の余白 12dp と、枠をずらす 4dp の和
const LEADING_SPACE: Dp = Dp(16.0);
/// 先頭のアイコンと文の間の余白。Compose のアイコンの枠の余白 12dp と、文の前の余白 4dp の和から、枠をずらす 4dp を引いた値
const ICON_TEXT_SPACE: Dp = Dp(12.0);
/// 末尾の要素の枠の後ろの余白と、文と枠の間の余白 (Compose の文の後ろの余白と、末尾の枠をずらす 4dp)
const TRAILING_SPACE: Dp = Dp(4.0);
/// 末尾の要素が無いときの文の後ろの余白 (Compose の `TextFieldDefaults.contentPaddingWithoutLabel` の 16dp)
const TEXT_END_SPACE: Dp = Dp(16.0);

type ChangeHandler = Box<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// MD3 の検索バー (Compose の `SearchBar` と `SearchBarDefaults.InputField`)
///
/// 角の丸い容器に、検索のアイコン、1 行の入力欄、子の要素を左から並べる。
/// 子の要素 (アイコンボタンやアバターなど) は末尾の高さと幅 48dp 以上の枠の中央に並び、文字の色は OnSurfaceVariant になる。
/// 検索バーは高さ 56dp、既定の幅 360dp で、親の幅を超えない。[`Styled`] で指定した大きさに従う。
/// 入力欄を使うには、先頭のアイコンと末尾の子の枠と余白の和を超える幅が要る。和以下の幅では入力欄は幅 0 になり、
/// 和に満たない幅では末尾の子が容器からはみ出す。
/// プレースホルダーは改行を空白にして 1 行に収め、収まらない部分を省略記号にする。
/// 文字列は呼び出し側が持ち、[`SearchBar::new`] に毎回渡す。検索バー自身は文字列を変えず、
/// 編集した後の文字列を [`SearchBar::on_change`] で、確定した文字列を [`SearchBar::on_search`] で求める
///
/// - 文字の入力と IME による入力、カーソルの移動と選択、クリップボードの操作。改行は入れない
/// - 入力欄にフォーカスがある間、IME で変換中でなければ、修飾キーなしの Enter で検索を求める
/// - 容器の、入力欄とフォーカスを受ける子の要素の外を押すと、入力欄にフォーカスを移す
///
/// 入力欄にキーボードでフォーカスを移した間は、容器の内側に Secondary のフォーカスリングを描く。
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
    focus_handle: Option<FocusHandle>,
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
            focus_handle: None,
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
    /// `handler` には Enter を押した時点で入力欄が持つ文字列を渡す。直前の編集を呼び出し側が受け入れていない場合も、
    /// 次の描画までは編集した後の文字列になる
    pub fn on_search(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_search = Some(Box::new(handler));
        self
    }

    /// `focus_handle` を入力欄のフォーカスとして使う
    ///
    /// 呼び出し側から `focus_handle` で入力欄へのフォーカスの移動や判定ができる。`focus_handle` は Tab で到達できるように設定される。
    /// 指定しない場合は検索バーが内部でフォーカスを持つ
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
        let focus_handle = self
            .focus_handle
            .unwrap_or_else(|| util::keyed_focus_handle(&self.id, window, cx));
        let focus_visible = gpui_headless::is_focus_visible(&focus_handle, window, cx);

        let editable = self.on_change.is_some();
        let mut input = HeadlessTextField::new("input", self.value.clone())
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
                    .truncate()
                    .text_color(theme.color(tokens::SUPPORTING_TEXT_COLOR))
                    // 改行ごとに行を作らないよう、1 行にする
                    .child(SharedString::from(placeholder.replace(['\r', '\n'], " ")))
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
        // 子の要素の id を検索バーごとに分けるため、外側の箱にも id を付ける
        let mut bar = div()
            .id(self.id.clone())
            .relative()
            .flex()
            .items_center()
            .h(dp(tokens::CONTAINER_HEIGHT))
            .w(dp(DEFAULT_WIDTH))
            .max_w_full()
            .pl(dp(LEADING_SPACE))
            .pr(dp(if has_children {
                TRAILING_SPACE
            } else {
                TEXT_END_SPACE
            }))
            .cursor(CursorStyle::IBeam)
            // 文の外を押してもフォーカスを移す。文の上の押下はテキストフィールドが受けてここに届かず、
            // フォーカスを受ける子の要素の上の押下は、子がフォーカスを移して既定の処理を止める
            .on_mouse_down(MouseButton::Left, {
                let focus_handle = focus_handle.clone();
                move |_, window, cx| {
                    if !window.default_prevented() {
                        window.focus(&focus_handle, cx);
                        // 入力欄に既にフォーカスがあると、フォーカスの変化ではソフトウェアキーボードが再び開かないため要求する
                        if editable {
                            window.request_virtual_keyboard();
                        }
                        // フォーカスを受ける祖先が、移したフォーカスを上書きしないようにする
                        window.prevent_default();
                    }
                }
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
                    // 呼び出し側の子の id を、内蔵の入力欄の id と分ける
                    div()
                        .id("trailing")
                        .debug_selector(|| "md3-search-bar-trailing".into())
                        .relative()
                        .flex()
                        .flex_none()
                        .items_center()
                        .justify_center()
                        .min_w(dp(TRAILING_MIN_SIZE))
                        .min_h(dp(TRAILING_MIN_SIZE))
                        .ml(dp(TRAILING_SPACE))
                        .text_color(theme.color(tokens::TRAILING_ICON_COLOR))
                        .cursor(CursorStyle::Arrow)
                        .children(self.children),
                )
            })
            .when(focus_visible, |this| {
                // Compose はキーボードのフォーカスで容器の内側にリングを描く (`RippleThemeConfiguration.Focus.InsetRing`)
                this.child(Shaped::new(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .border(dp(focus_ring::WIDTH))
                        .border_color(theme.color(tokens::FOCUS_INDICATOR_COLOR)),
                    theme.clone(),
                    tokens::CONTAINER_SHAPE,
                ))
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

    use gpui::{
        Context, KeyDownEvent, KeyUpEvent, Keystroke, Modifiers, Render, TestAppContext,
        VisualTestContext, point, px,
    };

    use super::*;
    use gpui_headless::button::Button;

    /// (100, 100) に検索バーを置き、文字列と、検索を求められた文字列と、押された末尾のボタンを持つビュー
    struct BarView {
        value: SharedString,
        searches: Rc<RefCell<Vec<SharedString>>>,
        clicks: Rc<RefCell<Vec<&'static str>>>,
        trailing: bool,
        /// 検索バーを置く親の幅
        parent_width: f32,
        /// 同じ子の id を持つ 2 本目の検索バーを置くか
        second: bool,
        focus: FocusHandle,
        /// 検索バーを包む、フォーカスを受ける親のフォーカス
        parent_focus: FocusHandle,
    }

    impl Render for BarView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let bar = |id: &'static str, this: &Self, cx: &mut Context<Self>| {
                let searches = this.searches.clone();
                let clicks = this.clicks.clone();
                SearchBar::new(id, this.value.clone(), "Search")
                    .placeholder("Search mail")
                    .on_change(cx.listener(|this, value: &SharedString, _, cx| {
                        this.value = value.clone();
                        cx.notify();
                    }))
                    .on_search(move |value, _, _| searches.borrow_mut().push(value.clone()))
                    .when(this.trailing, |this| {
                        this.child(
                            // アイコンボタンと同じ大きさ 40 の、フォーカスを受けるボタン
                            // 内蔵の入力欄と同じ id を使っても衝突しないこと
                            Button::new("input")
                                .size(px(40.))
                                .on_click(move |_, _, _| clicks.borrow_mut().push(id)),
                        )
                    })
            };
            div()
                .absolute()
                .left(px(100.))
                .top(px(100.))
                .w(px(self.parent_width))
                .track_focus(&self.parent_focus)
                .flex()
                .flex_col()
                .child(bar("first", self, cx).track_focus(&self.focus))
                .when(self.second, |this| this.child(bar("second", self, cx)))
        }
    }

    fn setup(
        cx: &mut TestAppContext,
        f: impl FnOnce(&mut BarView),
    ) -> (gpui::Entity<BarView>, &mut VisualTestContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            let mut view = BarView {
                value: SharedString::default(),
                searches: Rc::default(),
                clicks: Rc::default(),
                trailing: true,
                parent_width: 800.,
                second: false,
                focus: cx.focus_handle(),
                parent_focus: cx.focus_handle(),
            };
            f(&mut view);
            view
        });
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        (view, cx)
    }

    /// 色 `role` で塗った矩形 (x, y, 幅, 高さ)
    fn quads(cx: &mut VisualTestContext, role: ColorRole) -> Vec<(f32, f32, f32, f32)> {
        cx.update(|window, cx| {
            let scale = window.scale_factor();
            let color: gpui::Background = cx.md3_theme().color(role).into();
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
                .collect()
        })
    }

    /// ボタンはキーを離したときに押されるため、押して離す
    fn press(cx: &mut VisualTestContext, key: &'static str) {
        let keystroke = Keystroke::parse(key).expect("テストのキー表記が有効であること");
        cx.simulate_event(KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(KeyUpEvent { keystroke });
        cx.run_until_parked();
    }

    /// 枠線を色 `role` で描いた矩形の数
    fn borders(cx: &mut VisualTestContext, role: ColorRole) -> usize {
        cx.update(|window, cx| {
            let color = cx.md3_theme().color(role);
            window
                .painted_quads()
                .into_iter()
                .filter(|quad| quad.border_color == color && quad.border_widths.top.0 > 0.)
                .count()
        })
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
        let (_, cx) = setup(cx, |_| {});
        assert_eq!(
            quads(cx, tokens::CONTAINER_COLOR),
            [(100., 100., 360., 56.)],
            "既定の幅は 360"
        );
        // 先頭の余白 16、アイコン 24、アイコンと文の間 12
        let (x, _, w, _) = bounds(cx, "md3-search-bar-input");
        assert_eq!(x, 152.);
        // 末尾の枠は 48 以上で、後ろの余白 4、文との間 4
        assert_eq!(
            bounds(cx, "md3-search-bar-trailing"),
            (408., 104., 48., 48.)
        );
        assert_eq!(x + w, 404.);
    }

    #[cfg(debug_assertions)]
    #[gpui::test]
    fn 末尾の要素が無ければ文の後ろに_16_の余白を空ける(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx, |view| view.trailing = false);
        let (x, _, w, _) = bounds(cx, "md3-search-bar-input");
        assert_eq!(x + w, 444.);
    }

    #[gpui::test]
    fn 親の幅が既定の幅より狭ければ親の幅に縮む(cx: &mut TestAppContext) {
        let (_, cx) = setup(cx, |view| view.parent_width = 300.);
        assert_eq!(
            quads(cx, tokens::CONTAINER_COLOR),
            [(100., 100., 300., 56.)]
        );
    }

    #[gpui::test]
    fn 容器を押すと入力欄にフォーカスを移し_enter_で検索を求める(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, |_| {});
        // 先頭のアイコンの上を押す
        cx.simulate_click(point(px(120.), px(128.)), Modifiers::none());
        let focus = view.read_with(cx, |view, _| view.focus.clone());
        assert!(
            cx.update(|window, _| focus.is_focused(window)),
            "フォーカスを受ける親にフォーカスを奪われないこと"
        );
        cx.simulate_input("rust");
        cx.simulate_keystrokes("enter");
        let (value, searches) = view.read_with(cx, |view, _| {
            (view.value.clone(), view.searches.borrow().clone())
        });
        assert_eq!(value, "rust");
        assert_eq!(searches, ["rust"]);
    }

    #[gpui::test]
    fn 末尾のボタンを押すとボタンにフォーカスが残る(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, |_| {});
        cx.simulate_click(point(px(432.), px(128.)), Modifiers::none());
        let focus = view.read_with(cx, |view, _| view.focus.clone());
        assert_eq!(
            *view.read_with(cx, |view, _| view.clicks.clone()).borrow(),
            ["first"]
        );
        assert!(
            cx.update(|window, cx| window.focused(cx).is_some() && !focus.is_focused(window)),
            "押したボタンからフォーカスを奪わないこと"
        );
    }

    #[gpui::test]
    fn 同じ子の_id_を持つ_2_本の検索バーの子は別に操作される(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, |view| view.second = true);
        // Tab の順は 1 本目の入力欄、1 本目のボタン、2 本目の入力欄、2 本目のボタン
        for key in ["tab", "tab", "space", "tab", "tab", "space"] {
            press(cx, key);
        }
        assert_eq!(
            *view.read_with(cx, |view, _| view.clicks.clone()).borrow(),
            ["first", "second"]
        );
    }

    #[gpui::test]
    fn キーボードでフォーカスした間だけフォーカスリングを描く(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx, |_| {});
        cx.simulate_click(point(px(120.), px(128.)), Modifiers::none());
        assert_eq!(borders(cx, tokens::FOCUS_INDICATOR_COLOR), 0);
        cx.simulate_keystrokes("tab shift-tab");
        assert!(borders(cx, tokens::FOCUS_INDICATOR_COLOR) > 0);
    }
}
