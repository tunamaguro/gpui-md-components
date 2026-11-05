//! ボタンの種類・大きさ・形を並べるページ

use gpui::{
    Context, Div, InteractiveElement as _, IntoElement, ParentElement as _, Render, SharedString,
    StatefulInteractiveElement as _, Styled as _, Window, div,
};
use md3_components::IconName;
use md3_components::button::{Button, ButtonShape, ButtonSize, ButtonVariant};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

use crate::theme_page::{section, styled_text};

const VARIANTS: [(ButtonVariant, &str); 5] = [
    (ButtonVariant::Elevated, "Elevated"),
    (ButtonVariant::Filled, "Filled"),
    (ButtonVariant::Tonal, "Tonal"),
    (ButtonVariant::Outlined, "Outlined"),
    (ButtonVariant::Text, "Text"),
];

const SIZES: [(ButtonSize, &str); 5] = [
    (ButtonSize::ExtraSmall, "XS"),
    (ButtonSize::Small, "S"),
    (ButtonSize::Medium, "M"),
    (ButtonSize::Large, "L"),
    (ButtonSize::ExtraLarge, "XL"),
];

/// ボタンを並べるページ。押した回数と、見本の形を持つ
pub struct ButtonPage {
    clicks: usize,
    shape: ButtonShape,
}

impl ButtonPage {
    pub fn new() -> Self {
        Self {
            clicks: 0,
            shape: ButtonShape::Round,
        }
    }
}

impl Render for ButtonPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let shape = self.shape;
        let entity = cx.entity();
        let count = |variant: ButtonVariant, size: ButtonSize, label: &str| {
            let entity = entity.clone();
            Button::new(SharedString::from(format!("{variant:?}-{size:?}")), label)
                .variant(variant)
                .size(size)
                .shape(shape)
                .on_click(move |_, _, cx| {
                    entity.update(cx, |page, cx| {
                        page.clicks += 1;
                        cx.notify();
                    })
                })
        };

        let shapes = [
            (ButtonShape::Round, "Round"),
            (ButtonShape::Square, "Square"),
        ]
        .map(|(candidate, label)| {
            let entity = entity.clone();
            Button::new(SharedString::from(format!("shape-{label}")), label)
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
                count(variant, ButtonSize::Small, "Label").into_any_element(),
                count(variant, ButtonSize::ExtraSmall, "With icon")
                    .icon(IconName::Add)
                    .into_any_element(),
                Button::new(SharedString::from(format!("{name}-disabled")), "Disabled")
                    .variant(variant)
                    .shape(shape)
                    .disabled(true)
                    .into_any_element(),
            ])
        });
        let sizes = SIZES.map(|(size, name)| {
            row(&theme, name).child(count(ButtonVariant::Filled, size, "Send").icon(IconName::Send))
        });

        div()
            .id("button-page")
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
                "大きさ",
                div().flex().flex_col().gap_2().children(sizes),
            ))
    }
}

/// 見出し `name` の付いた横 1 列
pub(crate) fn row(theme: &Theme, name: &str) -> Div {
    div()
        .flex()
        .flex_wrap()
        .items_center()
        .gap_4()
        .child(
            div()
                .w_24()
                .child(styled_text(theme, TypeScale::LabelLarge, name.to_owned())),
        )
}
