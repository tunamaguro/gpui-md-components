//! md3-components のコンポーネントを並べて表示するギャラリー。
// Web 版は `start` から起動するので、wasm では `main` を置かない。
#![cfg_attr(target_family = "wasm", no_main)]

mod button_page;
mod theme_page;

use gpui::{
    AnyView, App, AppContext as _, Context, InteractiveElement as _, IntoElement, KeyBinding,
    ParentElement as _, Render, Styled as _, Window, WindowOptions, actions, div,
};
use md3_components::button::{Button, ButtonVariant};
use md3_theme::ThemeExt as _;
use md3_theme::md3_color::ColorRole;

use crate::button_page::ButtonPage;
use crate::theme_page::ThemePage;

actions!(
    gallery,
    [
        /// 次の Tab stop へフォーカスを移す
        FocusNext,
        /// 前の Tab stop へフォーカスを移す
        FocusPrevious
    ]
);

/// ギャラリーのルート。表示中のページを選び、各ページのエンティティを持つ
struct Gallery {
    pages: Vec<(&'static str, AnyView)>,
    selected: usize,
}

impl Gallery {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            pages: vec![
                ("テーマ", cx.new(|_| ThemePage).into()),
                ("ボタン", cx.new(|_| ButtonPage::new()).into()),
            ],
            selected: 0,
        }
    }
}

impl Render for Gallery {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let tabs = self.pages.iter().enumerate().map(|(index, (name, _))| {
            Button::new(("page", index), *name)
                .variant(if index == self.selected {
                    ButtonVariant::Tonal
                } else {
                    ButtonVariant::Text
                })
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.selected = index;
                    cx.notify();
                }))
        });
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.color(ColorRole::Surface))
            .on_action(|_: &FocusNext, window, cx| window.focus_next(cx))
            .on_action(|_: &FocusPrevious, window, cx| window.focus_prev(cx))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_1()
                    .px_4()
                    .py_1()
                    .bg(theme.color(ColorRole::SurfaceContainer))
                    .children(tabs),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .child(self.pages[self.selected].1.clone()),
            )
    }
}

fn run() {
    gpui_platform::application().run(|cx: &mut App| {
        example_support::load_fonts(cx);
        // gpui は Tab にフォーカスの移動を割り当てないため、アプリで割り当てる
        cx.bind_keys([
            KeyBinding::new("tab", FocusNext, None),
            KeyBinding::new("shift-tab", FocusPrevious, None),
        ]);
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
