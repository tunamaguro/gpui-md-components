//! スナックバーを表示するページ

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Refineable as _, Render,
    SharedString, Styled as _, Window, div,
};
use md3_components::{
    Button, ButtonVariant, SnackbarDuration, SnackbarHost, SnackbarHostState, SnackbarMessage,
    SnackbarResult,
};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

/// スナックバーを表示するページ。表示の列と、最後に表示を終えたスナックバーの結果を持つ
pub struct SnackbarPage {
    state: SnackbarHostState,
    last_result: Option<SharedString>,
}

impl SnackbarPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            state: SnackbarHostState::new(cx),
            last_result: None,
        }
    }

    fn show_button(
        &self,
        id: &'static str,
        text: &'static str,
        message: SnackbarMessage,
        cx: &mut Context<Self>,
    ) -> Button {
        let state = self.state.clone();
        let page = cx.entity().downgrade();
        Button::new(id)
            .variant(ButtonVariant::FilledTonal)
            .on_click(move |_, _, cx| {
                let page = page.clone();
                let label = text;
                state.show(
                    message.clone(),
                    move |result, cx| {
                        page.update(cx, |page, cx| {
                            page.last_result = Some(
                                format!(
                                    "{label}: {}",
                                    match result {
                                        SnackbarResult::Dismissed => "dismissed",
                                        SnackbarResult::ActionPerformed => "action",
                                    }
                                )
                                .into(),
                            );
                            cx.notify();
                        })
                        .ok();
                    },
                    cx,
                )
            })
            .child(gpui::text!(
                id = SharedString::from(format!("{id}-label")),
                text
            ))
    }
}

impl Render for SnackbarPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let buttons = div()
            .flex()
            .flex_wrap()
            .gap_4()
            .child(self.show_button(
                "show-text",
                "Text only",
                SnackbarMessage::new("Photo saved"),
                cx,
            ))
            .child(self.show_button(
                "show-action",
                "With action",
                SnackbarMessage::new("Message archived").action("Undo"),
                cx,
            ))
            .child(self.show_button(
                "show-dismiss",
                "With close button",
                SnackbarMessage::new("Connection restored")
                    .dismiss_action("Dismiss")
                    .duration(SnackbarDuration::Long),
                cx,
            ))
            .child(self.show_button(
                "show-new-line",
                "Action on new line",
                SnackbarMessage::new(
                    "Your files were backed up. Some photos could not be uploaded because they are too large.",
                )
                .action("Review")
                .dismiss_action("Dismiss")
                .action_on_new_line(true),
                cx,
            ));

        div()
            .id("snackbar-page")
            .size_full()
            .relative()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_col()
            .gap_6()
            .child(label(
                &theme,
                "last-result",
                format!(
                    "最後の結果: {}",
                    self.last_result.as_deref().unwrap_or("なし")
                ),
            ))
            .child(buttons)
            .child(
                SnackbarHost::new("snackbar-host", &self.state)
                    .absolute()
                    .bottom_0()
                    .left_0()
                    .right_0(),
            )
    }
}

/// 支援技術に公開されるテキスト
///
/// `id` は支援技術のノードの id に使うため、ページ内で重ならない値を渡し、テキストが変わっても同じ値を渡すこと
fn label(
    theme: &Theme,
    id: impl Into<gpui::ElementId>,
    text: impl Into<gpui::SharedString>,
) -> gpui::Div {
    let mut element = div();
    element
        .text_style()
        .refine(&theme.text_style(TypeScale::BodyMedium));
    element.child(gpui::text!(id = id.into(), text.into()))
}
