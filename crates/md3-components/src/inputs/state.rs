//! テキスト入力の中身。

use gpui::{App, AppContext, Context, Entity, FocusHandle, Focusable, Subscription};

/// 入力欄の中身（文字、カーソル、変換中の範囲）。ビューが持ち、[`TextField`](super::TextField) などに渡す。
///
/// 複製しても同じ中身を指す。
#[derive(Clone)]
pub struct TextFieldState(Entity<gpui_headless::TextFieldState>);

impl TextFieldState {
    /// 空の状態を作る。
    pub fn new(cx: &mut App) -> Self {
        Self(cx.new(gpui_headless::TextFieldState::new))
    }

    /// 文字の入った状態を作る。
    pub fn with_text(text: impl Into<String>, cx: &mut App) -> Self {
        let text = text.into();
        Self(cx.new(|cx| gpui_headless::TextFieldState::new(cx).with_text(text)))
    }

    /// 入力されている文字列。
    pub fn text<'a>(&self, cx: &'a App) -> &'a str {
        self.0.read(cx).text()
    }

    /// 文字列を置き換え、カーソルを末尾に置く。
    pub fn set_text(&self, text: impl Into<String>, cx: &mut App) {
        self.0.update(cx, |state, cx| state.set_text(text, cx));
    }

    /// 文字が変わったとき（変換中の文字を含む）の処理を登録する。
    pub fn on_change<V: 'static>(
        &self,
        cx: &mut Context<V>,
        mut handler: impl FnMut(&mut V, &mut Context<V>) + 'static,
    ) -> Subscription {
        cx.subscribe(
            &self.0,
            move |view, _, _: &gpui_headless::TextFieldEvent, cx| handler(view, cx),
        )
    }

    pub(crate) fn entity(&self) -> &Entity<gpui_headless::TextFieldState> {
        &self.0
    }
}

impl Focusable for TextFieldState {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.0.read(cx).focus_handle(cx)
    }
}
