//! md3-components のコンポーネントを並べて表示するギャラリー。
// Web 版は `start` から起動するので、wasm では `main` を置かない。
#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    App, AppContext as _, Context, IntoElement, ParentElement as _, Render, Styled as _, Window,
    WindowOptions, div,
};

struct Gallery;

impl Render for Gallery {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(gpui::white())
            .text_color(gpui::black())
            .font_family(example_support::FONT_FAMILY)
            .child("Hello world")
    }
}

fn run() {
    gpui_platform::application().run(|cx: &mut App| {
        example_support::load_fonts(cx);
        cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| Gallery))
            .expect("failed to open window");
        cx.activate(true);
    });
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    run();
}

/// Web 版のエントリポイント。wasm の読み込み時に呼ばれる。
#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    run();
}
