//! テキストフィールドを並べるページ

use gpui::{
    Context, IntoElement, ParentElement as _, Render, SharedString, Styled as _, Window, div,
    prelude::*, px,
};
use md3_components::{TextField, TextFieldVariant};
use md3_theme::ThemeExt as _;
use md3_theme::md3_color::ColorRole;

/// 種類と状態の違うテキストフィールドを並べるページ。テキストフィールドごとに文字列を持つ
pub struct TextFieldPage {
    values: [SharedString; 7],
}

impl Default for TextFieldPage {
    fn default() -> Self {
        Self {
            values: [
                SharedString::default(),
                SharedString::default(),
                "1200".into(),
                SharedString::default(),
                "Read only".into(),
                "Disabled".into(),
                SharedString::default(),
            ],
        }
    }
}

impl TextFieldPage {
    /// 番号 `ix` の文字列を表示し、編集を受け入れるテキストフィールド
    fn text_field(&self, ix: usize, cx: &mut Context<Self>) -> TextField {
        TextField::new(("text-field", ix), self.values[ix].clone())
            .on_change(cx.listener(move |this, value: &SharedString, _, cx| {
                this.values[ix] = value.clone();
                cx.notify();
            }))
            .w(px(280.))
    }
}

impl Render for TextFieldPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        div()
            .id("text-field-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_wrap()
            .content_start()
            .items_start()
            .gap_6()
            .child(
                self.text_field(0, cx)
                    .label("Name")
                    .placeholder("Your name"),
            )
            .child(
                self.text_field(1, cx)
                    .variant(TextFieldVariant::Outlined)
                    .label("Email")
                    .leading_icon(md3_icons::MAIL)
                    .supporting_text("We never share your email"),
            )
            .child(
                self.text_field(2, cx)
                    .label("Price")
                    .prefix("$")
                    .suffix("USD"),
            )
            .child(
                self.text_field(3, cx)
                    .variant(TextFieldVariant::Outlined)
                    .label("Username")
                    .error(self.values[3].is_empty())
                    .supporting_text("Required"),
            )
            .child(self.text_field(4, cx).label("Note").read_only(true))
            .child(
                self.text_field(5, cx)
                    .variant(TextFieldVariant::Outlined)
                    .label("Disabled")
                    .disabled(true),
            )
            .child(
                self.text_field(6, cx)
                    .aria_label("Search")
                    .placeholder("Search")
                    .leading_icon(md3_icons::SEARCH),
            )
    }
}
