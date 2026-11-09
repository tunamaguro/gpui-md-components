//! スイッチの状態を並べるページ

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Refineable as _, Render,
    StatefulInteractiveElement as _, Styled as _, Window, div,
};
use md3_components::Switch;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

const STATES: [(bool, &str); 2] = [(false, "Off"), (true, "On")];

/// スイッチを並べるページ。操作できるスイッチの状態を持つ
#[derive(Default)]
pub struct SwitchPage {
    wifi: bool,
    bluetooth: bool,
}

impl Render for SwitchPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let row = |name: &'static str, configure: fn(Switch) -> Switch| {
            div()
                .flex()
                .items_center()
                .gap_6()
                .child(label(&theme, name).w_32())
                .children(STATES.map(|(checked, state_name)| {
                    configure(Switch::new(format!("{name}-{state_name}"), checked))
                        .aria_label(format!("{name} {state_name}"))
                }))
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
            .items_start()
            .gap_6()
            .child(row("Enabled", |switch| switch))
            .child(row("Icon", |switch| switch.check_icon(true)))
            .child(row("Disabled", |switch| switch.disabled(true)))
            .child(row("Disabled icon", |switch| {
                switch.disabled(true).check_icon(true)
            }))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_6()
                    .child(label(&theme, "Wi-Fi").w_32())
                    .child(Switch::new("wifi", self.wifi).aria_label("Wi-Fi").on_click(
                        cx.listener(|this, checked: &bool, _, cx| {
                            this.wifi = *checked;
                            cx.notify();
                        }),
                    )),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_6()
                    .child(label(&theme, "Bluetooth").w_32())
                    .child(
                        Switch::new("bluetooth", self.bluetooth)
                            .aria_label("Bluetooth")
                            .check_icon(true)
                            .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                this.bluetooth = *checked;
                                cx.notify();
                            })),
                    ),
            )
    }
}

/// 支援技術に公開されるテキスト
///
/// `text` を支援技術のノードの id にも使うため、ページ内で重ならない文字列を渡すこと
fn label(theme: &Theme, text: impl Into<gpui::SharedString>) -> gpui::Div {
    let text = text.into();
    let mut element = div();
    element
        .text_style()
        .refine(&theme.text_style(TypeScale::BodyMedium));
    element.child(gpui::text!(id = text.clone(), text))
}
