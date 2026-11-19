//! トップアプリバーを並べるページ

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Render,
    StatefulInteractiveElement as _, Styled as _, Window, div,
};
use md3_components::{IconButton, TopAppBar, TopAppBarTitleAlignment, TopAppBarVariant};
use md3_theme::ThemeExt as _;
use md3_theme::md3_color::ColorRole;

/// トップアプリバーを並べるページ
pub struct AppBarPage;

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

        div()
            .id("app-bar-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::SurfaceContainerHighest))
            .p_6()
            .flex()
            .flex_col()
            .gap_6()
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
