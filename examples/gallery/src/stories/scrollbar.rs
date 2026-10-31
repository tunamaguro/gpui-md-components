//! スクロールバーの比較。操作状態とフェードはポインタで確認する。
//! 隣り合う見本は設定を 1 つだけ変える。
use super::Section;
use gpui::{App, Axis, IntoElement, Window, div, prelude::*, px};
use md3_components::Text;
use md3_components::buttons::Button;
use md3_components::containment::{Card, CardVariant, List, ListItem};
use md3_components::inputs::TextField;
use md3_components::scrollbar::{ScrollArea, ScrollbarInteraction};
use md3_components::theme::TypeScale;

use md3_components::inputs::TextFieldState;
pub fn page(window: &mut Window, cx: &mut App) -> impl IntoElement {
    let mut samples = div().flex().flex_wrap().gap_4();
    for (id, label, axis, interaction, fade, count) in [
        (
            "scroll-drag",
            "縦：ドラッグ・ホバー・400ms 後フェード",
            Axis::Vertical,
            ScrollbarInteraction::Draggable,
            true,
            30,
        ),
        (
            "scroll-always",
            "縦：ドラッグ・常時表示",
            Axis::Vertical,
            ScrollbarInteraction::Draggable,
            false,
            30,
        ),
        (
            "scroll-passive",
            "縦：表示のみ（つかめない）・常時表示",
            Axis::Vertical,
            ScrollbarInteraction::Passive,
            false,
            30,
        ),
        (
            "scroll-horizontal",
            "横：ドラッグ・常時表示",
            Axis::Horizontal,
            ScrollbarInteraction::Draggable,
            false,
            30,
        ),
        (
            "scroll-fit",
            "内容が収まる：非表示",
            Axis::Vertical,
            ScrollbarInteraction::Draggable,
            true,
            1,
        ),
    ] {
        let content = div()
            .flex()
            .flex_none()
            .when(axis == Axis::Vertical, |this| this.flex_col())
            .children((0..count).map(|i| div().flex_none().p_3().child(format!("項目 {}", i + 1))));
        let scroller = ScrollArea::new(id)
            .w(px(240.0))
            .h(px(180.0))
            .axis(axis)
            .scrollbar_interaction(interaction)
            .scrollbar_fade(fade)
            .child(content);
        samples = samples.child(
            div()
                .flex()
                .flex_col()
                .w(px(240.0))
                .gap_2()
                .child(Text::new(TypeScale::LabelMedium, label))
                .child(scroller),
        );
    }
    Section::new("Scrollbar（操作はマウス、キー操作はコンテナ）")
        .child(samples)
        .child(keyboard_sample(window, cx))
}

/// ページと内側の領域が、キーボードフォーカスについて動く見本。
fn keyboard_sample(window: &mut Window, cx: &mut App) -> impl IntoElement {
    let state = window
        .use_keyed_state("keyboard-scroll-text", cx, |_, cx| TextFieldState::new(cx))
        .read(cx)
        .clone();
    let count = window.use_keyed_state("keyboard-scroll-count", cx, |_, _| 0usize);
    let selected = window.use_keyed_state("keyboard-scroll-selection", cx, |_, _| None::<usize>);
    let increment = |count: gpui::Entity<usize>| {
        move |_: &gpui::ClickEvent, _: &mut Window, cx: &mut App| {
            count.update(cx, |count, cx| {
                *count += 1;
                cx.notify();
            });
        }
    };
    let list = (0..12).fold(
        List::new("keyboard-scroll-list")
            .aria_label("候補")
            .max_height(px(120.0)),
        |list, i| {
            let selection = selected.clone();
            list.item(
                format!("item-{i}"),
                ListItem::new(format!("候補 {}", i + 1))
                    .selected(*selected.read(cx) == Some(i))
                    .on_select(move |_, cx| {
                        selection.update(cx, |selected, cx| {
                            *selected = Some(i);
                            cx.notify();
                        })
                    }),
            )
        },
    );
    let scroller = ScrollArea::new("keyboard-scroll")
        .w(px(320.0))
        .h(px(200.0))
        .child(
            div()
                .flex()
                .flex_col()
                .p_3()
                .gap_6()
                .child(
                    Button::new("keyboard-scroll-first", "先頭のボタン")
                        .on_click(increment(count.clone())),
                )
                .child(Button::new("keyboard-scroll-disabled", "無効なボタン").disabled(true))
                .child(
                    Card::new(CardVariant::Filled)
                        .on_click("keyboard-scroll-card", increment(count.clone()))
                        .aria_label("カード")
                        .child(div().p_4().child("押せるカード")),
                )
                .child(TextField::new("keyboard-scroll-field", &state).label("入力欄"))
                .child(list)
                .child(
                    Button::new("keyboard-scroll-last", "末尾のボタン")
                        .on_click(increment(count.clone())),
                ),
        );
    let row = ScrollArea::new("keyboard-scroll-horizontal")
        .w(px(320.0))
        .h(px(80.0))
        .axis(Axis::Horizontal)
        .child(
            div()
                .flex()
                .flex_none()
                .p_3()
                .gap_4()
                .children((0usize..8).map(|i| {
                    Button::new(
                        ("keyboard-scroll-horizontal-button", i),
                        format!("ボタン {}", i + 1),
                    )
                    .on_click(increment(count.clone()))
                })),
        );
    Section::new("キーボードフォーカスに追従")
        .child(Text::new(TypeScale::BodyMedium, "Tab / Shift+Tab でボタン・カード・入力欄を移動。リスト内は ↑↓ / Home / End。横の列もフォーカスに追従します。クリックではスクロールせず、フォーカスしたままホイールで離しても引き戻しません。").id("scrollbar-instructions"))
        .child(Text::new(TypeScale::BodySmall, format!("実行回数: {}", count.read(cx))))
        .child(scroller)
        .child(row)
}
