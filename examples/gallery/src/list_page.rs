//! リストを並べるページ

use gpui::{
    Context, IntoElement, ParentElement as _, Refineable as _, Render, Styled as _, Window, div,
    prelude::*, px,
};
use md3_components::{Divider, List, ListItem, ListItemKind};
use md3_theme::ThemeExt as _;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;

const FRUITS: [&str; 3] = ["Apple", "Pear", "Plum"];

/// 種類の違う項目のリストを並べるページ。チェックの状態と最後に実行した項目を持つ
#[derive(Default)]
pub struct ListPage {
    checked: [bool; 3],
    last_action: Option<&'static str>,
}

impl Render for ListPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let action = |id: &'static str, cx: &mut Context<Self>| {
            ListItem::new(id, id)
                .kind(ListItemKind::Action)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.last_action = Some(id);
                    cx.notify();
                }))
        };
        let actions = List::new("actions")
            .aria_label("Actions")
            .w(px(360.))
            .item(
                action("Inbox", cx)
                    .leading_icon(md3_icons::INBOX)
                    .trailing_supporting_text("100+"),
            )
            .item(
                action("Starred", cx)
                    .leading_icon(md3_icons::STAR)
                    .supporting_text("Messages you starred")
                    .trailing_icon(md3_icons::CHEVRON_RIGHT),
            )
            .child(Divider::new())
            .item(
                action("Settings", cx)
                    .leading_icon(md3_icons::SETTINGS)
                    .overline("Account")
                    .supporting_text("Notifications, privacy and storage"),
            )
            .item(
                action("Delete", cx)
                    .leading_icon(md3_icons::DELETE)
                    .disabled(true),
            );
        let checkboxes = List::new("checkboxes")
            .aria_label("Toppings")
            .w(px(360.))
            .items(FRUITS.iter().enumerate().map(|(ix, name)| {
                let checked = self.checked[ix];
                ListItem::new(("checkbox", ix), *name)
                    .kind(ListItemKind::Checkbox(checked))
                    .trailing_icon(if checked {
                        md3_icons::CHECK_BOX
                    } else {
                        md3_icons::CHECK_BOX_OUTLINE_BLANK
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.checked[ix] = !this.checked[ix];
                        cx.notify();
                    }))
            }));
        let texts = List::new("texts")
            .aria_label("Details")
            .w(px(360.))
            .item(ListItem::new("name", "Material Design 3").overline("Name"))
            .item(
                ListItem::new("person", "Contact")
                    .leading_icon(md3_icons::PERSON)
                    .supporting_text("A list item that cannot be operated"),
            );

        div()
            .id("list-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::SurfaceContainer))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_wrap()
            .content_start()
            .items_start()
            .gap_6()
            .child(div().flex().flex_col().gap_2().child(actions).child({
                let mut status = div().child(format!(
                    "Last action: {}",
                    self.last_action.unwrap_or("none")
                ));
                status
                    .text_style()
                    .refine(&theme.text_style(TypeScale::BodyMedium));
                status
            }))
            .child(checkboxes)
            .child(texts)
    }
}
