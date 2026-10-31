//! 入力欄の中身の操作と変更の通知。
use gpui::{AppContext, Focusable, TestAppContext};
use md3_components::inputs::TextFieldState;

#[gpui::test]
fn clones_share_the_text(cx: &mut TestAppContext) {
    cx.update(|cx| {
        let state = TextFieldState::with_text("あ", cx);
        let copy = state.clone();
        copy.set_text("い", cx);
        assert_eq!(state.text(cx), "い");
        assert_eq!(state.focus_handle(cx), copy.focus_handle(cx));
    });
}

#[gpui::test]
fn on_change_runs_for_each_change(cx: &mut TestAppContext) {
    let state = cx.update(TextFieldState::new);
    let view = cx.new(|cx| {
        state
            .on_change(cx, |count: &mut usize, _| *count += 1)
            .detach();
        0usize
    });
    cx.update(|cx| state.set_text("a", cx));
    cx.update(|cx| state.set_text("b", cx));
    assert_eq!(view.read_with(cx, |count, _| *count), 2);
}
