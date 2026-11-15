//! ダイアログを並べるページ

use gpui::{
    Context, FocusHandle, IntoElement, ParentElement as _, Refineable as _, Render, SharedString,
    Styled as _, Window, div, prelude::*,
};
use md3_components::{Button, ButtonVariant, Dialog, DialogKind};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

/// ダイアログを開くボタンを並べるページ。開いているダイアログと、最後の操作を持つ
pub struct DialogPage {
    /// 開いているダイアログの id
    open: Option<&'static str>,
    /// 最後にダイアログを閉じた操作
    last_action: Option<SharedString>,
    /// 削除を確認するダイアログの、閉じるボタンのフォーカス
    alert_cancel_focus: FocusHandle,
}

impl DialogPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            open: None,
            last_action: None,
            alert_cancel_focus: cx.focus_handle(),
        }
    }

    fn opener(&self, id: &'static str, label: &'static str, cx: &mut Context<Self>) -> Button {
        Button::new(id)
            .variant(ButtonVariant::FilledTonal)
            .on_click(cx.listener(move |this, _, _, cx| {
                this.open = Some(id);
                cx.notify();
            }))
            .child(gpui::text!(label))
    }

    /// `id` のダイアログ。Escape と幕のクリックでは閉じる
    fn dialog(&self, id: &'static str, cx: &mut Context<Self>) -> Dialog {
        Dialog::new(id, self.open == Some(id)).on_dismiss(cx.listener(|this, reason, _, cx| {
            this.open = None;
            this.last_action = Some(format!("Dismissed ({reason:?})").into());
            cx.notify();
        }))
    }

    /// 押すとダイアログを閉じ、`label` を最後の操作にするボタン
    fn action(&self, id: &'static str, label: &'static str, cx: &mut Context<Self>) -> Button {
        Button::new(id)
            .variant(ButtonVariant::Text)
            .on_click(cx.listener(move |this, _, _, cx| {
                this.open = None;
                this.last_action = Some(label.into());
                cx.notify();
            }))
            .child(gpui::text!(label))
    }
}

impl Render for DialogPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let last_action = match &self.last_action {
            Some(action) => format!("Last action: {action}"),
            None => "Last action: none".to_string(),
        };

        div()
            .id("dialog-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_col()
            .items_start()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_4()
                    .child(self.opener("open-basic", "Basic", cx))
                    .child(self.opener("open-alert", "Alert with icon", cx))
                    .child(self.opener("open-long", "Long content", cx)),
            )
            .child(label(&theme, "last-action", last_action))
            .child(
                self.dialog("open-basic", cx)
                    .headline("Reset settings?")
                    .supporting_text(
                        "This will reset your app preferences back to their default settings.",
                    )
                    .confirm_button(self.action("basic-accept", "Accept", cx))
                    .dismiss_button(self.action("basic-cancel", "Cancel", cx)),
            )
            .child(
                self.dialog("open-alert", cx)
                    .kind(DialogKind::Alert)
                    .icon(md3_icons::DELETE)
                    .headline("Delete selected images?")
                    .supporting_text("Images will be permanently removed from your account.")
                    // 取り消せない操作なので、開いたときは閉じるボタンにフォーカスする
                    .initial_focus(&self.alert_cancel_focus)
                    .confirm_button(self.action("alert-delete", "Delete", cx))
                    .dismiss_button(
                        self.action("alert-cancel", "Cancel", cx)
                            .track_focus(&self.alert_cancel_focus),
                    ),
            )
            .child(
                self.dialog("open-long", cx)
                    .headline("Terms of service")
                    .children((1..=40u32).map(|ix| {
                        gpui::text!(
                            id = ("terms", ix),
                            SharedString::from(format!("{ix}. Please read this line carefully."))
                        )
                    }))
                    .confirm_button(self.action("long-agree", "Agree", cx))
                    .dismiss_button(self.action("long-decline", "Decline", cx)),
            )
    }
}

/// 支援技術に公開されるテキスト
///
/// `id` は支援技術のノードの id に使うため、ページ内で重ならない値を渡すこと
fn label(theme: &Theme, id: &'static str, text: impl Into<SharedString>) -> gpui::Div {
    let mut element = div();
    element
        .text_style()
        .refine(&theme.text_style(TypeScale::BodyMedium));
    element.child(gpui::text!(id = id, text.into()))
}
