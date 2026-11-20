//! トップアプリバーを並べるページ

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Refineable as _, Render,
    ScrollHandle, StatefulInteractiveElement as _, Styled as _, Window, div, px,
};
use md3_components::{IconButton, TopAppBar, TopAppBarTitleAlignment, TopAppBarVariant};
use md3_theme::ThemeExt as _;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;

/// トップアプリバーを並べるページ
#[derive(Default)]
pub struct AppBarPage {
    /// バーの下でスクロールする内容の位置
    content_scroll: ScrollHandle,
}

impl Render for AppBarPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let bar = |id: &'static str, title: &'static str| {
            TopAppBar::new(id, title)
                .navigation_icon(IconButton::new("navigation", md3_icons::ARROW_BACK, "Back"))
                .child(IconButton::new(
                    "attach",
                    md3_icons::ATTACH_FILE,
                    "Attach file",
                ))
                .child(IconButton::new(
                    "more",
                    md3_icons::MORE_VERT,
                    "More options",
                ))
        };

        let mut content = div()
            .id("scrolled-content")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&self.content_scroll)
            // gpui は重なったスクロール領域をすべて動かすため、ページまで動かないように止める
            .on_scroll(|_, _, cx| cx.stop_propagation())
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface));
        content
            .text_style()
            .refine(&theme.text_style(TypeScale::BodyLarge));
        let content =
            content.children((1..=20).map(|i| div().px_4().py_3().child(format!("Item {i}"))));

        div()
            .id("app-bar-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::SurfaceContainerHighest))
            .p_6()
            .flex()
            .flex_col()
            .gap_6()
            .child(
                div()
                    .flex_none()
                    .h(px(280.0))
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .child(
                        bar("scrolled", "Scroll the list")
                            .scrolled(self.content_scroll.offset().y < px(0.0)),
                    )
                    .child(content),
            )
            .child(bar("small", "Small"))
            .child(bar("center", "Center aligned").title_alignment(TopAppBarTitleAlignment::Center))
            .child(bar("small-subtitle", "Small").subtitle("With a subtitle"))
            .child(
                TopAppBar::new("no-navigation", "Without navigation icon").child(IconButton::new(
                    "search",
                    md3_icons::SEARCH,
                    "Search",
                )),
            )
            .child(bar("medium", "Medium").variant(TopAppBarVariant::Medium))
            .child(
                bar("medium-subtitle", "Medium")
                    .variant(TopAppBarVariant::Medium)
                    .subtitle("With a subtitle"),
            )
            .child(bar("large", "Large").variant(TopAppBarVariant::Large))
            .child(
                bar("large-center", "Large centered")
                    .variant(TopAppBarVariant::Large)
                    .title_alignment(TopAppBarTitleAlignment::Center)
                    .subtitle("With a subtitle"),
            )
    }
}
