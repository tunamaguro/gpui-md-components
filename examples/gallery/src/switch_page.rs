//! スイッチの状態とアイコンの有無を並べるページ

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Render, SharedString,
    StatefulInteractiveElement as _, Styled as _, Window, div,
};
use md3_components::switch::Switch;
use md3_theme::ThemeExt as _;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;

use crate::theme_page::{section, styled_text};

const SETTINGS: [&str; 2] = ["Wi-Fi", "Bluetooth"];

/// スイッチを並べるページ。操作できる見本のオンとオフを持つ
pub struct SwitchPage {
    settings: [bool; 2],
}

impl SwitchPage {
    pub fn new() -> Self {
        Self {
            settings: [true, false],
        }
    }
}

impl Render for SwitchPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let entity = cx.entity();
        let labeled = |switch: Switch, label: &str| {
            div()
                .flex()
                .items_center()
                .gap_4()
                .child(styled_text(&theme, TypeScale::BodyLarge, label.to_owned()))
                .child(switch)
        };

        let settings = [false, true].map(|icons| {
            let rows = SETTINGS.iter().enumerate().map(|(index, name)| {
                let entity = entity.clone();
                let switch = Switch::new(
                    SharedString::from(format!("{name}-{icons}")),
                    self.settings[index],
                    *name,
                )
                .icons(icons)
                .on_change(move |selected, _, cx| {
                    entity.update(cx, |page, cx| {
                        page.settings[index] = selected;
                        cx.notify();
                    })
                });
                labeled(switch, name).w_64().justify_between()
            });
            section(
                &theme,
                if icons {
                    "アイコンあり"
                } else {
                    "アイコンなし"
                },
                div().flex().flex_col().children(rows),
            )
        });

        let disabled = [false, true].map(|selected| {
            let name = if selected { "On" } else { "Off" };
            div()
                .flex()
                .gap_8()
                .child(labeled(
                    Switch::new(
                        SharedString::from(format!("disabled-{name}")),
                        selected,
                        name,
                    )
                    .disabled(true),
                    name,
                ))
                .child(labeled(
                    Switch::new(
                        SharedString::from(format!("disabled-icons-{name}")),
                        selected,
                        name,
                    )
                    .icons(true)
                    .disabled(true),
                    &format!("{name} (icons)"),
                ))
        });

        div()
            .id("switch-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_col()
            .gap_8()
            .children(settings)
            .child(section(
                &theme,
                "無効",
                div().flex().flex_col().children(disabled),
            ))
    }
}
