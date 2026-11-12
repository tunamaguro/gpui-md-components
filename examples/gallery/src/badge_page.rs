//! バッジを並べるページ

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Refineable as _, Render,
    StatefulInteractiveElement as _, Styled as _, Window, div, px, svg,
};
use md3_components::{Badge, BadgeColors, BadgedBox};
use md3_icons::StyledIcon;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

/// バッジを並べるページ
pub struct BadgePage;

impl Render for BadgePage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let icon = |icon: md3_icons::Icon| {
            svg()
                .data(StyledIcon::from(icon).svg().as_bytes())
                .size(px(24.))
                .text_color(theme.color(ColorRole::OnSurfaceVariant))
        };
        let row = |name: &'static str| {
            div()
                .flex()
                .items_center()
                .gap_8()
                .child(label(&theme, name).w_32())
        };

        div()
            .id("badge-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_col()
            .items_start()
            .gap_8()
            .child(
                row("Small").child(
                    BadgedBox::new(Badge::new("notifications").aria_label("New notifications"))
                        .child(icon(md3_icons::NOTIFICATIONS)),
                ),
            )
            .child(
                row("Large")
                    .child(
                        BadgedBox::new(Badge::new("mail").child("3").aria_label("3 new messages"))
                            .child(icon(md3_icons::MAIL)),
                    )
                    .child(
                        BadgedBox::new(
                            Badge::new("chat")
                                .child("999+")
                                .aria_label("Over 999 chats"),
                        )
                        .child(icon(md3_icons::CHAT)),
                    ),
            )
            .child(
                row("Colors").child(
                    BadgedBox::new(
                        Badge::new("cart")
                            .child("12")
                            .colors(BadgeColors {
                                container: Some(ColorRole::Primary),
                                content: Some(ColorRole::OnPrimary),
                            })
                            .aria_label("12 items in cart"),
                    )
                    .child(icon(md3_icons::SHOPPING_CART)),
                ),
            )
            .child(row("Standalone").child(Badge::new("standalone").child("NEW").aria_label("New")))
    }
}

/// 支援技術に公開されるテキスト
///
/// `text` を支援技術のノードの id にも使うため、ページ内で重ならない文字列を渡すこと
fn label(theme: &Theme, text: &'static str) -> gpui::Div {
    let mut element = div();
    element
        .text_style()
        .refine(&theme.text_style(TypeScale::BodyMedium));
    element.child(gpui::text!(id = text, text))
}
