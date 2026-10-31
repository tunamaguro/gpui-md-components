//! UI テスト用の補助。

#![expect(
    dead_code,
    reason = "テストバイナリごとに使う補助が違い、どのバイナリにも未使用の項目が残る"
)]

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
    cx.update(md3_components::init);
    let (_, cx) = cx.add_window_view(|_, _| Harness {
        render: Box::new(render),
    });
    cx.update(|window, _| window.activate_window());
    draw(cx);
    cx
}

/// 描画してフレームを確定させる。次のフレームで行う処理（オーバーレイの最初のフォーカスなど）も済ませる。
pub fn draw(cx: &mut VisualTestContext) {
    for _ in 0..3 {
        cx.run_until_parked();
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            window.simulate_next_frame(cx);
        });
    }
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

/// Tab キーで次の要素へフォーカスを移す。
pub fn tab(cx: &mut VisualTestContext) {
    press(cx, "tab");
}
