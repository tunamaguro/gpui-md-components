//! ボタン類のページ。

use gpui::{App, IntoElement, WeakEntity, Window, div, prelude::*};
use md3_components::buttons::{
    Button, ButtonVariant, ExtendedFab, Fab, FabColor, FabSize, IconButton, IconButtonVariant,
    Segment, SegmentedButton,
};
use md3_components::theme::TypeScale;
use md3_components::{IconName, Text};

use super::{Section, state_header, state_row};
use crate::{Demo, Gallery};

/// ボタン類のページ。
pub fn page(
    demo: &Demo,
    view: WeakEntity<Gallery>,
    _window: &mut Window,
    _cx: &mut App,
) -> impl IntoElement {
    let buttons = ButtonVariant::ALL.map(|variant| {
        let cells = vec![
            Button::new(format!("button-{variant:?}-Enabled"), "Label")
                .variant(variant)
                .into_any_element(),
            Button::new(format!("button-{variant:?}-disabled"), "Label")
                .variant(variant)
                .disabled(true)
                .into_any_element(),
        ];
        state_row(format!("{variant:?}"), cells)
    });
    let icon_buttons = IconButtonVariant::ALL.map(|variant| {
        let cells = vec![
            IconButton::new(
                format!("icon-button-{variant:?}-Enabled"),
                IconName::Settings,
                "設定",
            )
            .variant(variant)
            .into_any_element(),
            IconButton::new(
                format!("icon-button-{variant:?}-disabled"),
                IconName::Settings,
                "設定",
            )
            .variant(variant)
            .disabled(true)
            .into_any_element(),
        ];
        state_row(format!("{variant:?}"), cells)
    });
    let toggles = IconButtonVariant::ALL.map(|variant| {
        let favorite = demo.favorite;
        let view = view.clone();
        div()
            .flex()
            .gap_2()
            .child(
                IconButton::new(
                    format!("toggle-{variant:?}"),
                    IconName::Favorite,
                    "お気に入り",
                )
                .variant(variant)
                .selected(favorite)
                .on_click(move |_, _, cx| {
                    Gallery::update_demo(&view, cx, |demo| demo.favorite = !demo.favorite)
                }),
            )
            .child(
                IconButton::new(
                    format!("toggle-{variant:?}-on"),
                    IconName::Favorite,
                    "お気に入り",
                )
                .variant(variant)
                .selected(true),
            )
            .child(
                IconButton::new(
                    format!("toggle-{variant:?}-off"),
                    IconName::Favorite,
                    "お気に入り",
                )
                .variant(variant)
                .selected(false),
            )
    });
    let fabs = FabColor::ALL.map(|color| {
        let mut cells: Vec<_> = FabSize::ALL
            .map(|size| {
                Fab::new(format!("fab-{color:?}-{size:?}"), IconName::Edit, "作成")
                    .size(size)
                    .color(color)
                    .into_any_element()
            })
            .into();
        cells.push(
            ExtendedFab::new(format!("extended-{color:?}"), "作成")
                .icon(IconName::Edit)
                .color(color)
                .into_any_element(),
        );
        state_row(format!("{color:?}"), cells)
    });

    let clicks = demo.clicks;
    let segment = demo.segment;
    let segments = demo.segments.clone();
    div()
        .flex()
        .flex_col()
        .gap_6()
        .child(
            Section::new("Button")
                .child(state_header())
                .children(buttons)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_4()
                        .child(
                            Button::new("button-icon", "追加")
                                .icon(IconName::Add)
                                .on_click({
                                    let view = view.clone();
                                    move |_, _, cx| {
                                        Gallery::update_demo(&view, cx, |demo| demo.clicks += 1)
                                    }
                                }),
                        )
                        .child(
                            Button::new("button-text-icon", "編集")
                                .variant(ButtonVariant::Text)
                                .icon(IconName::Edit),
                        )
                        .child(Text::new(
                            TypeScale::BodyMedium,
                            format!("押された回数: {clicks}"),
                        )),
                ),
        )
        .child(
            Section::new("Icon button")
                .child(state_header())
                .children(icon_buttons)
                .child(Text::new(
                    TypeScale::LabelLarge,
                    "Toggle（操作できる / 選択 / 非選択）",
                ))
                .child(div().flex().gap_6().children(toggles)),
        )
        .child(Section::new("FAB").children(fabs))
        .child(
            Section::new("Segmented button")
                .child(
                    SegmentedButton::new("segmented-single", "期間")
                        .segment(Segment::new("日"))
                        .segment(Segment::new("週"))
                        .segment(Segment::new("月"))
                        .segment(Segment::new("年").disabled(true))
                        .selected([segment])
                        .on_change({
                            let view = view.clone();
                            move |index, _, cx| {
                                Gallery::update_demo(&view, cx, |demo| demo.segment = index)
                            }
                        }),
                )
                .child(
                    SegmentedButton::new("segmented-multi", "サイズ")
                        .multi_select(true)
                        .segment(Segment::new("S"))
                        .segment(Segment::new("M"))
                        .segment(Segment::new("L"))
                        .segment(Segment::new("XL"))
                        .selected(segments)
                        .on_change(move |index, _, cx| {
                            Gallery::update_demo(&view, cx, |demo| {
                                if let Some(at) = demo.segments.iter().position(|&i| i == index) {
                                    demo.segments.remove(at);
                                } else {
                                    demo.segments.push(index);
                                }
                            })
                        }),
                ),
        )
}
