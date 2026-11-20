// 配置と大きさは Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ModalBottomSheet.kt
//         (`ModalBottomSheet`)、SheetDefaults.kt (`BottomSheetDefaults`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2023 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 のモーダルのボトムシート

use gpui::{
    AnyElement, App, ElementId, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    Refineable, RenderOnce, SharedString, StatefulInteractiveElement, StyleRefinement, Styled,
    Window, div, relative,
};
use gpui_headless::dialog::{Dialog as HeadlessDialog, DismissReason};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::Dp;
use md3_theme::md3_tokens::component::{scrim, sheet_bottom as tokens};
use md3_theme::{SharedTheme, ThemeExt, dp};

use crate::shape::Shaped;

/// シートの最大幅 (Compose の `BottomSheetDefaults.SheetMaxWidth`)
const MAX_WIDTH: Dp = Dp(640.0);

type DismissHandler = Box<dyn Fn(&DismissReason, &mut Window, &mut App)>;

/// MD3 のモーダルのボトムシート (Compose の `ModalBottomSheet`)
///
/// 開いている間、ウィンドウ全体を幕で覆い、その下端の中央に子の要素を入れたシートを描く。置いた場所のレイアウトには影響しない。
/// シートは幅 640dp までウィンドウの幅に広がり、高さは子の内容に合わせてウィンドウの高さまで伸びる。
/// 上側の角を丸め、SurfaceContainerLow で塗る。子の内容がウィンドウの高さに収まらない場合は、シートの中が縦にスクロールする。
/// 余白は付けないため、子の要素で付けること。子はシートの角丸で切り抜かないため、
/// 上端の子に背景を付ける場合は、角丸の内側に収まるよう余白を空けること。
/// ドラッグで高さを変える操作と、途中まで開いた状態は持たない。
/// 開いているかどうかは呼び出し側が持ち、[`ModalBottomSheet::new`] に毎回渡す。閉じることは [`ModalBottomSheet::on_dismiss`] で求める。
/// シートがウィンドウを覆うと幕をクリックできないため、Escape を使えない利用者のために、子に閉じるボタンを置くこと
///
/// 開いている間はモーダルのダイアログとして振る舞う
///
/// - 開いたとき、[`ModalBottomSheet::initial_focus`] の要素か、シートの中で Tab で到達できる最初の要素にフォーカスを移す。
///   どちらも無ければシートにフォーカスを移す
/// - フォーカスが表示範囲の外の要素に移っても、シートの中はスクロールしない
/// - Tab と Shift+Tab はシートの中だけを巡る
/// - 修飾キーなしの Escape と、幕のクリックで閉じるよう求める
/// - 閉じたとき、フォーカスがシートの中にあれば、開く前にフォーカスのあった要素へ戻す
///
/// 支援技術には modal の `dialog` として、[`ModalBottomSheet::new`] の `label` を名前として公開される。
/// ウィンドウのルートを [`crate::root`] で包むと、開いている間はシート以外を支援技術から隠す。
/// ただし、一覧のように子を遅れてレイアウトする要素や、キャッシュされたビューの中に置くと、隠すかどうかの判定に含まれない。
/// ほかの部品がウィンドウの手前に描く要素 (開いているメニューなど) は隠れないため、開く前に閉じること。
/// アプリ全体のキー割り当てと、置いた場所の祖先のキーのハンドラは開いている間も働くため、ウィンドウのルートの近くに置くこと。
/// [`Styled`] で付けたスタイルはシートに適用する。シートの背景はシートの色の層で塗るため、背景を付けても色は変わらない
#[derive(IntoElement)]
pub struct ModalBottomSheet {
    id: ElementId,
    open: bool,
    label: SharedString,
    children: Vec<AnyElement>,
    on_dismiss: Option<DismissHandler>,
    initial_focus: Option<FocusHandle>,
    style: StyleRefinement,
    theme: Option<SharedTheme>,
}

impl ModalBottomSheet {
    /// `open` なら開いた状態で表示するボトムシートを作る
    ///
    /// `label` はシートのアクセシブルな名前にする。画面には表示しない。
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>, open: bool, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            open,
            label: label.into(),
            children: Vec::new(),
            on_dismiss: None,
            initial_focus: None,
            style: StyleRefinement::default(),
            theme: None,
        }
    }

    /// Escape と幕のクリックで閉じるよう求められたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には、閉じるよう求めた操作を渡す。閉じるには、`handler` で呼び出し側の状態を更新し、
    /// 次の描画で [`ModalBottomSheet::new`] に `false` を渡すこと。
    /// 設定しない場合、Escape と幕のクリックでは何も起きない
    pub fn on_dismiss(
        mut self,
        handler: impl Fn(&DismissReason, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }

    /// 開いたときに、Tab で到達できる最初の要素の代わりに `focus_handle` の要素へフォーカスを移す
    ///
    /// `focus_handle` はシートの中の要素のフォーカスにすること。
    /// 取り消せない操作を含むシートで、閉じるボタンにフォーカスしたいときなどに使う
    pub fn initial_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.initial_focus = Some(focus_handle.clone());
        self
    }

    /// 色と形に使うテーマを設定する。設定しない場合はアプリのテーマを使う
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl ParentElement for ModalBottomSheet {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Styled for ModalBottomSheet {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for ModalBottomSheet {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let mut dialog = HeadlessDialog::new(self.id.clone(), self.open);
        if let Some(handler) = self.on_dismiss {
            dialog = dialog.on_dismiss(handler);
        }
        if let Some(focus_handle) = &self.initial_focus {
            dialog = dialog.initial_focus(focus_handle);
        }
        if !self.open {
            return dialog;
        }
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        // 幕の色は Compose の `BottomSheetDefaults.ScrimColor`
        let scrim = div()
            .flex()
            .flex_col()
            .justify_end()
            .items_center()
            .bg(theme
                .color(scrim::CONTAINER_COLOR)
                .opacity(scrim::CONTAINER_OPACITY));
        // 角丸は描画時の大きさで決まるため、背景と影は Shaped で包んだ層に描く
        let container = Shaped::new(
            div()
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .bottom_0()
                .bg(theme.color(tokens::DOCKED_CONTAINER_COLOR))
                .shadow(theme.shadow(tokens::DOCKED_MODAL_CONTAINER_ELEVATION)),
            theme.clone(),
            tokens::DOCKED_CONTAINER_SHAPE,
        );
        let content = div()
            .id((self.id, "content"))
            .relative()
            .flex()
            .flex_col()
            .min_h_0()
            .overflow_y_scroll()
            // スクロール領域の直下の子は縮むため、内容の高さを保つ箱に入れる
            .child(div().flex().flex_col().flex_none().children(self.children));
        let mut dialog = dialog
            .scrim(scrim)
            .aria_label(self.label)
            .relative()
            .flex()
            .flex_col()
            .w_full()
            .max_w(dp(MAX_WIDTH))
            .max_h(relative(1.))
            // シートは置いた場所の文字のスタイルを受け継ぐため、折り返しを戻す
            .whitespace_normal()
            .text_color(theme.color(ColorRole::OnSurface))
            .child(container)
            .child(content);
        dialog.style().refine(&self.style);
        dialog
    }
}

#[cfg(test)]
mod tests {
    use gpui::{
        Context, FocusHandle, Modifiers, Render, TestAppContext, VisualTestContext, point, px,
    };

    use super::*;

    /// 開いているかどうかと、閉じるよう求められた操作を持ち、高さ 100 の内容を入れたシートを描くビュー
    struct SheetView {
        open: bool,
        dismissed: Vec<DismissReason>,
        /// シートの中のボタンのフォーカス
        inner: FocusHandle,
        /// シートの外のボタンのフォーカス
        outside: FocusHandle,
    }

    impl Render for SheetView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .child(
                    gpui_headless::button::Button::new("outside")
                        .track_focus(&self.outside)
                        .size(px(20.0)),
                )
                .child(
                    ModalBottomSheet::new("sheet", self.open, "Options")
                        .on_dismiss(cx.listener(|this, reason: &DismissReason, _, cx| {
                            this.dismissed.push(*reason);
                            this.open = false;
                            cx.notify();
                        }))
                        .child(
                            gpui_headless::button::Button::new("inner")
                                .track_focus(&self.inner)
                                .h(px(100.0)),
                        ),
                )
        }
    }

    fn setup(cx: &mut TestAppContext) -> (gpui::Entity<SheetView>, &mut VisualTestContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            SheetView {
                open: false,
                dismissed: Vec::new(),
                inner: cx.focus_handle(),
                outside: cx.focus_handle(),
            }
        });
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        (view, cx)
    }

    fn set_open(view: &gpui::Entity<SheetView>, cx: &mut VisualTestContext) {
        view.update(cx, |view, cx| {
            view.open = true;
            cx.notify();
        });
        cx.run_until_parked();
    }

    /// シートの色で塗った矩形 (x, y, 幅, 高さ)
    fn sheet_quads(cx: &mut VisualTestContext) -> Vec<(f32, f32, f32, f32)> {
        cx.update(|window, cx| {
            let scale = window.scale_factor();
            let color: gpui::Background =
                cx.md3_theme().color(tokens::DOCKED_CONTAINER_COLOR).into();
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

    #[gpui::test]
    fn 閉じている間は描かず_開くと内容の高さのシートを下端の中央に描く(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        assert!(sheet_quads(cx).is_empty());
        set_open(&view, cx);
        let size = cx.update(|window, _| window.viewport_size());
        let (width, height) = (f32::from(size.width), f32::from(size.height));
        let sheet_width = width.min(MAX_WIDTH.0);
        assert_eq!(
            sheet_quads(cx),
            vec![(
                (width - sheet_width) / 2.0,
                height - 100.0,
                sheet_width,
                100.0
            )]
        );
    }

    #[gpui::test]
    fn ウィンドウより高い内容は縮めずにシートの中でスクロールする(
        cx: &mut TestAppContext,
    ) {
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            TallView
        });
        cx.run_until_parked();
        let height = cx.update(|window, _| f32::from(window.viewport_size().height));
        let (sheet, content) = cx.update(|window, cx| {
            let scale = window.scale_factor();
            let sheet: gpui::Background =
                cx.md3_theme().color(tokens::DOCKED_CONTAINER_COLOR).into();
            let content: gpui::Background = cx.md3_theme().color(ColorRole::Primary).into();
            let heights = |color| {
                window
                    .painted_quads()
                    .into_iter()
                    .filter(|quad| quad.background == color)
                    .map(|quad| quad.bounds.size.height.0 / scale)
                    .collect::<Vec<_>>()
            };
            (heights(sheet), heights(content))
        });
        assert_eq!(sheet, vec![height], "シートはウィンドウの高さで止まること");
        assert_eq!(content, vec![height * 2.0], "内容は縮めないこと");
    }

    /// ウィンドウの 2 倍の高さの内容を入れた、開いたシートを描くビュー
    struct TallView;

    impl Render for TallView {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let height = window.viewport_size().height * 2.0;
            let color = cx.md3_theme().color(ColorRole::Primary);
            ModalBottomSheet::new("sheet", true, "Options").child(div().h(height).bg(color))
        }
    }

    #[gpui::test]
    fn 開くと中にフォーカスし_escape_と幕のクリックで閉じるよう求める(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        let (inner, outside) =
            view.read_with(cx, |view, _| (view.inner.clone(), view.outside.clone()));
        cx.update(|window, cx| window.focus(&outside, cx));
        set_open(&view, cx);
        assert!(cx.update(|window, _| inner.is_focused(window)));
        cx.simulate_keystrokes("tab");
        assert!(
            cx.update(|window, _| inner.is_focused(window)),
            "Tab でシートの外へ出ないこと"
        );
        cx.simulate_keystrokes("escape");
        cx.run_until_parked();
        assert!(
            cx.update(|window, _| outside.is_focused(window)),
            "閉じたら開く前のフォーカスに戻すこと"
        );

        set_open(&view, cx);
        cx.simulate_click(point(px(10.0), px(10.0)), Modifiers::none());
        cx.run_until_parked();
        assert_eq!(
            view.read_with(cx, |view, _| view.dismissed.clone()),
            vec![DismissReason::Escape, DismissReason::Scrim]
        );
    }
}
