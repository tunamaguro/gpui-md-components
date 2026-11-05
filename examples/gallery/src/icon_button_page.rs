//! アイコンボタンの種類・大きさ・幅・形を並べるページ

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Render, SharedString,
    StatefulInteractiveElement as _, Styled as _, Window, div,
};
use md3_components::IconName;
use md3_components::button::{Button, ButtonVariant};
use md3_components::icon_button::{
    IconButton, IconButtonShape, IconButtonSize, IconButtonVariant, IconButtonWidth,
};
use md3_theme::ThemeExt as _;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;

use crate::button_page::row;
use crate::theme_page::{section, styled_text};

const VARIANTS: [(IconButtonVariant, &str); 4] = [
    (IconButtonVariant::Standard, "Standard"),
    (IconButtonVariant::Filled, "Filled"),
    (IconButtonVariant::Tonal, "Tonal"),
    (IconButtonVariant::Outlined, "Outlined"),
];

const SIZES: [(IconButtonSize, &str); 5] = [
    (IconButtonSize::ExtraSmall, "XS"),
    (IconButtonSize::Small, "S"),
    (IconButtonSize::Medium, "M"),
    (IconButtonSize::Large, "L"),
    (IconButtonSize::ExtraLarge, "XL"),
];

const WIDTHS: [(IconButtonWidth, &str); 3] = [
    (IconButtonWidth::Narrow, "Narrow"),
    (IconButtonWidth::Default, "Default"),
    (IconButtonWidth::Wide, "Wide"),
];

/// アイコンボタンを並べるページ。押した回数と、見本の形を持つ
pub struct IconButtonPage {
    clicks: usize,
    shape: IconButtonShape,
}

impl IconButtonPage {
    pub fn new() -> Self {
        Self {
            clicks: 0,
            shape: IconButtonShape::Round,
        }
    }
}

impl Render for IconButtonPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let shape = self.shape;
        let entity = cx.entity();
        let count = |id: String, icon: IconName, label: &str| {
            let entity = entity.clone();
            IconButton::new(SharedString::from(id), icon, label.to_owned())
                .shape(shape)
                .on_click(move |_, _, cx| {
                    entity.update(cx, |page, cx| {
                        page.clicks += 1;
                        cx.notify();
                    })
                })
        };

        let shapes = [
            (IconButtonShape::Round, "Round"),
            (IconButtonShape::Square, "Square"),
        ]
        .map(|(candidate, label)| {
            let entity = entity.clone();
            Button::new(SharedString::from(format!("icon-shape-{label}")), label)
                .variant(if candidate == shape {
                    ButtonVariant::Filled
                } else {
                    ButtonVariant::Outlined
                })
                .on_click(move |_, _, cx| {
                    entity.update(cx, |page, cx| {
                        page.shape = candidate;
                        cx.notify();
                    })
                })
        });

        let variants = VARIANTS.map(|(variant, name)| {
            row(&theme, name).children([
                count(format!("{name}-favorite"), IconName::Favorite, "Favorite")
                    .variant(variant)
                    .into_any_element(),
                count(format!("{name}-settings"), IconName::Settings, "Settings")
                    .variant(variant)
                    .into_any_element(),
                IconButton::new(
                    SharedString::from(format!("{name}-icon-disabled")),
                    IconName::Delete,
                    "Delete",
                )
                .variant(variant)
                .shape(shape)
                .disabled(true)
                .into_any_element(),
            ])
        });
        let sizes = SIZES.map(|(size, name)| {
            row(&theme, name).children(WIDTHS.map(|(width, width_name)| {
                count(format!("{name}-{width_name}"), IconName::Edit, "Edit")
                    .variant(IconButtonVariant::Tonal)
                    .size(size)
                    .width(width)
            }))
        });

        div()
            .id("icon-button-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_col()
            .gap_8()
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_2()
                    .children(shapes)
                    .child(styled_text(
                        &theme,
                        TypeScale::BodyLarge,
                        format!("押した回数: {}", self.clicks),
                    )),
            )
            .child(section(
                &theme,
                "種類",
                div().flex().flex_col().gap_2().children(variants),
            ))
            .child(section(
                &theme,
                "大きさと幅 (Narrow・Default・Wide)",
                div().flex().flex_col().gap_2().children(sizes),
            ))
    }
}
