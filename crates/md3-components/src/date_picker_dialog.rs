// 容器の大きさ・形・色、ボタンの並べ方と余白は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/androidMain/kotlin/androidx/compose/material3/DatePickerDialog.android.kt
//         (`DatePickerDialog`、`DialogButtonsPadding`、`DialogButtonsMainAxisSpacing`、`DialogButtonsCrossAxisSpacing`)
//         compose/material3/material3/src/androidMain/res/values/strings.xml (`m3c_date_picker_title`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2023 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
//
// 幕の色とウィンドウの端からの余白は Dialog と同じく Material Web の値を使う
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 のデイトピッカーのダイアログ

use gpui::{
    AnyElement, App, ElementId, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    Refineable, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div,
    relative,
};
use gpui_headless::dialog::{Dialog as HeadlessDialog, DismissReason};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::{date_picker_modal, dialog};
use md3_theme::md3_tokens::{Dp, supplement};
use md3_theme::{SharedTheme, ThemeExt, dp};

use crate::shape::Shaped;

/// ボタンの並びの下の余白 (Compose の `DialogButtonsPadding` の bottom)
const BUTTONS_BOTTOM_PADDING: Dp = Dp(8.0);
/// ボタンの並びの末尾の余白 (Compose の `DialogButtonsPadding` の end)
const BUTTONS_END_PADDING: Dp = Dp(6.0);
/// 横に並ぶボタンの間隔 (Compose の `DialogButtonsMainAxisSpacing`)
const BUTTONS_MAIN_AXIS_SPACING: Dp = Dp(8.0);
/// 縦に並ぶボタンの間隔 (Compose の `DialogButtonsCrossAxisSpacing`)
const BUTTONS_CROSS_AXIS_SPACING: Dp = Dp(8.0);

type DismissHandler = Box<dyn Fn(&DismissReason, &mut Window, &mut App)>;

/// MD3 のデイトピッカーのダイアログ (Compose の `DatePickerDialog`)
///
/// 子に置いた [`DatePicker`](crate::DatePicker) を、確定するボタンと閉じるボタンと一緒にダイアログで表示する。
/// 開いているかどうかは呼び出し側が持ち、[`DatePickerDialog::new`] に毎回渡す。
/// 置いた場所のレイアウトには影響せず、開いている間は、ウィンドウ全体を覆う幕と、中央に置いたダイアログを、ほかの要素より手前に描く。
/// ウィンドウのルートの近くに置くこと。ウィンドウのルートを [`crate::root`] で包むと、開いている間はダイアログ以外を支援技術から隠す
///
/// 容器は角を丸めて SurfaceContainerHigh で塗る。子は容器の角丸で切り抜かないため、
/// [`DatePicker`](crate::DatePicker) は `bg(gpui::transparent_black())` で背景を透明にして置くこと。
/// 背景を塗ったまま置くと、上側の角丸の外まで塗られる
///
/// 幅は 360dp で、高さは 568dp とウィンドウの高さのうち小さい方までにする。
/// 子の内容が収まらない場合は、ボタンを残して子の内容を縦にスクロールする。
/// ボタンは右下に、閉じるボタン、確定するボタンの順に並べる
///
/// - 開いたとき、[`DatePickerDialog::initial_focus`] の要素か、ダイアログの中で Tab で到達できる最初の要素にフォーカスを移す
/// - Tab と Shift+Tab は、ダイアログの中だけを巡る。ボタンは、確定するボタン、閉じるボタンの順に巡る
/// - Escape と幕のクリックで、[`DatePickerDialog::on_dismiss`] のハンドラを呼ぶ
/// - 閉じたとき、開く前にフォーカスのあった要素へフォーカスを戻す
///
/// 支援技術には modal の `dialog` として公開される。名前は既定で Select date で、[`DatePickerDialog::aria_label`] で変えられる
#[derive(IntoElement)]
pub struct DatePickerDialog {
    id: ElementId,
    open: bool,
    on_dismiss: Option<DismissHandler>,
    children: Vec<AnyElement>,
    confirm_button: Option<AnyElement>,
    dismiss_button: Option<AnyElement>,
    aria_label: SharedString,
    initial_focus: Option<FocusHandle>,
    theme: Option<SharedTheme>,
}

impl DatePickerDialog {
    /// `open` なら開いた状態で表示するダイアログを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>, open: bool) -> Self {
        Self {
            id: id.into(),
            open,
            on_dismiss: None,
            children: Vec::new(),
            confirm_button: None,
            dismiss_button: None,
            aria_label: "Select date".into(),
            initial_focus: None,
            theme: None,
        }
    }

    /// Escape か幕のクリックで閉じるよう求められたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には、閉じるよう求めた操作を渡す。
    /// 閉じるには、`handler` で呼び出し側の状態を更新し、次の描画で [`DatePickerDialog::new`] に渡すこと。
    /// ボタンが押されたときには呼ばないため、ボタンのハンドラでも状態を更新すること
    pub fn on_dismiss(
        mut self,
        handler: impl Fn(&DismissReason, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }

    /// 選んだ日付を確定するボタンを置く。Text のボタンを渡すこと
    pub fn confirm_button(mut self, button: impl IntoElement) -> Self {
        self.confirm_button = Some(button.into_any_element());
        self
    }

    /// 選ばずに閉じるボタンを置く。Text のボタンを渡すこと
    pub fn dismiss_button(mut self, button: impl IntoElement) -> Self {
        self.dismiss_button = Some(button.into_any_element());
        self
    }

    /// ダイアログのアクセシブルな名前を `label` にする。既定は Select date
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = label.into();
        self
    }

    /// 開いたときに `focus_handle` の要素へフォーカスを移す
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

impl ParentElement for DatePickerDialog {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for DatePickerDialog {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let mut headless = HeadlessDialog::new(self.id, self.open);
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
        let scrim = div()
            .flex()
            .items_center()
            .justify_center()
            .p(dp(supplement::dialog::WINDOW_MARGIN))
            .bg(theme
                .color(ColorRole::Scrim)
                .opacity(supplement::dialog::SCRIM_OPACITY));
        // 角丸は描画時の大きさで決まるため、背景は Shaped で包んだ層に描く。
        // Compose の DatePickerDialog の容器は影を持たない (`DatePickerDefaults.TonalElevation` は Level0)
        let container = div()
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .bottom_0()
            .bg(theme.color(date_picker_modal::CONTAINER_COLOR));
        // Compose と同じく、ボタンを押し出さないよう子の内容だけを縮め、収まらない分は Dialog と同じくスクロールする
        let content = div()
            .id("content")
            .relative()
            .flex()
            .flex_col()
            .flex_shrink_1()
            .min_h_0()
            .overflow_y_scroll()
            .children(self.children);
        // `AlertDialogFlowRow`。確定するボタンを先に並べて向きを反転し、横では末尾に、折り返すと上に置く
        let mut buttons = div()
            .relative()
            .flex()
            .flex_none()
            .flex_row_reverse()
            .flex_wrap()
            .self_end()
            .pb(dp(BUTTONS_BOTTOM_PADDING))
            .pr(dp(BUTTONS_END_PADDING))
            .gap_x(dp(BUTTONS_MAIN_AXIS_SPACING))
            .gap_y(dp(BUTTONS_CROSS_AXIS_SPACING))
            .text_color(theme.color(dialog::ACTION_LABEL_TEXT_COLOR));
        buttons
            .text_style()
            .refine(&theme.text_style(dialog::ACTION_LABEL_TEXT_FONT));
        headless
            .aria_label(self.aria_label)
            .scrim(scrim)
            .relative()
            .flex()
            .flex_col()
            .justify_between()
            .w(dp(date_picker_modal::CONTAINER_WIDTH))
            .max_h(relative(1.))
            .whitespace_normal()
            .child(Shaped::new(
                container,
                theme.clone(),
                date_picker_modal::CONTAINER_SHAPE,
            ))
            // 高さは 568dp とウィンドウに収まる高さの小さい方までにするため、568dp の上限は内側の箱に付ける
            .child(
                div()
                    .relative()
                    .flex()
                    .flex_col()
                    .justify_between()
                    .flex_shrink_1()
                    .min_h_0()
                    .max_h(dp(date_picker_modal::CONTAINER_HEIGHT))
                    .child(content)
                    .child(
                        buttons
                            .children(self.confirm_button)
                            .children(self.dismiss_button),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Context, FocusHandle, Render, TestAppContext, VisualTestContext, px, size};

    use super::*;
    use crate::{DatePicker, DatePickerDisplayMode};
    use gpui::prelude::FluentBuilder;
    use gpui_headless::button::Button;
    use gpui_headless::calendar::Date;

    /// 開いたデイトピッカーのダイアログを置く。ボタンは Primary で塗った 60x40 の箱
    struct DialogView {
        open: bool,
        dismisses: usize,
        dismiss_clicks: usize,
        /// デイトピッカーの後に加える子の高さ
        extra: f32,
        /// デイトピッカーを文字の入力の表示にする
        input: bool,
        /// デイトピッカーに求められた日付
        changes: Vec<Option<Date>>,
        confirm_focus: FocusHandle,
        dismiss_focus: FocusHandle,
    }

    impl Render for DialogView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let primary = cx.md3_theme().color(ColorRole::Primary);
            crate::root(
                div().size_full().child(
                    DatePickerDialog::new("dialog", self.open)
                        .on_dismiss(cx.listener(|this, _, _, cx| {
                            this.dismisses += 1;
                            this.open = false;
                            cx.notify();
                        }))
                        .confirm_button(
                            Button::new("confirm")
                                .track_focus(&self.confirm_focus)
                                .w(px(60.))
                                .h(px(40.))
                                .bg(primary),
                        )
                        .dismiss_button(
                            Button::new("dismiss")
                                .track_focus(&self.dismiss_focus)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.dismiss_clicks += 1;
                                    cx.notify();
                                }))
                                .w(px(60.))
                                .h(px(40.))
                                .bg(primary),
                        )
                        .child(
                            DatePicker::new("picker", Date::new(2026, 11, 10))
                                .when(self.input, |this| {
                                    this.display_mode(DatePickerDisplayMode::Input)
                                })
                                .on_change(cx.listener(|this, date: &Option<Date>, _, cx| {
                                    this.changes.push(*date);
                                    cx.notify();
                                }))
                                .bg(gpui::transparent_black()),
                        )
                        .child(div().flex_none().h(px(self.extra))),
                ),
            )
        }
    }

    fn setup(
        cx: &mut TestAppContext,
        height: f32,
    ) -> (gpui::Entity<DialogView>, &mut VisualTestContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            DialogView {
                open: true,
                dismisses: 0,
                dismiss_clicks: 0,
                extra: 0.,
                input: false,
                changes: Vec::new(),
                confirm_focus: cx.focus_handle(),
                dismiss_focus: cx.focus_handle(),
            }
        });
        cx.update(|window, _| window.activate_window());
        cx.simulate_resize(size(px(800.), px(height)));
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

    /// Primary で塗った 60x40 のボタン
    fn buttons(cx: &mut VisualTestContext) -> Vec<(f32, f32, f32, f32)> {
        quads(cx, ColorRole::Primary)
            .into_iter()
            .filter(|quad| (quad.2, quad.3) == (60., 40.))
            .collect()
    }

    /// デイトピッカーの高さ。見出し 120dp、区切り線 1dp、月の行 56dp、曜日と 6 週の 336dp
    const PICKER_HEIGHT: f32 = 120. + 1. + 56. + 336.;

    #[gpui::test]
    fn 幅_360dp_で中央に置きボタンを右下に並べる(cx: &mut TestAppContext) {
        let (_, cx) = setup(cx, 700.);
        // デイトピッカーと 40dp のボタン、その下の 8dp の余白
        let height = PICKER_HEIGHT + 40. + 8.;
        let top = (700. - height) / 2.;
        let containers = quads(cx, date_picker_modal::CONTAINER_COLOR);
        assert_eq!(containers, [(220., top, 360., height)]);
        // 確定するボタンを末尾 (右から 6dp) に、閉じるボタンをその左 8dp に置く
        let right = 220. + 360. - 6.;
        let button_top = top + height - 8. - 40.;
        assert_eq!(
            buttons(cx),
            [
                (right - 60., button_top, 60., 40.),
                (right - 60. - 8. - 60., button_top, 60., 40.)
            ]
        );
    }

    #[gpui::test]
    fn 高さは_568dp_までにする(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, 1000.);
        view.update(cx, |view, cx| {
            view.extra = 100.;
            cx.notify();
        });
        cx.run_until_parked();
        let containers = quads(cx, date_picker_modal::CONTAINER_COLOR);
        assert_eq!(containers[0], (220., (1000. - 568.) / 2., 360., 568.));
        let buttons = buttons(cx);
        assert!(
            buttons
                .iter()
                .all(|button| button.1 == (1000. + 568.) / 2. - 8. - 40.),
            "ボタンをダイアログの下端に置くこと: {buttons:?}"
        );
    }

    #[gpui::test]
    fn 低いウィンドウでは日付をスクロールしてボタンを残す(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx, 400.);
        // 上下に 24dp ずつ空け、ボタンはダイアログの下端に残す
        let button_top = 400. - 24. - 8. - 40.;
        let buttons = buttons(cx);
        assert_eq!(buttons.len(), 2, "ボタンを 2 つ描くこと: {buttons:?}");
        assert!(
            buttons.iter().all(|button| button.1 == button_top),
            "ボタンをダイアログの下端に置くこと: {buttons:?}"
        );
    }

    #[gpui::test]
    fn ボタンは確定するボタン_閉じるボタンの順に巡り_escape_で閉じるよう求める(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, 600.);
        // 最初の要素から Shift+Tab で戻ると、最後の閉じるボタンに移る
        cx.simulate_keystrokes("shift-tab");
        let (confirm, dismiss) = view.read_with(cx, |view, _| {
            (view.confirm_focus.clone(), view.dismiss_focus.clone())
        });
        assert!(cx.update(|window, _| dismiss.is_focused(window)));
        cx.simulate_keystrokes("shift-tab");
        assert!(cx.update(|window, _| confirm.is_focused(window)));
        cx.simulate_keystrokes("escape");
        cx.run_until_parked();
        assert_eq!(view.read_with(cx, |view, _| view.dismisses), 1);
    }

    #[gpui::test]
    fn 文字の入力の表示で開くと日付を入れる欄にフォーカスを移す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, 700.);
        view.update(cx, |view, cx| {
            view.open = false;
            view.input = true;
            cx.notify();
        });
        cx.run_until_parked();
        view.update(cx, |view, cx| {
            view.open = true;
            cx.notify();
        });
        cx.run_until_parked();
        // 欄の 11/10/2026 の末尾に入れても 8 桁を超えるため、すべてを選択して置き換える
        if cfg!(target_os = "macos") {
            cx.simulate_keystrokes("cmd-a");
        } else {
            cx.simulate_keystrokes("ctrl-a");
        }
        cx.simulate_input("1");
        assert_eq!(view.read_with(cx, |view, _| view.changes.clone()), [None]);
    }
}
