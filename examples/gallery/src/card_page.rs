//! カードを並べるページ

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Refineable as _, Render,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div, px,
};
use md3_components::{Card, CardVariant};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

const VARIANTS: [(CardVariant, &str); 3] = [
    (CardVariant::Filled, "Filled"),
    (CardVariant::Elevated, "Elevated"),
    (CardVariant::Outlined, "Outlined"),
];

/// カードを並べるページ。押せるカードを押した回数と、最後に押したカードを持つ
#[derive(Default)]
pub struct CardPage {
    clicks: usize,
    last_clicked: Option<&'static str>,
}

impl Render for CardPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        // 見出しと本文を縦に並べたカードの中身。`id` はテキストのノードの id に使う
        let content = |id: &str, title: &str, body: &str| {
            [
                text(
                    &theme,
                    TypeScale::TitleMedium,
                    format!("{id}-title"),
                    title.to_owned(),
                ),
                text(
                    &theme,
                    TypeScale::BodyMedium,
                    format!("{id}-body"),
                    body.to_owned(),
                ),
            ]
        };

        div()
            .id("card-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_col()
            .items_start()
            .gap_6()
            .child(text(
                &theme,
                TypeScale::BodyMedium,
                "clicks".to_owned(),
                format!(
                    "押した回数: {} / 最後に押したカード: {}",
                    self.clicks,
                    self.last_clicked.unwrap_or("なし")
                ),
            ))
            .children(VARIANTS.map(|(variant, name)| {
                let card =
                    |id: &'static str| Card::new(id).variant(variant).w(px(240.)).p_4().gap_2();
                div()
                    .flex()
                    .items_start()
                    .gap_6()
                    .child(card(name).children(content(name, name, "Content only")))
                    .child({
                        let id = match variant {
                            CardVariant::Filled => "clickable-filled",
                            CardVariant::Elevated => "clickable-elevated",
                            CardVariant::Outlined => "clickable-outlined",
                        };
                        card(id)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.clicks += 1;
                                this.last_clicked = Some(id);
                                cx.notify();
                            }))
                            .children(content(id, &format!("{name} clickable"), "Click me"))
                    })
                    .child({
                        let id = match variant {
                            CardVariant::Filled => "disabled-filled",
                            CardVariant::Elevated => "disabled-elevated",
                            CardVariant::Outlined => "disabled-outlined",
                        };
                        card(id)
                            .on_click(|_, _, _| {})
                            .disabled(true)
                            .children(content(id, &format!("{name} disabled"), "Not available"))
                    })
            }))
    }
}

/// 書体 `scale` で描き、支援技術に公開されるテキスト
///
/// `id` は支援技術のノードの id に使うため、ページ内で重ならない値を渡し、テキストが変わっても同じ値を渡すこと
fn text(theme: &Theme, scale: TypeScale, id: String, text: String) -> gpui::Div {
    let mut element = div();
    element.text_style().refine(&theme.text_style(scale));
    element.child(gpui::text!(
        id = SharedString::from(id),
        SharedString::from(text)
    ))
}
