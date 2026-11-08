//! Tab と Shift+Tab によるフォーカスの移動

use gpui::{App, KeyBinding, Window, actions};

actions!(
    gpui_headless,
    [
        /// Tab で到達できる次の要素へフォーカスを移す
        FocusNext,
        /// Tab で到達できる前の要素へフォーカスを移す
        FocusPrevious,
    ]
);

/// Tab と Shift+Tab でフォーカスを移すキー割り当てと操作を登録する。[`crate::init`] から 1 回呼ぶ
///
/// Tab に [`FocusNext`]、Shift+Tab に [`FocusPrevious`] を割り当て、操作したウィンドウの中で
/// Tab で到達できる要素の間でフォーカスを移す。末尾の次は先頭に、先頭の前は末尾に戻る。
/// 何もフォーカスしていない状態からも移れる
pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("tab", FocusNext, None),
        KeyBinding::new("shift-tab", FocusPrevious, None),
    ]);
    cx.on_action(|_: &FocusNext, cx| move_focus(cx, Window::focus_next));
    cx.on_action(|_: &FocusPrevious, cx| move_focus(cx, Window::focus_prev));
}

fn move_focus(cx: &mut App, f: fn(&mut Window, &mut App)) {
    // 要素の action ハンドラは、何もフォーカスしていないとキー操作が届かないため、App 全体で受ける。
    // キー操作を処理している間はそのウィンドウを更新できないため、処理が終わってから移す
    let Some(window) = cx.active_window() else {
        return;
    };
    cx.defer(move |cx| {
        // 失敗するのは、移す前にウィンドウが閉じられた場合だけで、そのときは移す先もない
        let _ = window.update(cx, |_, window, cx| f(window, cx));
    });
}

/// APG の Keyboard Interface (content/practices/keyboard-interface/keyboard-interface-practice.html)
/// で期待される Tab と Shift+Tab の振る舞い
#[cfg(test)]
mod tests {
    use gpui::{
        Context, Entity, FocusHandle, IntoElement, ParentElement, Render, TestAppContext,
        VisualTestContext, div,
    };

    use super::*;
    use crate::button::Button;
    use crate::test_window::add_test_window;

    struct ButtonsView {
        handles: [FocusHandle; 2],
    }

    impl Render for ButtonsView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .child(Button::new("first").track_focus(&self.handles[0]))
                .child(Button::new("second").track_focus(&self.handles[1]))
        }
    }

    fn setup(cx: &mut TestAppContext) -> (Entity<ButtonsView>, &mut VisualTestContext) {
        add_test_window(cx, |_, cx| ButtonsView {
            handles: [cx.focus_handle(), cx.focus_handle()],
        })
    }

    fn focused(view: &Entity<ButtonsView>, cx: &mut VisualTestContext) -> Option<usize> {
        let handles = view.read_with(cx, |view, _| view.handles.clone());
        cx.update(|window, _| handles.iter().position(|handle| handle.is_focused(window)))
    }

    #[gpui::test]
    fn 何もフォーカスしていない状態から_tab_で最初の要素に移る(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), Some(0));
    }

    #[gpui::test]
    fn tab_で次の要素に_shift_tab_で前の要素に移る(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        cx.simulate_keystrokes("tab tab");
        assert_eq!(focused(&view, cx), Some(1));
        cx.simulate_keystrokes("shift-tab");
        assert_eq!(focused(&view, cx), Some(0));
    }

    #[gpui::test]
    fn 末尾から_tab_で先頭に戻る(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        cx.simulate_keystrokes("tab tab tab");
        assert_eq!(focused(&view, cx), Some(0));
    }
}
