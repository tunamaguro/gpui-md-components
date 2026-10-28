//! 入れ物の部品のページ。

use gpui::{
    AnyElement, App, IntoElement, SharedString, UniformListScrollHandle, WeakEntity, Window, div,
    prelude::*, px,
};
use md3_components::buttons::{Button, ButtonVariant};
use md3_components::containment::{
    Card, CardVariant, Divider, LazyList, List, ListItem, ListLeading, ListTrailing,
};
use md3_components::theme::md3_color::ColorRole;
use md3_components::theme::{Md3ThemeExt, TypeScale};
use md3_components::{IconName, Text};

use super::{STATES, Section, state_header, state_row};
use crate::{Demo, Gallery};

/// 見本の小見出しと中身を縦に並べる。
fn sample(label: &str, element: impl IntoElement) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            Text::new(TypeScale::LabelMedium, label.to_owned()).color(ColorRole::OnSurfaceVariant),
        )
        .child(element)
        .into_any_element()
}

/// カードの中身の見本。
fn card_body(title: &str) -> impl IntoElement {
    div()
        .p_4()
        .flex()
        .flex_col()
        .gap_1()
        .child(Text::new(TypeScale::TitleMedium, title.to_owned()))
        .child(
            Text::new(TypeScale::BodyMedium, "週末の旅行の予定と持ち物の一覧。")
                .color(ColorRole::OnSurfaceVariant),
        )
}

/// LazyList の見本の項目数。
const CONTACTS: usize = 10_000;

/// 入れ物の部品のページ。
pub fn page(
    demo: &Demo,
    contacts: &UniformListScrollHandle,
    view: WeakEntity<Gallery>,
    _window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    // カード
    let variants = [
        ("Elevated", CardVariant::Elevated),
        ("Filled", CardVariant::Filled),
        ("Outlined", CardVariant::Outlined),
    ];
    let static_cards =
        div()
            .flex()
            .flex_wrap()
            .gap_4()
            .children(variants.map(|(name, variant)| {
                sample(
                    name,
                    Card::new(variant)
                        .w(px(240.0))
                        .child(card_body("旅行の計画")),
                )
            }));
    let clickable_rows: Vec<AnyElement> = variants
        .iter()
        .map(|(name, variant)| {
            let mut cells: Vec<AnyElement> = STATES
                .iter()
                .map(|(state, forced)| {
                    let view = view.clone();
                    let mut card = Card::new(*variant)
                        .on_click(
                            format!("card-{name}-{state}"),
                            "旅行の計画",
                            move |_, _, cx| {
                                Gallery::update_demo(&view, cx, |demo| demo.clicks += 1)
                            },
                        )
                        .w(px(140.0))
                        .h(px(80.0))
                        .child(div().p_3().child(Text::new(TypeScale::TitleSmall, "旅行")));
                    if let Some(forced) = forced {
                        card = card.forced(*forced);
                    }
                    card.into_any_element()
                })
                .collect();
            cells.push(
                Card::new(*variant)
                    .on_click(format!("card-{name}-disabled"), "旅行の計画", |_, _, _| {})
                    .disabled(true)
                    .w(px(140.0))
                    .h(px(80.0))
                    .child(div().p_3().child(Text::new(TypeScale::TitleSmall, "旅行")))
                    .into_any_element(),
            );
            state_row(*name, cells).into_any_element()
        })
        .collect();
    let cards = Section::new("Card")
        .child(static_cards)
        .child(Text::new(
            TypeScale::BodyMedium,
            format!("押せるカード（押した回数: {}）", demo.clicks),
        ))
        .child(state_header())
        .children(clickable_rows);

    // 区切り線
    let theme = cx.md3();
    let dividers = Section::new("Divider").child(
        div()
            .flex()
            .gap_6()
            .child(sample(
                "横線（全幅 / 左を 16dp 空ける）",
                div()
                    .w(px(320.0))
                    .flex()
                    .flex_col()
                    .bg(theme.color(ColorRole::SurfaceContainerLow))
                    .child(div().p_3().child("項目 1"))
                    .child(Divider::horizontal())
                    .child(div().p_3().child("項目 2"))
                    .child(Divider::horizontal().inset_start(px(16.0)))
                    .child(div().p_3().child("項目 3")),
            ))
            .child(sample(
                "縦線",
                div()
                    .h(px(48.0))
                    .flex()
                    .items_center()
                    .gap_3()
                    .child("左")
                    .child(Divider::vertical())
                    .child("右"),
            )),
    );

    // リスト
    let select = |index: usize| {
        let view = view.clone();
        move |_: &mut Window, cx: &mut App| {
            Gallery::update_demo(&view, cx, |demo| demo.list_selected = Some(index))
        }
    };
    let selected = |index: usize| demo.list_selected == Some(index);
    let list = List::new("mail", "メール")
        .item(
            "inbox",
            ListItem::new("受信トレイ")
                .leading(ListLeading::Icon(IconName::Inbox))
                .trailing(ListTrailing::Text("24".into()))
                .selected(selected(0))
                .on_select(select(0)),
        )
        .item(
            "sakura",
            ListItem::new("青木 さくら")
                .supporting_text("来週の打ち合わせの件")
                .leading(ListLeading::Avatar("青".into()))
                .trailing(ListTrailing::Text("10:24".into()))
                .selected(selected(1))
                .on_select(select(1)),
        )
        .item(
            "daisuke",
            ListItem::new("井上 大輔")
                .overline("プロジェクト")
                .supporting_text("資料を共有しました。確認をお願いします。差分は前回のコメントを反映したものです。")
                .leading(ListLeading::Avatar("井".into()))
                .trailing(ListTrailing::Icon(IconName::Star))
                .selected(selected(2))
                .on_select(select(2)),
        )
        .item(
            "archived",
            ListItem::new("アーカイブ（無効）")
                .leading(ListLeading::Icon(IconName::Delete))
                .disabled(true)
                .on_select(select(3)),
        )
        .item(
            "settings",
            ListItem::new("設定")
                .leading(ListLeading::Icon(IconName::Settings))
                .trailing(ListTrailing::Icon(IconName::ChevronRight))
                .selected(selected(4))
                .on_select(select(4)),
        );
    let item_states: Vec<AnyElement> = STATES
        .iter()
        .map(|(state, forced)| {
            let mut item = ListItem::new("受信トレイ")
                .leading(ListLeading::Icon(IconName::Inbox))
                .on_select(|_, _| {});
            if let Some(forced) = forced {
                item = item.forced(*forced);
            }
            sample(
                state,
                div()
                    .w(px(220.0))
                    .child(List::new(format!("list-state-{state}"), "見本").item("a", item)),
            )
        })
        .chain([
            sample(
                "Selected",
                div().w(px(220.0)).child(
                    List::new("list-state-selected", "見本").item(
                        "a",
                        ListItem::new("受信トレイ")
                            .leading(ListLeading::Icon(IconName::Inbox))
                            .selected(true)
                            .on_select(|_, _| {}),
                    ),
                ),
            ),
            sample(
                "Disabled",
                div().w(px(220.0)).child(
                    List::new("list-state-disabled", "見本").item(
                        "a",
                        ListItem::new("受信トレイ")
                            .leading(ListLeading::Icon(IconName::Inbox))
                            .disabled(true)
                            .on_select(|_, _| {}),
                    ),
                ),
            ),
        ])
        .collect();
    let lists = Section::new("List（↑↓ / Home / End で移る、Enter / Space で選ぶ）")
        .child(
            div()
                .w(px(420.0))
                .bg(theme.color(ColorRole::Surface))
                .child(list),
        )
        .child(div().flex().flex_wrap().gap_4().children(item_states));

    let lazy = Section::new(format!(
        "LazyList（{CONTACTS} 件。見えている項目だけを描く）"
    ))
    .child(
        div()
            .w(px(420.0))
            .h(px(320.0))
            .border_1()
            .border_color(theme.color(ColorRole::OutlineVariant))
            .child(
                LazyList::new("contacts", "連絡先", CONTACTS, contacts, {
                    let view = view.clone();
                    let selected = demo.contact;
                    move |index, _| {
                        let view = view.clone();
                        (
                            SharedString::from(format!("contact-{index}")),
                            ListItem::new(format!("連絡先 {}", index + 1))
                                .supporting_text(format!("user{index}@example.com"))
                                .leading(ListLeading::Avatar(
                                    format!("{}", char::from(b'A' + (index % 26) as u8)).into(),
                                ))
                                .selected(selected == Some(index))
                                .on_select(move |_, cx| {
                                    Gallery::update_demo(&view, cx, |demo| {
                                        demo.contact = Some(index)
                                    })
                                }),
                        )
                    }
                })
                .size_full(),
            ),
    )
    .child(
        div().child(
            Button::new("contacts-top", "先頭へ")
                .variant(ButtonVariant::Text)
                .on_click({
                    let contacts = contacts.clone();
                    move |_, _, _| contacts.scroll_to_item(0, gpui::ScrollStrategy::Top)
                }),
        ),
    );

    div()
        .flex()
        .flex_col()
        .gap_8()
        .child(cards)
        .child(dividers)
        .child(lists)
        .child(lazy)
}
