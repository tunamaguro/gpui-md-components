//! 検索バーを並べるページ

use gpui::{
    Context, IntoElement, ParentElement as _, Refineable as _, Render, SharedString, Styled as _,
    Window, div, px,
};
use md3_components::{IconButton, SearchBar};
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

/// 検索バーを並べるページ。入力中の文字列と、最後に検索した文字列を持つ
pub struct SearchBarPage {
    query: SharedString,
    last_search: Option<SharedString>,
}

impl SearchBarPage {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            query: SharedString::default(),
            last_search: None,
        }
    }
}

impl Render for SearchBarPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let mut bar = SearchBar::new("search", self.query.clone(), "Search mail")
            .placeholder("Search mail")
            .on_change(cx.listener(|this, value: &SharedString, _, cx| {
                this.query = value.clone();
                cx.notify();
            }))
            .on_search(cx.listener(|this, value: &SharedString, _, cx| {
                this.last_search = Some(value.clone());
                cx.notify();
            }));
        // 文字列があれば消すボタンを、無ければアカウントのボタンを末尾に置く
        bar = if self.query.is_empty() {
            bar.child(IconButton::new(
                "account",
                md3_icons::ACCOUNT_CIRCLE,
                "Account",
            ))
        } else {
            bar.child(
                IconButton::new("clear", md3_icons::CLOSE, "Clear search").on_click(cx.listener(
                    |this, _, _, cx| {
                        this.query = SharedString::default();
                        cx.notify();
                    },
                )),
            )
        };
        let status = match &self.last_search {
            Some(query) => format!("Searched: {query}"),
            None => "Press Enter to search".to_owned(),
        };

        div()
            .flex()
            .flex_col()
            .items_start()
            .gap(px(16.))
            .p_6()
            .child(bar)
            .child(label(&theme, "search-status", status))
    }
}

/// 支援技術に公開されるテキスト
fn label(theme: &Theme, id: &'static str, text: impl Into<SharedString>) -> gpui::Div {
    let mut element = div();
    element
        .text_style()
        .refine(&theme.text_style(TypeScale::BodyMedium));
    element.child(gpui::text!(id = id, text.into()))
}
