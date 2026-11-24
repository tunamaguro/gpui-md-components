//! MD3 の Full-screen dialog

use gpui::{
    AnyElement, App, ElementId, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, ScrollHandle, SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};
use gpui_headless::dialog::{Dialog as HeadlessDialog, DismissReason};
use md3_theme::md3_color::ColorRole;
use md3_theme::{SharedTheme, ThemeExt};

use crate::{IconButton, TopAppBar};

type DismissHandler = Box<dyn Fn(&DismissReason, &mut Window, &mut App)>;

/// MD3 の Full-screen dialog
///
/// 開いているかどうかは呼び出し側が持ち、[`FullScreenDialog::new`] に毎回渡す。
/// 置いた場所のレイアウトには影響せず、開いている間は、ウィンドウ全体を覆うダイアログを、ほかの要素より手前に描く。
/// ウィンドウのルートの近くに置くこと。ウィンドウのルートを [`crate::root`] で包むと、開いている間はダイアログ以外を支援技術から隠す
///
/// 上端に Small の [`TopAppBar`] を置き、ナビゲーションのアイコン (閉じるボタン)、見出し、確定するボタンを並べる。
/// その下の残りの高さに子の内容を置き、収まらない内容は縦にスクロールする。
/// 内容がバーの下にスクロールして入り込んでいる間は、バーをスクロール中の色で塗る ([`TopAppBar::scrolled`])。
/// 内容の余白と書体は呼び出し側が付ける。書体は置いた場所の text style を受け継ぐ
///
/// - 開いたとき、[`FullScreenDialog::initial_focus`] の要素か、ダイアログの中で Tab で到達できる最初の要素にフォーカスを移す。
///   Tab の順は、ナビゲーションのアイコン、確定するボタン、内容の順になる
/// - Tab と Shift+Tab は、ダイアログの中だけを巡る
/// - 修飾キーなしの Escape で、[`FullScreenDialog::on_dismiss`] のハンドラを呼ぶ
/// - 閉じたとき、フォーカスがダイアログの中にあれば、開く前にフォーカスのあった要素へ戻す
///
/// 支援技術には、見出しを名前とする modal の `dialog` として公開される
#[derive(IntoElement)]
pub struct FullScreenDialog {
    id: ElementId,
    open: bool,
    headline: SharedString,
    on_dismiss: Option<DismissHandler>,
    navigation_icon: Option<IconButton>,
    confirm_button: Option<AnyElement>,
    children: Vec<AnyElement>,
    initial_focus: Option<FocusHandle>,
    theme: Option<SharedTheme>,
}

impl FullScreenDialog {
    /// `open` なら開いた状態で表示する、`headline` を見出しにしたダイアログを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>, open: bool, headline: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            open,
            headline: headline.into(),
            on_dismiss: None,
            navigation_icon: None,
            confirm_button: None,
            children: Vec::new(),
            initial_focus: None,
            theme: None,
        }
    }

    /// Escape で閉じるよう求められたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には、閉じるよう求めた操作を渡す。
    /// 閉じるには、`handler` で呼び出し側の状態を更新し、次の描画で [`FullScreenDialog::new`] に渡すこと。
    /// ボタンが押されたときには呼ばないため、閉じるボタンと確定するボタンのハンドラでも状態を更新すること
    pub fn on_dismiss(
        mut self,
        handler: impl Fn(&DismissReason, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }

    /// 見出しの前に `icon` を置く。閉じるボタン ([`md3_icons::CLOSE`] のアイコン) を置くこと
    pub fn navigation_icon(mut self, icon: IconButton) -> Self {
        self.navigation_icon = Some(icon);
        self
    }

    /// バーの右端に、内容を確定して閉じるボタンとして `button` を置く。Text button を置くこと
    pub fn confirm_button(mut self, button: impl IntoElement) -> Self {
        self.confirm_button = Some(button.into_any_element());
        self
    }

    /// 開いたときに `focus_handle` の要素へフォーカスを移す
    ///
    /// `focus_handle` はダイアログの中の要素のフォーカスにすること
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

impl ParentElement for FullScreenDialog {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for FullScreenDialog {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
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
        let scroll = window
            .use_keyed_state((self.id, "scroll"), cx, |_, _| ScrollHandle::new())
            .read(cx)
            .clone();
        let mut bar = TopAppBar::new("bar", self.headline.clone())
            .scrolled(scroll.offset().y < px(0.))
            .theme(theme.clone());
        if let Some(icon) = self.navigation_icon {
            bar = bar.navigation_icon(icon);
        }
        if let Some(button) = self.confirm_button {
            bar = bar.child(button);
        }
        dialog
            .scrim(div())
            .aria_label(self.headline)
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            // ダイアログは置いた場所の文字のスタイルを受け継ぐため、折り返しを戻す
            .whitespace_normal()
            .child(div().flex_none().child(bar))
            .child(
                div()
                    .id("content")
                    .track_scroll(&scroll)
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    // スクロール領域の直下の子は縮むため、内容の高さを保つ箱に入れる
                    .child(div().flex().flex_col().flex_none().children(self.children)),
            )
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use gpui::{
        Context, FocusHandle, KeyDownEvent, KeyUpEvent, Keystroke, Render, ScrollDelta,
        ScrollEvent, TestAppContext, VisualTestContext, point, px,
    };

    use super::*;
    use gpui_headless::button::Button;

    /// 開いているかと、閉じるよう求められた操作と、押されたボタンを持ち、高さ 2000 の赤い内容を入れたダイアログを描くビュー
    struct DialogView {
        open: bool,
        events: Rc<RefCell<Vec<&'static str>>>,
        close: FocusHandle,
        confirm: FocusHandle,
        /// 内容の中のボタンのフォーカス
        field: FocusHandle,
    }

    impl Render for DialogView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let close = self.events.clone();
            let confirm = self.events.clone();
            div().size_full().child(
                FullScreenDialog::new("dialog", self.open, "New event")
                    .on_dismiss(cx.listener(|this, _: &DismissReason, _, cx| {
                        this.events.borrow_mut().push("dismiss");
                        this.open = false;
                        cx.notify();
                    }))
                    .navigation_icon(
                        IconButton::new("close", md3_icons::CLOSE, "Close")
                            .track_focus(&self.close)
                            .on_click(move |_, _, _| close.borrow_mut().push("close")),
                    )
                    .confirm_button(
                        Button::new("save")
                            .track_focus(&self.confirm)
                            .size(px(40.))
                            .on_click(move |_, _, _| confirm.borrow_mut().push("save")),
                    )
                    .child(div().flex_none().h(px(2000.)).bg(gpui::red()))
                    .child(Button::new("field").track_focus(&self.field).size(px(40.))),
            )
        }
    }

    fn setup(cx: &mut TestAppContext) -> (gpui::Entity<DialogView>, &mut VisualTestContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            DialogView {
                open: true,
                events: Rc::default(),
                close: cx.focus_handle(),
                confirm: cx.focus_handle(),
                field: cx.focus_handle(),
            }
        });
        cx.update(|window, cx| {
            window.activate_window();
            cx.set_reduce_motion(true);
        });
        cx.run_until_parked();
        (view, cx)
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

    /// 赤く塗った内容の (x, y)
    fn content(cx: &mut VisualTestContext) -> Option<(f32, f32)> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            window
                .painted_quads()
                .into_iter()
                .find(|quad| quad.background == gpui::red().into())
                .map(|quad| {
                    (
                        quad.bounds.origin.x.0 / scale,
                        quad.bounds.origin.y.0 / scale,
                    )
                })
        })
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

    #[gpui::test]
    fn バーの下に内容を置き_収まらない内容をスクロールする(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx);
        let size = cx.update(|window, _| window.viewport_size());
        let (width, height) = (size.width / px(1.), size.height / px(1.));
        assert!(
            quads(cx, ColorRole::Surface).contains(&(0., 0., width, height)),
            "ウィンドウ全体を Surface で塗ること"
        );
        assert!(quads(cx, ColorRole::SurfaceContainer).is_empty());
        // Small のトップアプリバーの高さは 64
        assert_eq!(content(cx), Some((0., 64.)));
        cx.simulate_event(ScrollEvent {
            position: point(px(500.), px(500.)),
            delta: ScrollDelta::Pixels(point(px(0.), px(-100.))),
            ..Default::default()
        });
        assert_eq!(content(cx), Some((0., -36.)));
        assert!(
            quads(cx, ColorRole::SurfaceContainer).contains(&(0., 0., width, 64.)),
            "スクロールするとバーをスクロール中の色で塗ること"
        );
    }

    #[gpui::test]
    fn 閉じるボタンから_tab_で確定するボタンへ移り_escape_で閉じるよう求める(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        let (close, confirm) =
            view.read_with(cx, |view, _| (view.close.clone(), view.confirm.clone()));
        assert!(
            cx.update(|window, _| close.is_focused(window)),
            "最初は閉じるボタン"
        );
        cx.simulate_keystrokes("tab");
        assert!(cx.update(|window, _| confirm.is_focused(window)));
        let field = view.read_with(cx, |view, _| view.field.clone());
        cx.simulate_keystrokes("tab");
        assert!(
            cx.update(|window, _| field.is_focused(window)),
            "確定するボタンの後は内容"
        );
        cx.simulate_keystrokes("shift-tab");
        press(cx, "space");
        cx.simulate_keystrokes("shift-tab");
        press(cx, "space");
        cx.simulate_keystrokes("escape");
        assert_eq!(
            *view.read_with(cx, |view, _| view.events.clone()).borrow(),
            ["save", "close", "dismiss"]
        );
        assert_eq!(content(cx), None, "閉じると描かない");
    }
}
