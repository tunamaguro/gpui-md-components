//! 検索バーを並べるページ

use gpui::{
    Context, FocusHandle, IntoElement, ParentElement as _, Refineable as _, Render, SharedString,
    Styled as _, Window, div, px,
};
use md3_components::{
    IconButton, List, ListItem, ListItemKind, SearchBar, SearchView, SearchViewVariant,
};
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

/// 検索の画面で候補にする語
const SUGGESTIONS: [&str; 6] = ["Apple", "Banana", "Cherry", "Grape", "Lemon", "Orange"];

/// 検索の画面を持つ検索の形。添字は [`SearchBarPage::views`] の位置
const VARIANTS: [SearchViewVariant; 2] = [SearchViewVariant::FullScreen, SearchViewVariant::Docked];

/// 検索バーを並べるページ。入力中の文字列と、最後に検索した文字列と、入力欄のフォーカスと、
/// 検索の画面を持つ検索ごとの文字列と開いているかを持つ
pub struct SearchBarPage {
    query: SharedString,
    last_search: Option<SharedString>,
    focus: FocusHandle,
    views: [(SharedString, bool); 2],
}

impl SearchBarPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            query: SharedString::default(),
            last_search: None,
            focus: cx.focus_handle(),
            views: Default::default(),
        }
    }
}

impl Render for SearchBarPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let mut bar = SearchBar::new("search", self.query.clone(), "Search mail")
            .placeholder("Search mail")
            .track_focus(&self.focus)
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
                    |this, _, window, cx| {
                        this.query = SharedString::default();
                        // 押したボタンは消えるため、入力欄へフォーカスを戻す
                        window.focus(&this.focus, cx);
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
            .child(self.search_view(0, cx))
            .child(self.search_view(1, cx))
    }
}

impl SearchBarPage {
    /// 開くと候補を並べる、[`VARIANTS`] の `index` 番目の形の検索
    fn search_view(&self, index: usize, cx: &mut Context<Self>) -> SearchView {
        let (value, expanded) = self.views[index].clone();
        let query = value.to_lowercase();
        let items = SUGGESTIONS
            .into_iter()
            .filter(|item| item.to_lowercase().contains(&query))
            .map(|item| {
                ListItem::new(item, item)
                    .kind(ListItemKind::Action)
                    .leading_icon(md3_icons::HISTORY)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.views[index] = (item.into(), false);
                        cx.notify();
                    }))
            });
        let (id, label): (&'static str, &'static str) = match VARIANTS[index] {
            SearchViewVariant::FullScreen => ("fruit-search", "Search fruits"),
            SearchViewVariant::Docked => ("docked-fruit-search", "Search fruits (docked)"),
        };
        SearchView::new(id, value, label, expanded)
            .variant(VARIANTS[index])
            .placeholder(label)
            .on_change(cx.listener(move |this, value: &SharedString, _, cx| {
                this.views[index].0 = value.clone();
                cx.notify();
            }))
            .on_search(cx.listener(move |this, _: &SharedString, _, cx| {
                this.views[index].1 = false;
                cx.notify();
            }))
            .on_expanded_change(cx.listener(move |this, expanded: &bool, _, cx| {
                this.views[index].1 = *expanded;
                cx.notify();
            }))
            .child(
                List::new("suggestions")
                    .aria_label("Suggestions")
                    .items(items),
            )
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
