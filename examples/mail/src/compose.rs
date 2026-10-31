//! メール作成ダイアログ。

use md3_components::gpui::{Context, Focusable, WeakEntity, div, prelude::*};
use md3_components::inputs::TextField;
use md3_components::overlays::{Dialog, DialogAction, SnackbarMessage};

use crate::{MailView, model::Draft};

impl MailView {
    pub(crate) fn open_compose(&mut self, draft: Option<Draft>, cx: &mut Context<Self>) {
        let draft = draft.unwrap_or(Draft {
            to: String::new(),
            subject: String::new(),
            body: String::new(),
        });
        self.to.set_text(draft.to, cx);
        self.subject.set_text(draft.subject, cx);
        self.body.set_text(draft.body, cx);
        self.to_error = false;
        self.compose_open = true;
        cx.notify();
    }

    pub(crate) fn compose_dialog(&self, view: &WeakEntity<Self>) -> Dialog {
        let close_view = view.clone();
        let dismiss_view = view.clone();
        let send_view = view.clone();
        let mut to = TextField::new("compose-to", &self.to)
            .label("To")
            .required(true)
            .error(self.to_error);
        if self.to_error {
            to = to.supporting_text("Enter an address containing @.");
        }
        Dialog::new("compose-dialog")
            .headline("Compose")
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(to)
                    .child(TextField::new("compose-subject", &self.subject).label("Subject"))
                    .child(TextField::new("compose-body", &self.body).label("Body")),
            )
            .action(DialogAction::new("Cancel", move |_, cx| {
                Self::update(&close_view, cx, |this| this.compose_open = false);
            }))
            .action(DialogAction::validating("Send", move |window, cx| {
                send_view
                    .update(cx, |this, cx| {
                        let draft = Draft {
                            to: this.to.text(cx).to_owned(),
                            subject: this.subject.text(cx).to_owned(),
                            body: this.body.text(cx).to_owned(),
                        };
                        let sent = this.mail.send(draft);
                        if sent {
                            this.to.set_text("", cx);
                            this.subject.set_text("", cx);
                            this.body.set_text("", cx);
                            this.compose_open = false;
                            this.to_error = false;
                            this.snackbar.update(cx, |host, cx| {
                                host.show(SnackbarMessage::new("Message sent"), cx);
                            });
                        } else {
                            this.to_error = true;
                            let focus = this.to.focus_handle(cx);
                            window.focus(&focus, cx);
                        }
                        cx.notify();
                        sent
                    })
                    .unwrap_or(false)
            }))
            .on_dismiss(move |_, cx| {
                Self::update(&dismiss_view, cx, |this| this.compose_open = false);
            })
    }
}
