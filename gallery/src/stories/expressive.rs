//! M3 Expressive（ToggleButton、ButtonGroup）のページ。

use gpui::{App, IntoElement, WeakEntity, Window, div, prelude::*};
use md3_components::expressive::{ButtonGroup, ToggleButton};
use md3_components::theme::md3_color::ColorRole;
use md3_components::theme::{Md3ThemeExt, MotionScheme, TypeScale};
use md3_components::{IconName, Text};

use super::{STATES, Section, state_header, state_row};
use crate::{Demo, Gallery};

/// M3 Expressive のページ。
pub fn page(
    demo: &Demo,
    view: WeakEntity<Gallery>,
    _window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let motion = match (cx.md3().motion_scheme(), cx.reduce_motion()) {
        (_, true) => "Reduce motion: バネは動かず、最終の形で描く",
        (MotionScheme::Standard, false) => "Motion: Standard（行き過ぎの少ないバネ）",
        (MotionScheme::Expressive, false) => "Motion: Expressive（形と幅が弾むバネ）",
    };

    let toggle_rows = [false, true].map(|checked| {
        let name = if checked { "Checked" } else { "Unchecked" };
        let mut cells: Vec<_> = STATES
            .iter()
            .map(|&(state, forced)| {
                let button = ToggleButton::new(format!("toggle-{name}-{state}"), "Label")
                    .icon(IconName::Favorite)
                    .checked(checked);
                match forced {
                    Some(forced) => button.forced(forced).into_any_element(),
                    None => button.into_any_element(),
                }
            })
            .collect();
        cells.push(
            ToggleButton::new(format!("toggle-{name}-disabled"), "Label")
                .icon(IconName::Favorite)
                .checked(checked)
                .disabled(true)
                .into_any_element(),
        );
        state_row(name, cells)
    });

    let toggle = ToggleButton::new("toggle-demo", "お気に入り")
        .icon(IconName::Favorite)
        .checked(demo.toggle)
        .on_change({
            let view = view.clone();
            move |checked, _, cx| Gallery::update_demo(&view, cx, |demo| demo.toggle = checked)
        });

    let standard = ButtonGroup::new("group-standard", "書式").children(
        [
            ("group-bold", "太字"),
            ("group-italic", "斜体"),
            ("group-underline", "下線"),
        ]
        .into_iter()
        .enumerate()
        .map(|(index, (id, label))| {
            let view = view.clone();
            ToggleButton::new(id, label)
                .checked(demo.formats[index])
                .on_change(move |checked, _, cx| {
                    Gallery::update_demo(&view, cx, |demo| demo.formats[index] = checked)
                })
        }),
    );

    let connected_single = ButtonGroup::new("group-range", "期間")
        .connected(true)
        .children(
            ["日", "週", "月", "年"]
                .into_iter()
                .enumerate()
                .map(|(index, label)| {
                    let view = view.clone();
                    ToggleButton::new(("group-range", index), label)
                        .checked(demo.range == index)
                        .on_change(move |_, _, cx| {
                            Gallery::update_demo(&view, cx, |demo| demo.range = index)
                        })
                }),
        );

    let connected_multi = ButtonGroup::new("group-filters", "表示する項目")
        .connected(true)
        .children(
            [
                (IconName::Mail, "メール"),
                (IconName::Notifications, "通知"),
                (IconName::Star, "スター"),
            ]
            .into_iter()
            .enumerate()
            .map(|(index, (icon, label))| {
                let view = view.clone();
                ToggleButton::new(("group-filters", index), label)
                    .icon(icon)
                    .checked(demo.shown[index])
                    .disabled(index == 2)
                    .on_change(move |checked, _, cx| {
                        Gallery::update_demo(&view, cx, |demo| demo.shown[index] = checked)
                    })
            }),
        );

    div()
        .flex()
        .flex_col()
        .gap_6()
        .child(Text::new(TypeScale::BodyMedium, motion).color(ColorRole::OnSurfaceVariant))
        .child(
            Section::new("ToggleButton")
                .child(state_header())
                .children(toggle_rows)
                .child(div().flex().child(toggle)),
        )
        .child(
            Section::new("ButtonGroup")
                .child(Text::new(TypeScale::LabelLarge, "Standard"))
                .child(standard)
                .child(Text::new(TypeScale::LabelLarge, "Connected（1 つを選ぶ）"))
                .child(connected_single)
                .child(Text::new(
                    TypeScale::LabelLarge,
                    "Connected（複数選択、無効な項目あり）",
                ))
                .child(connected_multi),
        )
}
