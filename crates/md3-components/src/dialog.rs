// 内容の並べ方・余白・色・書体は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/AlertDialog.kt
//         (`BasicAlertDialog`、`AlertDialogImpl`、`AlertDialogContent`、`AlertDialogFlowRow`、`AlertDialogDefaults`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
//
// 幕の色、ウィンドウの端からの余白、収まらない本文のスクロールは Material Web から移植している
// (https://github.com/material-components/material-web)
// Source: dialog/internal/_dialog.scss、dialog/internal/dialog.ts
//         @ de5470f7ed6a653457ca8f5cc0e7f7602c7a8391
// Copyright 2023 Google LLC. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 の Basic dialog

use gpui::{
    AnyElement, App, ElementId, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    Refineable, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, relative, svg,
};
use gpui_headless::dialog::Dialog as HeadlessDialog;
pub use gpui_headless::dialog::{DialogKind, DismissReason};
use md3_icons::StyledIcon;

use crate::shape::Shaped;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::dialog;
use md3_theme::md3_tokens::supplement;
use md3_theme::{SharedTheme, ThemeExt, dp};

type DismissHandler = Box<dyn Fn(&DismissReason, &mut Window, &mut App)>;

/// MD3 の Basic dialog
///
/// 開いているかどうかは呼び出し側が持ち、[`Dialog::new`] に毎回渡す。
/// 置いた場所のレイアウトには影響せず、開いている間は、ウィンドウ全体を覆う幕と、中央に置いたダイアログを、ほかの要素より手前に描く。
/// ウィンドウのルートの近くに置くこと。ウィンドウのルートを [`crate::root`] で包むと、開いている間はダイアログ以外を支援技術から隠す
///
/// 上から、アイコン、見出し、本文 ([`Dialog::supporting_text`] と子の内容)、ボタンを並べる。
/// 幅は内容に合わせて 280dp から 560dp にし、ウィンドウに収まらない本文は縦にスクロールする
///
/// - 開いたとき、[`Dialog::initial_focus`] の要素か、ダイアログの中で Tab で到達できる最初の要素にフォーカスを移す。
///   Tab の順は、確定するボタン、閉じるボタンの順になる
/// - Tab と Shift+Tab は、ダイアログの中だけを巡る
/// - Escape と幕のクリックで、[`Dialog::on_dismiss`] のハンドラを呼ぶ
/// - 閉じたとき、開く前にフォーカスのあった要素へフォーカスを戻す
///
/// 支援技術には modal の `dialog` か `alertdialog` として公開される。名前は見出しか [`Dialog::aria_label`] で与えること
#[derive(IntoElement)]
pub struct Dialog {
    id: ElementId,
    open: bool,
    on_dismiss: Option<DismissHandler>,
    kind: DialogKind,
    icon: Option<StyledIcon>,
    headline: Option<SharedString>,
    supporting_text: Option<SharedString>,
    children: Vec<AnyElement>,
    confirm_button: Option<AnyElement>,
    dismiss_button: Option<AnyElement>,
    aria_label: Option<SharedString>,
    initial_focus: Option<FocusHandle>,
    theme: Option<SharedTheme>,
}

impl Dialog {
    /// `open` なら開いた状態で表示するダイアログを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>, open: bool) -> Self {
        Self {
            id: id.into(),
            open,
            on_dismiss: None,
            kind: DialogKind::default(),
            icon: None,
            headline: None,
            supporting_text: None,
            children: Vec::new(),
            confirm_button: None,
            dismiss_button: None,
            aria_label: None,
            initial_focus: None,
            theme: None,
        }
    }

    /// Escape か幕のクリックで閉じるよう求められたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には、閉じるよう求めた操作を渡す。
    /// 閉じるには、`handler` で呼び出し側の状態を更新し、次の描画で [`Dialog::new`] に渡すこと。
    /// ボタンが押されたときには呼ばないため、ボタンのハンドラでも状態を更新すること
    pub fn on_dismiss(
        mut self,
        handler: impl Fn(&DismissReason, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }

    /// 支援技術に公開する role を決める種類を設定する。既定は [`DialogKind::Dialog`]
    ///
    /// [`DialogKind::Alert`] では、[`Dialog::supporting_text`] をダイアログの説明として公開する
    pub fn kind(mut self, kind: DialogKind) -> Self {
        self.kind = kind;
        self
    }

    /// 見出しの上に `icon` を表示する。見出しは中央にそろえる
    pub fn icon(mut self, icon: impl Into<StyledIcon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// 見出しを表示し、ダイアログの名前にする
    pub fn headline(mut self, headline: impl Into<SharedString>) -> Self {
        self.headline = Some(headline.into());
        self
    }

    /// 本文の先頭に `text` を表示する
    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting_text = Some(text.into());
        self
    }

    /// 提案した操作を確定するボタンを置く。横に並ぶときは末尾に、縦に並ぶときは上に置く
    ///
    /// ボタンが押されたときの処理は `button` に設定すること
    pub fn confirm_button(mut self, button: impl IntoElement) -> Self {
        self.confirm_button = Some(button.into_any_element());
        self
    }

    /// ダイアログを閉じるボタンを置く。横に並ぶときは確定するボタンの前に、縦に並ぶときは下に置く
    ///
    /// ボタンが押されたときの処理は `button` に設定すること
    pub fn dismiss_button(mut self, button: impl IntoElement) -> Self {
        self.dismiss_button = Some(button.into_any_element());
        self
    }

    /// 見出しの代わりに `label` をダイアログの名前にする
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// 開いたときに、Tab で到達できる最初の要素の代わりに `focus_handle` の要素へフォーカスを移す
    ///
    /// `focus_handle` はダイアログの中の要素のフォーカスにすること。
    /// 取り消せない操作を確認するダイアログで、閉じるボタンにフォーカスしたいときなどに使う
    pub fn initial_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.initial_focus = Some(focus_handle.clone());
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl ParentElement for Dialog {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Dialog {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let mut headless = HeadlessDialog::new(self.id, self.open).kind(self.kind);
        if let Some(handler) = self.on_dismiss {
            headless = headless.on_dismiss(handler);
        }
        if let Some(focus_handle) = &self.initial_focus {
            headless = headless.initial_focus(focus_handle);
        }
        if !self.open {
            return headless;
        }

        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        if let Some(label) = self.aria_label.or_else(|| self.headline.clone()) {
            headless = headless.aria_label(label);
        }
        if self.kind == DialogKind::Alert
            && let Some(text) = self.supporting_text.clone()
        {
            headless = headless.aria_description(text);
        }

        let scrim = div()
            .flex()
            .items_center()
            .justify_center()
            .p(dp(supplement::dialog::WINDOW_MARGIN))
            .bg(theme
                .color(ColorRole::Scrim)
                .opacity(supplement::dialog::SCRIM_OPACITY));
        // 角丸は描画時の大きさで決まるため、背景と影は Shaped で包んだ層に描く
        let container = div()
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .bottom_0()
            .bg(theme.color(dialog::CONTAINER_COLOR))
            .shadow(theme.shadow(dialog::CONTAINER_ELEVATION));
        headless = headless
            .scrim(scrim)
            .relative()
            .flex()
            .flex_col()
            .min_w(dp(supplement::dialog::MIN_WIDTH))
            .max_w(dp(supplement::dialog::MAX_WIDTH))
            .max_h(relative(1.))
            .p(dp(supplement::dialog::PADDING))
            // ダイアログは置いた場所の文字のスタイルを受け継ぐため、折り返しを戻す
            .whitespace_normal()
            .child(Shaped::new(
                container,
                theme.clone(),
                dialog::CONTAINER_SHAPE,
            ));

        if let Some(icon) = self.icon {
            headless = headless.child(
                svg()
                    .data(icon.svg().as_bytes())
                    .relative()
                    .flex_none()
                    .self_center()
                    .size(dp(dialog::ICON_SIZE))
                    .mb(dp(supplement::dialog::ICON_BOTTOM_PADDING))
                    .text_color(theme.color(dialog::ICON_COLOR)),
            );
        }
        if let Some(headline) = self.headline {
            let mut element = div()
                .relative()
                .flex_none()
                .mb(dp(supplement::dialog::TITLE_BOTTOM_PADDING))
                .text_color(theme.color(dialog::HEADLINE_COLOR))
                // アイコンがあるときは見出しを中央にそろえる
                .map(|this| {
                    if self.icon.is_some() {
                        this.self_center()
                    } else {
                        this.self_start()
                    }
                });
            element
                .text_style()
                .refine(&theme.text_style(dialog::HEADLINE_FONT));
            headless = headless.child(element.child(gpui::text!(id = "headline", headline)));
        }
        if self.supporting_text.is_some() || !self.children.is_empty() {
            // Compose は本文を残りの高さまで縮める。Material Web と同じく、収まらない分はスクロールする
            let mut content = div()
                .id("content")
                .relative()
                .flex()
                .flex_col()
                .overflow_y_scroll()
                .mb(dp(supplement::dialog::TEXT_BOTTOM_PADDING))
                .text_color(theme.color(dialog::SUPPORTING_TEXT_COLOR));
            content
                .text_style()
                .refine(&theme.text_style(dialog::SUPPORTING_TEXT_FONT));
            headless = headless.child(
                content
                    .children(
                        self.supporting_text
                            .map(|text| gpui::text!(id = "supporting-text", text)),
                    )
                    .children(self.children),
            );
        }
        if self.confirm_button.is_some() || self.dismiss_button.is_some() {
            // `AlertDialogFlowRow`。確定するボタンを先に並べて向きを反転し、横では末尾に、折り返すと上に置く
            let mut buttons = div()
                .relative()
                .flex()
                .flex_none()
                .flex_row_reverse()
                .flex_wrap()
                .gap_x(dp(supplement::dialog::BUTTONS_MAIN_AXIS_SPACING))
                .gap_y(dp(supplement::dialog::BUTTONS_CROSS_AXIS_SPACING))
                .text_color(theme.color(dialog::ACTION_LABEL_TEXT_COLOR));
            buttons
                .text_style()
                .refine(&theme.text_style(dialog::ACTION_LABEL_TEXT_FONT));
            headless = headless.child(
                buttons
                    .children(self.confirm_button)
                    .children(self.dismiss_button),
            );
        }
        headless
    }
}

#[cfg(test)]
mod tests {
    use gpui::{
        Context, FocusHandle, Hsla, Modifiers, Render, TestAppContext, VisualTestContext, point,
        px, size,
    };

    use super::*;
    use gpui_headless::button::Button;

    /// 800x600 のウィンドウに、左上に置いた開くボタンと、見出し・本文・2 つのボタンを持つダイアログを置く。
    /// ダイアログのボタンは Primary で塗った高さ 40 の箱で、幅を `button_width` にする
    struct DialogView {
        open: bool,
        icon: bool,
        button_width: f32,
        /// 本文の後に加える子の高さ
        content_height: f32,
        opener_focus: FocusHandle,
        confirm_focus: FocusHandle,
        dismiss_focus: FocusHandle,
    }

    impl Render for DialogView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let primary = cx.md3_theme().color(ColorRole::Primary);
            let dialog = Dialog::new("dialog", self.open)
                .on_dismiss(cx.listener(|this, _, _, cx| {
                    this.open = false;
                    cx.notify();
                }))
                .when(self.icon, |this| this.icon(md3_icons::DELETE))
                .headline("Discard draft?")
                .supporting_text("This can't be undone.")
                .when(self.content_height > 0., |this| {
                    this.child(div().flex_none().h(px(self.content_height)))
                })
                .confirm_button(
                    Button::new("confirm")
                        .track_focus(&self.confirm_focus)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.open = false;
                            cx.notify();
                        }))
                        .w(px(self.button_width))
                        .h(px(40.))
                        .bg(primary),
                )
                .dismiss_button(
                    Button::new("dismiss")
                        .track_focus(&self.dismiss_focus)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.open = false;
                            cx.notify();
                        }))
                        .w(px(self.button_width))
                        .h(px(40.))
                        .bg(primary),
                );
            crate::root(
                div()
                    .size_full()
                    .child(
                        Button::new("opener")
                            .track_focus(&self.opener_focus)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.open = true;
                                cx.notify();
                            }))
                            .size(px(20.)),
                    )
                    .child(dialog),
            )
        }
    }

    fn add_view(cx: &mut TestAppContext) -> (gpui::Entity<DialogView>, &mut VisualTestContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            DialogView {
                open: false,
                icon: false,
                button_width: 100.,
                content_height: 0.,
                opener_focus: cx.focus_handle(),
                confirm_focus: cx.focus_handle(),
                dismiss_focus: cx.focus_handle(),
            }
        });
        cx.update(|window, _| window.activate_window());
        cx.simulate_resize(size(px(800.), px(600.)));
        (view, cx)
    }

    fn update(
        view: &gpui::Entity<DialogView>,
        cx: &mut VisualTestContext,
        f: impl FnOnce(&mut DialogView),
    ) {
        view.update(cx, |view, cx| {
            f(view);
            cx.notify();
        });
        cx.run_until_parked();
    }

    fn color(cx: &mut VisualTestContext, role: ColorRole) -> Hsla {
        cx.update(|_, cx| cx.md3_theme().color(role))
    }

    /// 矩形の x, y, 幅, 高さと、左上の角丸の半径
    type Rect = ((f32, f32, f32, f32), f32);

    /// 背景を `color` で塗った矩形
    fn filled(cx: &mut VisualTestContext, color: Hsla) -> Vec<Rect> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            window
                .painted_quads()
                .iter()
                .filter(|quad| quad.background == color.into())
                .map(|quad| {
                    let b = quad.bounds;
                    (
                        (
                            b.origin.x.0 / scale,
                            b.origin.y.0 / scale,
                            b.size.width.0 / scale,
                            b.size.height.0 / scale,
                        ),
                        quad.corner_radii.top_left.0 / scale,
                    )
                })
                .collect()
        })
    }

    fn container(cx: &mut VisualTestContext) -> Vec<Rect> {
        let color = color(cx, ColorRole::SurfaceContainerHigh);
        filled(cx, color)
    }

    /// ダイアログの中のボタンの x と y
    fn buttons(cx: &mut VisualTestContext) -> Vec<(f32, f32)> {
        let color = color(cx, ColorRole::Primary);
        filled(cx, color)
            .into_iter()
            .map(|((x, y, _, _), _)| (x, y))
            .collect()
    }

    /// 期待値は Compose の `DialogMinWidth` (280dp)、`dialogPadding` (24dp)、見出し (HeadlineSmall、行の高さ 32sp) と
    /// その下の 16dp、本文 (BodyMedium、行の高さ 20sp) とその下の 24dp、ボタンの高さ 40、
    /// DialogTokens の `ContainerShape` (CornerExtraLarge、28dp) と `ContainerColor` (SurfaceContainerHigh)。
    /// 幕は Material Web の scrim の色の 32%
    #[gpui::test]
    fn 幕の中央に最小の幅で描く(cx: &mut TestAppContext) {
        let (view, cx) = add_view(cx);
        assert_eq!(container(cx), vec![]);
        update(&view, cx, |view| view.open = true);
        let height = 24. + 32. + 16. + 20. + 24. + 40. + 24.;
        assert_eq!(
            container(cx),
            vec![((260., (600. - height) / 2., 280., height), 28.)]
        );
        let scrim = color(cx, ColorRole::Scrim).opacity(0.32);
        assert_eq!(filled(cx, scrim), vec![((0., 0., 800., 600.), 0.)]);
    }

    /// 期待値は Compose の `AlertDialogFlowRow`。ボタンは末尾にそろえて間隔 8dp で並べ、
    /// 確定するボタン (先に描く) を末尾に置く。内容の右端は 260 + 280 - 24 = 516
    #[gpui::test]
    fn ボタンは末尾にそろえ確定するボタンを末尾に置く(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = add_view(cx);
        update(&view, cx, |view| view.open = true);
        let y = 210. + 24. + 32. + 16. + 20. + 24.;
        assert_eq!(buttons(cx), vec![(416., y), (308., y)]);
    }

    /// 幅 300 のボタン 2 つと間隔 8dp は、最大の幅 (`DialogMaxWidth`、560dp) の内容の幅 512 に収まらない
    #[gpui::test]
    fn 収まらないボタンは確定するボタンを上にして縦に並べる(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = add_view(cx);
        update(&view, cx, |view| {
            view.open = true;
            view.button_width = 300.;
        });
        let height = 180. + 8. + 40.;
        let top = (600. - height) / 2.;
        assert_eq!(container(cx), vec![((120., top, 560., height), 28.)]);
        let y = top + 24. + 32. + 16. + 20. + 24.;
        assert_eq!(buttons(cx), vec![(356., y), (356., y + 48.)]);
    }

    /// 期待値は DialogTokens の `IconSize` (24dp) と Compose の `IconPadding` (下に 16dp)
    #[gpui::test]
    fn アイコンを見出しの上に置く(cx: &mut TestAppContext) {
        let (view, cx) = add_view(cx);
        update(&view, cx, |view| {
            view.open = true;
            view.icon = true;
        });
        let height = 180. + 24. + 16.;
        assert_eq!(
            container(cx),
            vec![((260., (600. - height) / 2., 280., height), 28.)]
        );
    }

    /// 期待値は Material Web の `max-height: calc(100% - 48px)`。本文が縮み、ボタンは下端から 24dp に残る
    #[gpui::test]
    fn 収まらない高さはウィンドウの端から余白を空け本文を縮める(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = add_view(cx);
        update(&view, cx, |view| {
            view.open = true;
            view.content_height = 1000.;
        });
        assert_eq!(container(cx), vec![((260., 24., 280., 552.), 28.)]);
        assert_eq!(buttons(cx), vec![(416., 512.), (308., 512.)]);
    }

    #[gpui::test]
    fn 開くと最初のボタンにフォーカスを移し_escape_で閉じて戻す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = add_view(cx);
        cx.simulate_click(point(px(10.), px(10.)), Modifiers::none());
        assert!(view.read_with(cx, |view, _| view.open));
        let (opener, confirm) = view.read_with(cx, |view, _| {
            (view.opener_focus.clone(), view.confirm_focus.clone())
        });
        assert!(cx.update(|window, _| confirm.is_focused(window)));
        cx.simulate_keystrokes("escape");
        assert!(!view.read_with(cx, |view, _| view.open));
        assert!(cx.update(|window, _| opener.is_focused(window)));
    }

    #[gpui::test]
    fn 幕のクリックで閉じ_ダイアログの上のクリックでは閉じない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = add_view(cx);
        update(&view, cx, |view| view.open = true);
        cx.simulate_click(point(px(400.), px(240.)), Modifiers::none());
        assert!(view.read_with(cx, |view, _| view.open));
        cx.simulate_click(point(px(100.), px(500.)), Modifiers::none());
        assert!(!view.read_with(cx, |view, _| view.open));
    }
}
