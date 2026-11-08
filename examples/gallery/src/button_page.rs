//! ボタンのバリアントと状態を並べるページ

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Refineable as _, Render,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div,
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

/// ボタンを並べるページ。押した回数と最後に押したボタンを持つ
#[derive(Default)]
pub struct ButtonPage {
    clicks: usize,
    last_clicked: Option<SharedString>,
}

impl Render for ButtonPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let rows = VARIANTS.map(|(variant, name)| {
            let button = |suffix: &str| button(format!("{name}-{suffix}"), cx).variant(variant);
            div()
                .flex()
                .items_center()
                .gap_4()
                .child(label(&theme, name).w_32())
                .child(button("plain").child(name))
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
            .child(label(
                &theme,
                format!(
                    "押した回数: {} / 最後に押したボタン: {}",
                    self.clicks,
                    self.last_clicked.as_deref().unwrap_or("なし")
                ),
            ))
            .children(rows)
            .children(SIZES.map(|(size, name)| {
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .child(label(&theme, name).w_32())
                    .child(button(format!("size-{name}"), cx).size(size).child(name))
                    .child(
                        button(format!("size-{name}-icon"), cx)
                            .size(size)
                            .variant(ButtonVariant::Outlined)
                            .icon(md3_icons::ADD)
                            .child("With icon"),
                    )
            }))
            .child(
                button("full-width", cx)
                    .variant(ButtonVariant::Outlined)
                    .w_full()
                    .child("幅いっぱいのボタン"),
            )
    }
}

/// 押すと押した回数と最後に押したボタンを更新するボタン。`id` を最後に押したボタンとして表示する
fn button(id: impl Into<SharedString>, cx: &Context<ButtonPage>) -> Button {
    let id = id.into();
    Button::new(id.clone()).on_click(cx.listener(move |this, _, _, cx| {
        this.clicks += 1;
        this.last_clicked = Some(id.clone());
        cx.notify();
    }))
}

fn label(theme: &Theme, text: impl Into<gpui::SharedString>) -> gpui::Div {
    let mut element = div();
    element
        .text_style()
        .refine(&theme.text_style(TypeScale::BodyMedium));
    element.child(text.into())
}
