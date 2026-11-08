//! ボタンのバリアントと状態を並べるページ

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Refineable as _, Render,
    StatefulInteractiveElement as _, Styled as _, Window, div,
};
use md3_components::{Button, ButtonSize, ButtonVariant};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

const SIZES: [(ButtonSize, &str); 5] = [
    (ButtonSize::ExtraSmall, "Extra small"),
    (ButtonSize::Small, "Small"),
    (ButtonSize::Medium, "Medium"),
    (ButtonSize::Large, "Large"),
    (ButtonSize::ExtraLarge, "Extra large"),
];

const VARIANTS: [(ButtonVariant, &str); 5] = [
    (ButtonVariant::Filled, "Filled"),
    (ButtonVariant::Elevated, "Elevated"),
    (ButtonVariant::FilledTonal, "Filled tonal"),
    (ButtonVariant::Outlined, "Outlined"),
    (ButtonVariant::Text, "Text"),
];

/// ボタンを並べるページ。押した回数を持つ
#[derive(Default)]
pub struct ButtonPage {
    clicks: usize,
}

impl Render for ButtonPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let rows = VARIANTS.map(|(variant, name)| {
            let button = |suffix: &str| Button::new(format!("{name}-{suffix}")).variant(variant);
            div()
                .flex()
                .items_center()
                .gap_4()
                .child(label(&theme, name).w_32())
                .child(
                    button("plain")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.clicks += 1;
                            cx.notify();
                        }))
                        .child(name),
                )
                .child(button("icon").icon(md3_icons::ADD).child("With icon"))
                .child(button("disabled").disabled(true).child("Disabled"))
                .child(
                    button("disabled-icon")
                        .disabled(true)
                        .icon(md3_icons::ADD)
                        .child("Disabled"),
                )
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
            .gap_4()
            .child(label(&theme, format!("押した回数: {}", self.clicks)))
            .children(rows)
            .children(SIZES.map(|(size, name)| {
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .child(label(&theme, name).w_32())
                    .child(Button::new(format!("size-{name}")).size(size).child(name))
                    .child(
                        Button::new(format!("size-{name}-icon"))
                            .size(size)
                            .variant(ButtonVariant::Outlined)
                            .icon(md3_icons::ADD)
                            .child("With icon"),
                    )
            }))
            .child(
                Button::new("full-width")
                    .variant(ButtonVariant::Outlined)
                    .w_full()
                    .child("幅いっぱいのボタン"),
            )
    }
}

fn label(theme: &Theme, text: impl Into<gpui::SharedString>) -> gpui::Div {
    let mut element = div();
    element
        .text_style()
        .refine(&theme.text_style(TypeScale::BodyMedium));
    element.child(text.into())
}
