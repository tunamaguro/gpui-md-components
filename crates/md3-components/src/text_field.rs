// プレースホルダーと接辞を表示する条件は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/internal/TextFieldImpl.kt
//         (`CommonDecorationBox`、`placeholderOpacity`、`affixOpacity`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021, 2023 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある。入力欄の見た目は field.rs にある

//! MD3 のテキストフィールド

use gpui::{
    App, CursorStyle, ElementId, InteractiveElement, IntoElement, MouseButton, ParentElement,
    Refineable, RenderOnce, SharedString, StatefulInteractiveElement, StyleRefinement, Styled,
    Window, div, prelude::FluentBuilder,
};
use gpui_headless::text_field::TextField as HeadlessTextField;
use md3_icons::StyledIcon;

use crate::field::{Field, FieldVariant};
use crate::util;
use md3_theme::{SharedTheme, ThemeExt};

/// 入力欄の層が入力欄の hover を知るための group 名
const GROUP: &str = "md3-text-field";

type ChangeHandler = Box<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// テキストフィールドの入力欄の種類
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextFieldVariant {
    /// 容器を塗り、下端に線を引く
    #[default]
    Filled,
    /// 容器を塗らず、枠線で囲む
    Outlined,
}

/// MD3 の 1 行のテキストフィールド
///
/// 文字列は呼び出し側が持ち、[`TextField::new`] に毎回渡す。テキストフィールド自身は文字列を変えず、
/// 編集した後の文字列を [`TextField::on_change`] で求める。IME で変換中の文字列も含めて求める
///
/// - 文字の入力と IME による入力。改行は入れない
/// - 左右の矢印キー・Home・End でカーソルを移し、Shift と合わせると選択を広げる。
///   Ctrl (macOS では Option) と合わせると語の単位で移す
/// - Backspace・Delete で文字を消し、Ctrl (macOS では Option) と合わせると語の単位で消す
/// - Ctrl (macOS では Command) と A・X・C・V で、すべての選択・切り取り・コピー・貼り付け
/// - マウスで押した位置へカーソルを移し、ドラッグで選択する。2 回続けて押すと語を、3 回続けて押すとすべてを選択する。
///   入力欄の文の外を押すとフォーカスを移す
///
/// 入力欄は高さ 56dp、幅 280dp 以上で、[`Styled`] で指定した幅に広がる。文が収まらない場合は、カーソルが見える位置まで横にずらす。
/// [`TextField::label`] のラベルは、文字列が空でフォーカスも無い間は入力欄の中に、それ以外は入力欄の上端に小さく表示する。
/// プレースホルダー・接頭辞・接尾辞は、ラベルを入力欄の中に表示している間は表示しない。プレースホルダーは文字列が空の間だけ表示する
///
/// 支援技術には role `textbox` として、文字列と選択を公開する。
/// 名前はラベルか [`TextField::aria_label`] で与えること。補足の文は説明として、プレースホルダーはプレースホルダーとして公開する
#[derive(IntoElement)]
pub struct TextField {
    id: ElementId,
    value: SharedString,
    variant: TextFieldVariant,
    label: Option<SharedString>,
    aria_label: Option<SharedString>,
    placeholder: Option<SharedString>,
    leading_icon: Option<StyledIcon>,
    trailing_icon: Option<StyledIcon>,
    prefix: Option<SharedString>,
    suffix: Option<SharedString>,
    supporting_text: Option<SharedString>,
    error: bool,
    disabled: bool,
    read_only: bool,
    on_change: Option<ChangeHandler>,
    style: StyleRefinement,
    theme: Option<SharedTheme>,
}

impl TextField {
    /// `value` を表示する [`TextFieldVariant::Filled`] のテキストフィールドを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。
    /// `id` が変わるか 1 フレームでも描画されないと、フォーカス・カーソル・選択が失われる
    pub fn new(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            value: value.into(),
            variant: TextFieldVariant::default(),
            label: None,
            aria_label: None,
            placeholder: None,
            leading_icon: None,
            trailing_icon: None,
            prefix: None,
            suffix: None,
            supporting_text: None,
            error: false,
            disabled: false,
            read_only: false,
            on_change: None,
            style: StyleRefinement::default(),
            theme: None,
        }
    }

    /// 入力欄の種類を `variant` にする
    pub fn variant(mut self, variant: TextFieldVariant) -> Self {
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

    /// 文字列が空の間に、入力する内容の例やヒントとして `placeholder` を表示する
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// 入力欄の先頭に `icon` を表示する
    pub fn leading_icon(mut self, icon: impl Into<StyledIcon>) -> Self {
        self.leading_icon = Some(icon.into());
        self
    }

    /// 入力欄の末尾に `icon` を表示する。アイコンは操作を受けない
    pub fn trailing_icon(mut self, icon: impl Into<StyledIcon>) -> Self {
        self.trailing_icon = Some(icon.into());
        self
    }

    /// 文の前に `prefix` を表示する。接頭辞は文字列に含まれない
    pub fn prefix(mut self, prefix: impl Into<SharedString>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// 文の後に `suffix` を表示する。接尾辞は文字列に含まれない
    pub fn suffix(mut self, suffix: impl Into<SharedString>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }

    /// 入力欄の下に補足の文 `text` を表示する
    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting_text = Some(text.into());
        self
    }

    /// `error` なら、入力欄をエラーの色で表示する
    ///
    /// エラーであることは支援技術に公開しない。理由は [`TextField::supporting_text`] で伝えること
    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    /// テキストフィールドを無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効なテキストフィールドは編集も選択もできず、Tab でも到達できず、支援技術には disabled として公開される
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 文字列を編集させないようにする。`false` を渡すと編集できるように戻す
    ///
    /// 読み取り専用のテキストフィールドも、フォーカス・カーソルの移動・選択・コピーはでき、支援技術には read-only として公開される
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// 文字列を編集する操作をされたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には編集した後の文字列を渡す。表示を変えるには、`handler` で呼び出し側の状態を更新し、
    /// 次の描画で [`TextField::new`] に渡すこと。渡さなかった場合、編集は取り消される。
    /// 設定しない場合、テキストフィールドは文字列を編集させず、支援技術には read-only として公開される
    pub fn on_change(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl RenderOnce for TextField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let focus_handle = util::keyed_focus_handle(&self.id, window, cx);
        let field = Field {
            variant: match self.variant {
                TextFieldVariant::Filled => FieldVariant::Filled,
                TextFieldVariant::Outlined => FieldVariant::Outlined,
            },
            group: GROUP,
            label: self.label.clone(),
            leading_icon: self.leading_icon,
            trailing_icon: self.trailing_icon,
            flip_trailing_icon: false,
            prefix: self.prefix,
            suffix: self.suffix,
            supporting_text: self.supporting_text.clone(),
            error: self.error,
            disabled: self.disabled,
            focused: focus_handle.is_focused(window),
            populated: !self.value.is_empty(),
            theme,
        };
        let colors = field.input_colors();

        let mut input = HeadlessTextField::new(self.id.clone(), self.value.clone())
            .track_focus(&focus_handle.clone().tab_index(0))
            .disabled(self.disabled)
            .read_only(self.read_only)
            .caret_color(colors.caret)
            .selection_color(colors.selection)
            .w_full();
        if let Some(name) = self.label.clone().or(self.aria_label.clone()) {
            input = input.aria_label(name);
        }
        if let Some(placeholder) = &self.placeholder {
            input = input.placeholder(placeholder.clone());
        }
        if let Some(text) = &self.supporting_text {
            input = input.aria_description(text.clone());
        }
        if let Some(on_change) = self.on_change {
            input = input.on_change(on_change);
        }
        let placeholder = self
            .placeholder
            .filter(|_| self.value.is_empty() && field.input_visible())
            .map(|placeholder| {
                div()
                    .debug_selector(|| "md3-text-field-placeholder".into())
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .overflow_hidden()
                    .text_color(colors.placeholder)
                    .child(placeholder)
            });
        let input = div()
            .relative()
            .w_full()
            .children(placeholder)
            .child(input)
            .into_any_element();

        // 文の外を押してもフォーカスを移す。文の上の押下はテキストフィールドが受け、ここには届かない
        let disabled = self.disabled;
        let container = div().group(GROUP).when(!disabled, |this| {
            this.cursor(CursorStyle::IBeam).on_mouse_down(
                MouseButton::Left,
                move |_, window, cx| {
                    window.focus(&focus_handle, cx);
                },
            )
        });
        let mut outer = field.render(container, Some(input), window);
        outer.style().refine(&self.style);
        outer
    }
}

impl Styled for TextField {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

#[cfg(test)]
mod tests {
    use gpui::{
        Context, Hsla, Modifiers, Render, TestAppContext, VisualTestContext, point, px, size,
    };
    use md3_theme::md3_color::ColorRole;
    use md3_theme::md3_tokens::component::filled_text_field as filled;

    use super::*;

    /// (100, 100) に置いた幅 300 のテキストフィールド
    struct TextFieldView {
        value: SharedString,
        variant: TextFieldVariant,
        label: Option<&'static str>,
        placeholder: Option<&'static str>,
        prefix: Option<&'static str>,
        suffix: Option<&'static str>,
        error: bool,
        disabled: bool,
    }

    impl Render for TextFieldView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div().size_full().child(
                TextField::new("field", self.value.clone())
                    .variant(self.variant)
                    .when_some(self.label, |this, label| this.label(label))
                    .when_some(self.placeholder, |this, placeholder| {
                        this.placeholder(placeholder)
                    })
                    .when_some(self.prefix, |this, prefix| this.prefix(prefix))
                    .when_some(self.suffix, |this, suffix| this.suffix(suffix))
                    .error(self.error)
                    .disabled(self.disabled)
                    .on_change(cx.listener(|this, value: &SharedString, _, cx| {
                        this.value = value.clone();
                        cx.notify();
                    }))
                    .absolute()
                    .left(px(100.))
                    .top(px(100.))
                    .w(px(300.)),
            )
        }
    }

    fn add_view(
        cx: &mut TestAppContext,
        f: impl FnOnce(&mut TextFieldView),
    ) -> (gpui::Entity<TextFieldView>, &mut VisualTestContext) {
        cx.update(gpui_headless::init);
        let (view, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            let mut view = TextFieldView {
                value: SharedString::default(),
                variant: TextFieldVariant::Filled,
                label: Some("Name"),
                placeholder: None,
                prefix: None,
                suffix: None,
                error: false,
                disabled: false,
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

    fn value(view: &gpui::Entity<TextFieldView>, cx: &mut VisualTestContext) -> SharedString {
        view.read_with(cx, |view, _| view.value.clone())
    }

    /// 入力欄の文の外 (ラベルの右) を押す
    fn click_field(cx: &mut VisualTestContext) {
        cx.simulate_click(point(px(380.), px(128.)), Modifiers::none());
    }

    #[gpui::test]
    fn 入力欄を押すとフォーカスし入力した文字列を求める(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = add_view(cx, |_| {});
        cx.simulate_input("ab");
        assert_eq!(
            value(&view, cx),
            "",
            "フォーカスが無い間は入力を受けないこと"
        );
        click_field(cx);
        cx.simulate_input("ab");
        assert_eq!(value(&view, cx), "ab");
    }

    #[gpui::test]
    fn filled_の入力欄は高さ_56_で容器を塗りフォーカスで線を_primary_で太くする(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_view(cx, |_| {});
        let container = color(cx, filled::CONTAINER_COLOR);
        assert_eq!(filled(cx, container), [(100., 100., 300., 56.)]);
        let line = color(cx, filled::ACTIVE_INDICATOR_COLOR);
        assert_eq!(filled(cx, line), [(100., 155., 300., 1.)]);
        click_field(cx);
        let primary = color(cx, filled::FOCUS_ACTIVE_INDICATOR_COLOR);
        assert!(filled(cx, primary).contains(&(100., 154., 300., 2.)));
    }

    #[gpui::test]
    fn フォーカスの間はカーソルを_caret_の色で描く(cx: &mut TestAppContext) {
        let (_, cx) = add_view(cx, |_| {});
        click_field(cx);
        let caret = color(cx, filled::CARET_COLOR);
        // ラベルを上端から 8 に浮かせた下に、bodyLarge の行の高さ 24 の文を置く
        assert!(filled(cx, caret).contains(&(116., 124., 2., 24.)));
    }

    #[gpui::test]
    fn エラーではカーソルを_error_の色で描く(cx: &mut TestAppContext) {
        let (_, cx) = add_view(cx, |view| view.error = true);
        click_field(cx);
        let caret = color(cx, filled::ERROR_FOCUS_CARET_COLOR);
        assert!(filled(cx, caret).contains(&(116., 124., 2., 24.)));
    }

    #[gpui::test]
    fn 無効なテキストフィールドは押してもフォーカスせず容器を薄く塗る(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = add_view(cx, |view| view.disabled = true);
        click_field(cx);
        cx.simulate_input("ab");
        assert_eq!(value(&view, cx), "");
        let container = cx.update(|_, cx| {
            cx.md3_theme()
                .color(filled::DISABLED_CONTAINER_COLOR)
                .opacity(filled::DISABLED_CONTAINER_OPACITY)
        });
        assert_eq!(filled(cx, container), [(100., 100., 300., 56.)]);
    }

    #[gpui::test]
    #[cfg(debug_assertions)]
    fn 空でフォーカスが無い間はラベルを中央に置きフォーカスで浮かせる(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_view(cx, |_| {});
        let (x, y, _, height) = bounds(cx, "md3-field-label").unwrap();
        assert_eq!((x, y, height), (116., 116., 24.));
        assert_eq!(bounds(cx, "md3-field-input"), None);
        click_field(cx);
        let (x, y, _, height) = bounds(cx, "md3-field-label").unwrap();
        assert_eq!((x, y, height), (116., 108., 16.));
        let (x, y, width, height) = bounds(cx, "md3-field-text").unwrap();
        assert_eq!((x, y, width, height), (116., 124., 268., 24.));
    }

    #[gpui::test]
    #[cfg(debug_assertions)]
    fn ラベルが無ければ空でフォーカスが無くても文の欄を中央に置く(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_view(cx, |view| {
            view.label = None;
            view.placeholder = Some("Search");
        });
        let (_, y, _, height) = bounds(cx, "md3-field-text").unwrap();
        assert_eq!((y, height), (116., 24.));
        assert!(bounds(cx, "md3-text-field-placeholder").is_some());
    }

    #[gpui::test]
    #[cfg(debug_assertions)]
    fn プレースホルダーはラベルを浮かせ文字列が空の間だけ表示する(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_view(cx, |view| view.placeholder = Some("Your name"));
        assert_eq!(bounds(cx, "md3-text-field-placeholder"), None);
        click_field(cx);
        let (x, y, ..) = bounds(cx, "md3-text-field-placeholder").unwrap();
        assert_eq!((x, y), (116., 124.));
        cx.simulate_input("a");
        assert_eq!(bounds(cx, "md3-text-field-placeholder"), None);
    }

    #[gpui::test]
    #[cfg(debug_assertions)]
    fn 接辞はラベルを浮かせている間だけ文の前後に_2_空けて置く(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_view(cx, |view| {
            view.prefix = Some("$");
            view.suffix = Some("kg");
        });
        assert_eq!(bounds(cx, "md3-field-prefix"), None);
        assert_eq!(bounds(cx, "md3-field-suffix"), None);
        click_field(cx);
        let (prefix_x, _, prefix_width, _) = bounds(cx, "md3-field-prefix").unwrap();
        let (text_x, _, text_width, _) = bounds(cx, "md3-field-text").unwrap();
        let (suffix_x, _, suffix_width, _) = bounds(cx, "md3-field-suffix").unwrap();
        assert_eq!(prefix_x, 116.);
        assert_eq!(text_x, prefix_x + prefix_width);
        assert_eq!(suffix_x, text_x + text_width);
        assert_eq!(suffix_x + suffix_width, 384.);
    }

    #[gpui::test]
    #[cfg(debug_assertions)]
    fn outlined_は値があればラベルを枠線の上辺に置く(cx: &mut TestAppContext) {
        let (_, cx) = add_view(cx, |view| {
            view.variant = TextFieldVariant::Outlined;
            view.value = "a".into();
        });
        let (x, y, _, height) = bounds(cx, "md3-field-label").unwrap();
        assert_eq!((x, y, height), (116., 100., 16.));
        let (x, y, _, height) = bounds(cx, "md3-field-text").unwrap();
        // 入力欄は浮かせたラベルの半分の高さ 8 だけ下に置き、文はその中央に置く
        assert_eq!((x, y, height), (116., 124., 24.));
    }

    #[gpui::test]
    fn エラーでも選択を_primary_の不透明度_0_4_で塗る(cx: &mut TestAppContext) {
        let (_, cx) = add_view(cx, |view| {
            view.error = true;
            view.value = "abc".into();
        });
        click_field(cx);
        cx.simulate_keystrokes("ctrl-a");
        let selection = color(cx, ColorRole::Primary).opacity(0.4);
        let quads = filled(cx, selection);
        assert_eq!(quads.len(), 1);
        assert_eq!((quads[0].0, quads[0].1, quads[0].3), (116., 124., 24.));
    }
}
