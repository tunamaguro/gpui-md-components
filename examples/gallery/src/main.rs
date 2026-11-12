//! md3-components のコンポーネントを並べて表示するギャラリー。
// Web 版は `start` から起動するので、wasm では `main` を置かない。
#![cfg_attr(target_family = "wasm", no_main)]

mod badge_page;
mod button_page;
mod card_page;
mod checkbox_page;
mod divider_page;
mod fab_page;
mod icon_button_page;
mod progress_indicator_page;
mod radio_button_page;
mod switch_page;
mod theme_page;

use gpui::{
    AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render,
    Styled as _, Window, WindowOptions, div,
};
use md3_components::{Button, ButtonVariant};
use md3_theme::ThemeExt as _;
use md3_theme::md3_color::ColorRole;

use crate::badge_page::BadgePage;
use crate::button_page::ButtonPage;
use crate::card_page::CardPage;
use crate::checkbox_page::CheckboxPage;
use crate::divider_page::DividerPage;
use crate::fab_page::FabPage;
use crate::icon_button_page::IconButtonPage;
use crate::progress_indicator_page::ProgressIndicatorPage;
use crate::radio_button_page::RadioButtonPage;
use crate::switch_page::SwitchPage;
use crate::theme_page::ThemePage;

/// 表示するページ
#[derive(Clone, Copy, PartialEq)]
enum Page {
    Theme,
    Button,
    IconButton,
    Checkbox,
    RadioButton,
    Switch,
    Divider,
    ProgressIndicator,
    Badge,
    Card,
    Fab,
}

/// ギャラリーのルート。各ページの状態は持たず、ページのエンティティと表示中のページだけを持つ
struct Gallery {
    page: Page,
    theme_page: Entity<ThemePage>,
    button_page: Entity<ButtonPage>,
    icon_button_page: Entity<IconButtonPage>,
    checkbox_page: Entity<CheckboxPage>,
    radio_button_page: Entity<RadioButtonPage>,
    switch_page: Entity<SwitchPage>,
    divider_page: Entity<DividerPage>,
    progress_indicator_page: Entity<ProgressIndicatorPage>,
    badge_page: Entity<BadgePage>,
    card_page: Entity<CardPage>,
    fab_page: Entity<FabPage>,
}

impl Gallery {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            page: Page::Theme,
            theme_page: cx.new(|_| ThemePage),
            button_page: cx.new(|_| ButtonPage::default()),
            icon_button_page: cx.new(|_| IconButtonPage::default()),
            checkbox_page: cx.new(CheckboxPage::new),
            radio_button_page: cx.new(|_| RadioButtonPage::default()),
            switch_page: cx.new(|_| SwitchPage::default()),
            divider_page: cx.new(|_| DividerPage),
            progress_indicator_page: cx.new(|_| ProgressIndicatorPage::default()),
            badge_page: cx.new(|_| BadgePage),
            card_page: cx.new(|_| CardPage::default()),
            fab_page: cx.new(|_| FabPage::default()),
        }
    }
}

impl Render for Gallery {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tab = |page: Page, label: &'static str, cx: &mut Context<Self>| {
            Button::new(label)
                .variant(if self.page == page {
                    ButtonVariant::FilledTonal
                } else {
                    ButtonVariant::Text
                })
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.page = page;
                    cx.notify();
                }))
                .child(gpui::text!(label))
        };
        let page: AnyView = match self.page {
            Page::Theme => self.theme_page.clone().into(),
            Page::Button => self.button_page.clone().into(),
            Page::IconButton => self.icon_button_page.clone().into(),
            Page::Checkbox => self.checkbox_page.clone().into(),
            Page::RadioButton => self.radio_button_page.clone().into(),
            Page::Switch => self.switch_page.clone().into(),
            Page::Divider => self.divider_page.clone().into(),
            Page::ProgressIndicator => self.progress_indicator_page.clone().into(),
            Page::Badge => self.badge_page.clone().into(),
            Page::Card => self.card_page.clone().into(),
            Page::Fab => self.fab_page.clone().into(),
        };
        md3_components::root(
            div()
                .size_full()
                .flex()
                .flex_col()
                .bg(cx.md3_theme().color(ColorRole::Surface))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap_2()
                        .p_2()
                        .child(tab(Page::Theme, "Theme", cx))
                        .child(tab(Page::Button, "Button", cx))
                        .child(tab(Page::IconButton, "Icon button", cx))
                        .child(tab(Page::Checkbox, "Checkbox", cx))
                        .child(tab(Page::RadioButton, "Radio button", cx))
                        .child(tab(Page::Switch, "Switch", cx))
                        .child(tab(Page::Divider, "Divider", cx))
                        .child(tab(Page::ProgressIndicator, "Progress indicator", cx))
                        .child(tab(Page::Badge, "Badge", cx))
                        .child(tab(Page::Card, "Card", cx))
                        .child(tab(Page::Fab, "FAB", cx)),
                )
                .child(div().flex_1().min_h_0().child(page)),
        )
    }
}

fn run() {
    gpui_platform::application().run(|cx: &mut App| {
        example_support::load_fonts(cx);
        md3_components::init(cx);
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
