//! 区切り線を並べるページ

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Refineable as _, Render,
    StatefulInteractiveElement as _, Styled as _, Window, div, px,
};
use md3_components::{Divider, DividerOrientation};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

/// 区切り線を並べるページ
pub struct DividerPage;

impl Render for DividerPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let item = |text: &'static str| label(&theme, text).px_4().py_3();

        div()
            .id("divider-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_col()
            .gap_6()
            .child(
                div()
                    .w(px(360.))
                    .flex()
                    .flex_col()
                    .child(item("Full width"))
                    .child(Divider::new())
                    .child(item("Inset"))
                    .child(Divider::new().ml(px(16.)))
                    .child(item("Middle inset"))
                    .child(Divider::new().mx(px(16.)))
                    .child(item("Last")),
            )
            .child(
                div()
                    .h(px(48.))
                    .flex()
                    .items_center()
                    .child(item("Left"))
                    .child(Divider::new().orientation(DividerOrientation::Vertical))
                    .child(item("Middle"))
                    .child(
                        Divider::new()
                            .orientation(DividerOrientation::Vertical)
                            .my(px(8.)),
                    )
                    .child(item("Right")),
            )
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
