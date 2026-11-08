//! 操作できる部品の hover とフォーカスを、別の要素から参照するための値

use std::sync::atomic::{AtomicU64, Ordering};

use gpui::{App, FocusHandle, SharedString, Window};

/// 操作できる部品の hover とフォーカスを、その部品の子孫の要素から参照するための値
///
/// 部品に渡すと、部品はこの値のフォーカスを自分のフォーカスとして使い、この値の group の名前を自分に付ける。
/// 子孫の要素は、[`InteractionSource::group`] を `group_hover` に渡すと部品の hover で、
/// [`InteractionSource::is_focus_visible`] で部品のフォーカスで見た目を変えられる。
/// 行全体を操作の対象にして、その中の一部の要素に hover やフォーカスを表示する場合に使う
///
/// 呼び出し側の状態として作って持ち、描画のたびに同じ値を渡すこと。1 つの値を渡す部品は 1 つにすること
#[derive(Debug, Clone)]
pub struct InteractionSource {
    focus_handle: FocusHandle,
    group: SharedString,
}

impl InteractionSource {
    /// 新しいフォーカスと、ほかの値と重ならない group の名前を持つ値を作る
    pub fn new(cx: &mut App) -> Self {
        // gpui の group は同じ名前を持つ祖先のうち最も内側のものを指すため、入れ子にしても取り違えないよう名前を分ける
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        Self {
            focus_handle: cx.focus_handle().tab_index(0).tab_stop(true),
            group: format!("gpui-headless-interaction-{id}").into(),
        }
    }

    /// 部品が使うフォーカス
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    /// 部品に付く group の名前
    pub fn group(&self) -> &SharedString {
        &self.group
    }

    /// 部品がフォーカスを持っているかどうか
    pub fn is_focused(&self, window: &Window) -> bool {
        self.focus_handle.is_focused(window)
    }

    /// 部品にフォーカスがあることを表示すべきかどうか。判定は [`crate::is_focus_visible`] と同じ
    pub fn is_focus_visible(&self, window: &Window, cx: &App) -> bool {
        crate::is_focus_visible(&self.focus_handle, window, cx)
    }
}

#[cfg(test)]
mod tests {
    use gpui::{
        Context, IntoElement, Modifiers, ParentElement, Render, Styled, TestAppContext, div, point,
        px,
    };

    use super::*;
    use crate::choice::{Checkbox, CheckedState};
    use crate::test_window::add_test_window;

    struct RowView {
        source: InteractionSource,
    }

    impl Render for RowView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div().size_full().child(
                Checkbox::new("row", CheckedState::Unchecked)
                    .interaction_source(&self.source)
                    .on_change(|_, _, _| {})
                    .size(px(50.)),
            )
        }
    }

    #[gpui::test]
    fn tab_で移ると部品のフォーカスとして表示される(cx: &mut TestAppContext) {
        let (view, cx) = add_test_window(cx, |_, cx| RowView {
            source: InteractionSource::new(cx),
        });
        cx.simulate_keystrokes("tab");
        let source = view.read_with(cx, |view, _| view.source.clone());
        cx.update(|window, cx| {
            assert!(source.is_focused(window));
            assert!(source.is_focus_visible(window, cx));
        });
    }

    #[gpui::test]
    fn クリックではフォーカスを得るが表示しない(cx: &mut TestAppContext) {
        let (view, cx) = add_test_window(cx, |_, cx| RowView {
            source: InteractionSource::new(cx),
        });
        cx.simulate_click(point(px(10.), px(10.)), Modifiers::none());
        let source = view.read_with(cx, |view, _| view.source.clone());
        cx.update(|window, cx| {
            assert!(source.is_focused(window));
            assert!(!source.is_focus_visible(window, cx));
        });
    }

    struct TwoRowsView {
        sources: [InteractionSource; 2],
    }

    impl Render for TwoRowsView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .child(
                    Checkbox::new("first", CheckedState::Unchecked)
                        .interaction_source(&self.sources[0])
                        .on_change(|_, _, _| {})
                        .size(px(50.)),
                )
                .child(
                    Checkbox::new("second", CheckedState::Unchecked)
                        .interaction_source(&self.sources[1])
                        .on_change(|_, _, _| {})
                        .size(px(50.)),
                )
        }
    }

    #[gpui::test]
    fn クリックした後に_tab_で次の部品へ移る(cx: &mut TestAppContext) {
        let (view, cx) = add_test_window(cx, |_, cx| TwoRowsView {
            sources: [InteractionSource::new(cx), InteractionSource::new(cx)],
        });
        cx.simulate_click(point(px(10.), px(10.)), Modifiers::none());
        cx.simulate_keystrokes("tab");
        let sources = view.read_with(cx, |view, _| view.sources.clone());
        cx.update(|window, _| {
            assert!(
                !sources[0].is_focused(window),
                "最初の部品からフォーカスが外れること"
            );
            assert!(
                sources[1].is_focused(window),
                "次の部品にフォーカスが移ること"
            );
        });
    }

    #[gpui::test]
    fn 別々に作った値は_group_の名前が重ならない(cx: &mut TestAppContext) {
        let (first, second) =
            cx.update(|cx| (InteractionSource::new(cx), InteractionSource::new(cx)));
        assert_ne!(first.group(), second.group());
    }
}
