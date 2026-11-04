//! md3-components のコンポーネントを並べて表示するギャラリー。
// Web 版は `start` から起動するので、wasm では `main` を置かない。
#![cfg_attr(target_family = "wasm", no_main)]

mod theme_page;

use gpui::{
    App, AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, Styled as _,
    Window, WindowOptions, div,
};
use md3_theme::ThemeExt as _;

use crate::theme_page::ThemePage;

/// ギャラリーのルート。各ページの状態は持たず、ページのエンティティだけを持つ
struct Gallery {
    theme_page: Entity<ThemePage>,
}

impl Gallery {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            theme_page: cx.new(|_| ThemePage),
        }
    }
}

impl Render for Gallery {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.theme_page.clone())
    }
}

fn run() {
    gpui_platform::application().run(|cx: &mut App| {
        example_support::load_fonts(cx);
        // 既定のテーマのうち、フォントだけを同梱のものにする。Web 版にはシステムのフォントがないため
        cx.update_md3_theme(|settings| {
            settings.fonts.brand = example_support::FONT_FAMILY.into();
            settings.fonts.plain = example_support::FONT_FAMILY.into();
        })
        .expect("changing only the fonts keeps the default settings resolvable");
        cx.open_window(WindowOptions::default(), |window, cx| {
            cx.init_md3_theme(window);
            cx.new(Gallery::new)
        })
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
