//! UI テスト用の補助。

#![allow(dead_code)]

use gpui::{
    AnyElement, Context, InputEvent, IntoElement, KeyDownEvent, KeyUpEvent, Keystroke, Render,
    TestAppContext, VisualTestContext, Window, div, prelude::*,
};

type RenderFn = Box<dyn Fn(&mut Window, &mut Context<Harness>) -> AnyElement>;

/// 部品を 1 つ描くだけのビュー。
pub struct Harness {
    render: RenderFn,
}

impl Render for Harness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().p_4().child((self.render)(window, cx))
    }
}

/// `render` で部品を描くウィンドウを開き、最初のフレームを描く。
pub fn open(
    cx: &mut TestAppContext,
    render: impl Fn(&mut Window, &mut Context<Harness>) -> AnyElement + 'static,
) -> &mut VisualTestContext {
    let (_, cx) = cx.add_window_view(|_, _| Harness {
        render: Box::new(render),
    });
    draw(cx);
    cx
}

/// 描画してフレームを確定させる。
pub fn draw(cx: &mut VisualTestContext) {
    cx.run_until_parked();
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
}

/// キーを押して離す（キーボードでのクリックは押して離したときに起きる）。
pub fn press(cx: &mut VisualTestContext, key: &'static str) {
    let keystroke = Keystroke::parse(key)
        .map_err(|error| error.to_string())
        .expect("valid keystroke");
    cx.update(|window, cx| {
        window.dispatch_event(
            KeyDownEvent {
                keystroke: keystroke.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(KeyUpEvent { keystroke }.to_platform_input(), cx);
    });
    draw(cx);
}

/// 次の要素へフォーカスを移す（Tab 相当。gpui は Tab を既定で割り当てないため直接呼ぶ）。
pub fn tab(cx: &mut VisualTestContext) {
    cx.update(|window, cx| window.focus_next(cx));
    draw(cx);
}
