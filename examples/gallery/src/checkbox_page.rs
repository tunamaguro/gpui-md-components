//! チェックボックスの状態を並べるページ

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Refineable as _, Render,
    StatefulInteractiveElement as _, Styled as _, Window, div,
};
use md3_components::{Checkbox, CheckboxState};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

const STATES: [(CheckboxState, &str); 3] = [
    (CheckboxState::Unchecked, "Unchecked"),
    (CheckboxState::Checked, "Checked"),
    (CheckboxState::Indeterminate, "Indeterminate"),
];

/// チェックボックスを並べるページ。操作できるチェックボックスの状態を持つ
#[derive(Default)]
pub struct CheckboxPage {
    single: CheckboxState,
    children: [bool; 2],
}

impl Render for CheckboxPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let row = |name: &'static str, configure: fn(Checkbox) -> Checkbox| {
            div()
                .flex()
                .items_center()
                .gap_4()
                .child(label(&theme, name).w_32())
                .children(STATES.map(|(state, state_name)| {
                    configure(Checkbox::new(format!("{name}-{state_name}"), state))
                        .aria_label(format!("{name} {state_name}"))
                }))
        };

        let parent = match self.children {
            [true, true] => CheckboxState::Checked,
            [false, false] => CheckboxState::Unchecked,
            _ => CheckboxState::Indeterminate,
        };
        // チェックボックスはラベルを表示しないため、名前と同じテキストを横に並べる
        let labeled = |checkbox: Checkbox, text: &'static str| {
            div()
                .flex()
                .items_center()
                .child(checkbox.aria_label(text))
                .child(label(&theme, text))
        };
        let child = |index: usize, name: &'static str, cx: &mut Context<Self>| {
            let state = if self.children[index] {
                CheckboxState::Checked
            } else {
                CheckboxState::Unchecked
            };
            labeled(
                Checkbox::new(name, state).on_click(cx.listener(
                    move |this, state: &CheckboxState, _, cx| {
                        this.children[index] = *state == CheckboxState::Checked;
                        cx.notify();
                    },
                )),
                name,
            )
        };

        div()
            .id("checkbox-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_col()
            .gap_4()
            .child(row("Enabled", |checkbox| checkbox))
            .child(row("Error", |checkbox| {
                checkbox.error(Some("Required".into()))
            }))
            .child(row("Disabled", |checkbox| checkbox.disabled(true)))
            .child(labeled(
                Checkbox::new("single", self.single).on_click(cx.listener(
                    |this, state: &CheckboxState, _, cx| {
                        this.single = *state;
                        cx.notify();
                    },
                )),
                "テキストを横に並べたチェックボックス",
            ))
            .child(labeled(
                Checkbox::new("parent", parent).on_click(cx.listener(
                    |this, state: &CheckboxState, _, cx| {
                        this.children = [*state == CheckboxState::Checked; 2];
                        cx.notify();
                    },
                )),
                "すべて選択",
            ))
            .child(
                div()
                    .pl_6()
                    .flex()
                    .flex_col()
                    .child(child(0, "項目 1", cx))
                    .child(child(1, "項目 2", cx)),
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
