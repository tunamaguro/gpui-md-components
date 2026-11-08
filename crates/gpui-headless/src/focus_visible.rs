//! フォーカスがあることを表示すべきかの判定

use std::collections::HashSet;

use gpui::{App, DispatchPhase, FocusHandle, Global, MouseDownEvent, Window, WindowId};

/// 最後に押されたのがマウスボタンではなくキーだったウィンドウ
#[derive(Default)]
struct KeyboardWindows(HashSet<WindowId>);

impl Global for KeyboardWindows {}

/// キーの押下を記録する操作を登録する。[`crate::init`] から 1 回呼ぶ
pub(crate) fn init(cx: &mut App) {
    cx.set_global(KeyboardWindows::default());
    // action や要素のハンドラが処理したキーも記録するため、それらより前に受ける
    cx.intercept_keystrokes(|_, window, cx| set_keyboard(window, cx, true))
        .detach();
    cx.on_window_closed(|cx, window_id| {
        cx.global_mut::<KeyboardWindows>().0.remove(&window_id);
    })
    .detach();
}

/// ウィンドウ全体のマウスボタンの押下を記録する。[`crate::root`] が描画のたびに呼ぶ
pub(crate) fn listen(window: &mut Window) {
    window.on_mouse_event(|_: &MouseDownEvent, phase, window, cx| {
        if phase == DispatchPhase::Capture {
            set_keyboard(window, cx, false);
        }
    });
}

fn set_keyboard(window: &mut Window, cx: &mut App, keyboard: bool) {
    let window_id = window.window_handle().window_id();
    let windows = &mut cx.default_global::<KeyboardWindows>().0;
    let changed = if keyboard {
        windows.insert(window_id)
    } else {
        windows.remove(&window_id)
    };
    // マウスの移動の後は gpui の入力の種類が変わらず再描画されないため、表示が変わるここで描画し直す
    if changed {
        window.refresh();
    }
}

/// `focus_handle` にフォーカスがあることを表示すべきかどうか
///
/// `focus_handle` がフォーカスを持ち、このウィンドウで最後に押されたのがマウスボタンではなくキーだった場合に `true` を返す。
/// マウスの移動では変わらないため、Tab で移った後にマウスを動かしても `true` のまま。
/// クリックでフォーカスを得た場合は、キーが押されるまで `false` になる。
/// マウスボタンの押下は、ルートを [`crate::root`] で包んだウィンドウでだけ記録される
pub fn is_focus_visible(focus_handle: &FocusHandle, window: &Window, cx: &App) -> bool {
    focus_handle.is_focused(window)
        && cx
            .try_global::<KeyboardWindows>()
            .is_some_and(|windows| windows.0.contains(&window.window_handle().window_id()))
}

/// WCAG 2.4.7 Focus Visible の解説 (understanding/20/focus-visible.html) にある、
/// キーボードを使ったときにフォーカスを表示する操作モード
#[cfg(test)]
mod tests {
    use gpui::{
        Context, Entity, IntoElement, Modifiers, MouseButton, ParentElement, Render, Styled,
        TestAppContext, VisualTestContext, div, point, px,
    };

    use super::*;
    use crate::button::Button;
    use crate::test_window::add_test_window;

    struct ButtonView {
        focus_handle: FocusHandle,
    }

    impl Render for ButtonView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div().size_full().child(
                Button::new("button")
                    .track_focus(&self.focus_handle)
                    .size(px(50.)),
            )
        }
    }

    fn setup(cx: &mut TestAppContext) -> (Entity<ButtonView>, &mut VisualTestContext) {
        add_test_window(cx, |_, cx| ButtonView {
            focus_handle: cx.focus_handle(),
        })
    }

    fn focus_visible(view: &Entity<ButtonView>, cx: &mut VisualTestContext) -> bool {
        let focus_handle = view.read_with(cx, |view, _| view.focus_handle.clone());
        cx.update(|window, cx| is_focus_visible(&focus_handle, window, cx))
    }

    fn focused(view: &Entity<ButtonView>, cx: &mut VisualTestContext) -> bool {
        let focus_handle = view.read_with(cx, |view, _| view.focus_handle.clone());
        cx.update(|window, _| focus_handle.is_focused(window))
    }

    #[gpui::test]
    fn tab_で移ったフォーカスはマウスを動かしても表示し続ける(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        cx.simulate_keystrokes("tab");
        assert!(
            focus_visible(&view, cx),
            "Tab で移ったフォーカスを表示すること"
        );
        cx.simulate_mouse_move(point(px(30.), px(30.)), None, Modifiers::none());
        assert!(
            focus_visible(&view, cx),
            "マウスを動かしてもフォーカスを表示し続けること"
        );
    }

    #[gpui::test]
    fn 部品の外でマウスボタンを押すとフォーカスの表示が消える(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        cx.simulate_keystrokes("tab");
        cx.simulate_mouse_down(
            point(px(100.), px(100.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        assert!(focused(&view, cx), "部品の外を押してもフォーカスは残ること");
        assert!(
            !focus_visible(&view, cx),
            "マウスボタンを押すとフォーカスを表示しないこと"
        );
    }

    #[gpui::test]
    fn クリックで得たフォーカスはキーを押すまで表示しない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        cx.simulate_keystrokes("tab");
        cx.simulate_click(point(px(10.), px(10.)), Modifiers::none());
        assert!(focused(&view, cx), "クリックした部品にフォーカスがあること");
        assert!(
            !focus_visible(&view, cx),
            "クリックで得たフォーカスは表示しないこと"
        );
        cx.simulate_keystrokes("a");
        assert!(
            focus_visible(&view, cx),
            "キーを押すとフォーカスを表示すること"
        );
    }
}
