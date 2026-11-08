//! スイッチの状態を並べるページ

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Refineable as _, Render,
    StatefulInteractiveElement as _, Styled as _, Window, div,
};
use md3_components::Switch;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

/// スイッチを並べるページ。操作できるスイッチのオンかオフかを持つ
#[derive(Default)]
pub struct SwitchPage {
    plain: bool,
    icon: bool,
}

impl Render for SwitchPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let row = |name: &str| {
            div()
                .flex()
                .items_center()
                .gap_4()
                .child(label(&theme, name).w_32())
        };
        div()
            .id("switch-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                row("Plain").child(
                    Switch::new("plain", self.plain)
                        .aria_label("Plain")
                        .on_change(on_change(cx, |this, checked| this.plain = checked)),
                ),
            )
            .child(
                row("Check icon").child(
                    Switch::new("icon", self.icon)
                        .aria_label("Check icon")
                        .check_icon(true)
                        .on_change(on_change(cx, |this, checked| this.icon = checked)),
                ),
            )
            .child(
                row("Disabled")
                    .child(
                        Switch::new("disabled-off", false)
                            .aria_label("Disabled off")
                            .disabled(true),
                    )
                    .child(
                        Switch::new("disabled-on", true)
                            .aria_label("Disabled on")
                            .disabled(true),
                    )
                    .child(
                        Switch::new("disabled-icon", true)
                            .aria_label("Disabled with icon")
                            .check_icon(true)
                            .disabled(true),
                    ),
            )
    }
}

/// オンかオフかを `set` でページに書き込み、ページを描き直すハンドラ
fn on_change(
    cx: &Context<SwitchPage>,
    set: fn(&mut SwitchPage, bool),
) -> impl Fn(bool, &mut Window, &mut gpui::App) + 'static {
    let page = cx.weak_entity();
    move |checked, _, cx| {
        page.update(cx, |this, cx| {
            set(this, checked);
            cx.notify();
        })
        .ok();
    }
}

fn label(theme: &Theme, text: impl Into<gpui::SharedString>) -> gpui::Div {
    let mut element = div();
    element
        .text_style()
        .refine(&theme.text_style(TypeScale::BodyMedium));
    element.child(text.into())
}
