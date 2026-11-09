//! チェックボックスの状態を並べるページ

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Refineable as _, Render,
    StatefulInteractiveElement as _, Styled as _, Window, div,
};
use md3_components::{Checkbox, CheckboxIndicator, CheckboxRow, CheckboxState, InteractionSource};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

const STATES: [(CheckboxState, &str); 3] = [
    (CheckboxState::Unchecked, "Unchecked"),
    (CheckboxState::Checked, "Checked"),
    (CheckboxState::Indeterminate, "Indeterminate"),
];

/// チェックボックスを並べるページ。操作できるチェックボックスの状態と、行が使う [`InteractionSource`] を持つ
pub struct CheckboxPage {
    single: CheckboxState,
    children: [bool; 2],
    parent_source: InteractionSource,
    child_sources: [InteractionSource; 2],
}

impl CheckboxPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            single: CheckboxState::default(),
            children: [false; 2],
            parent_source: InteractionSource::new(cx),
            child_sources: [InteractionSource::new(cx), InteractionSource::new(cx)],
        }
    }
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
        // 行全体を押せる範囲にし、行の hover とフォーカスを見た目だけのチェックボックスに表示する
        let checkbox_row = |id: &'static str,
                            state: CheckboxState,
                            source: &InteractionSource,
                            text: &'static str| {
            CheckboxRow::new(id, state, source)
                .flex()
                .items_center()
                .pr_4()
                .child(CheckboxIndicator::new(state).interaction_source(source))
                .child(label(&theme, text))
        };
        let child = |index: usize, text: &'static str, cx: &mut Context<Self>| {
            let state = if self.children[index] {
                CheckboxState::Checked
            } else {
                CheckboxState::Unchecked
            };
            checkbox_row(text, state, &self.child_sources[index], text).on_click(cx.listener(
                move |this, state: &CheckboxState, _, cx| {
                    this.children[index] = *state == CheckboxState::Checked;
                    cx.notify();
                },
            ))
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
            .items_start()
            .gap_4()
            .child(row("Enabled", |checkbox| checkbox))
            .child(row("Error", |checkbox| {
                checkbox.error(Some("Required".into()))
            }))
            .child(row("Disabled", |checkbox| checkbox.disabled(true)))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .child(label(&theme, "単体").w_32())
                    .child(
                        Checkbox::new("single", self.single)
                            .aria_label("単体のチェックボックス")
                            .on_click(cx.listener(|this, state: &CheckboxState, _, cx| {
                                this.single = *state;
                                cx.notify();
                            })),
                    ),
            )
            .child(
                checkbox_row("parent", parent, &self.parent_source, "すべて選択").on_click(
                    cx.listener(|this, state: &CheckboxState, _, cx| {
                        this.children = [*state == CheckboxState::Checked; 2];
                        cx.notify();
                    }),
                ),
            )
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

/// 支援技術に公開されるテキスト。[`CheckboxRow`] の子にすると行の名前になる
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
