//! チェックボックスの状態と、まとめて操作するチェックボックスを並べるページ

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Render, SharedString,
    StatefulInteractiveElement as _, Styled as _, Window, div,
};
use md3_components::checkbox::{Checkbox, CheckedState};
use md3_theme::ThemeExt as _;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;

use crate::theme_page::{section, styled_text};

const TOPPINGS: [&str; 3] = ["Cheese", "Tomato", "Basil"];

/// チェックボックスを並べるページ。まとめて操作する見本の配下の状態を持つ
pub struct CheckboxPage {
    toppings: [bool; 3],
}

impl CheckboxPage {
    pub fn new() -> Self {
        Self {
            toppings: [true, false, false],
        }
    }
}

impl Render for CheckboxPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let entity = cx.entity();
        let labeled = |checkbox: Checkbox, label: &str| {
            div()
                .flex()
                .items_center()
                .child(checkbox)
                .child(styled_text(&theme, TypeScale::BodyLarge, label.to_owned()))
        };

        let all = match self.toppings.iter().filter(|checked| **checked).count() {
            0 => CheckedState::Unchecked,
            count if count == TOPPINGS.len() => CheckedState::Checked,
            _ => CheckedState::Mixed,
        };
        let parent = {
            let entity = entity.clone();
            Checkbox::new("toppings-all", all, "All toppings").on_change(move |state, _, cx| {
                entity.update(cx, |page, cx| {
                    page.toppings = [state.is_checked(); 3];
                    cx.notify();
                })
            })
        };
        let children = TOPPINGS.iter().enumerate().map(|(index, name)| {
            let entity = entity.clone();
            let checkbox = Checkbox::new(
                SharedString::from(format!("topping-{name}")),
                self.toppings[index].into(),
                *name,
            )
            .on_change(move |state, _, cx| {
                entity.update(cx, |page, cx| {
                    page.toppings[index] = state.is_checked();
                    cx.notify();
                })
            });
            labeled(checkbox, name).pl_8()
        });

        let states = [
            (CheckedState::Unchecked, "Unchecked"),
            (CheckedState::Checked, "Checked"),
            (CheckedState::Mixed, "Mixed"),
        ]
        .map(|(state, name)| {
            div().flex().flex_wrap().items_center().gap_4().children([
                labeled(
                    Checkbox::new(SharedString::from(format!("{name}-enabled")), state, name),
                    name,
                ),
                labeled(
                    Checkbox::new(SharedString::from(format!("{name}-disabled")), state, name)
                        .disabled(true),
                    &format!("{name} (disabled)"),
                ),
            ])
        });

        div()
            .id("checkbox-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_col()
            .gap_8()
            .child(section(
                &theme,
                "まとめて操作する",
                div()
                    .flex()
                    .flex_col()
                    .child(labeled(parent, "All toppings"))
                    .children(children),
            ))
            .child(section(
                &theme,
                "状態",
                div().flex().flex_col().gap_2().children(states),
            ))
    }
}
