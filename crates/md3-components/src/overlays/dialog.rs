//! Dialog（基本のダイアログ）。

use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    AnyElement, App, ElementId, IntoElement, ParentElement, RenderOnce, Role, SharedString, Window,
    div, prelude::*, px,
};
use md3_theme::md3_tokens::component::dialog as tokens;
use md3_theme::md3_tokens::layout;
use md3_theme::{Elevation, Md3ThemeExt};

use super::layer::{
    DismissHandler, InitialFocus, MODAL_PRIORITY, dismiss_and_restore, modal_container,
    overlay_focus, restore_focus, scrim, window_layer,
};
use crate::buttons::{Button, ButtonVariant};
use crate::{Icon, IconName, StyledExt};
use gpui_headless::{A11yExt, A11yNode};

/// ダイアログの下に並ぶ操作（Text ボタン）。
pub struct DialogAction {
    label: SharedString,
    handler: DismissHandler,
}

impl DialogAction {
    /// ラベルと、押されたときの処理から作る。処理の中でダイアログを閉じる（描かないようにする）こと。
    pub fn new(
        label: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            handler: Rc::new(handler),
        }
    }
}

/// モーダルなダイアログ。開いている間だけ描く。
///
/// 開くとダイアログの中の最初の要素にフォーカスを移し、Tab / Shift-Tab はダイアログの中で回る。
/// Esc とスクリムの押下で `on_dismiss` を呼ぶ。`on_dismiss` か操作のボタンで閉じると、
/// 開く前にフォーカスしていた要素にフォーカスを戻す。
///
/// ```ignore
/// Dialog::new("delete", "削除しますか？")
///     .supporting_text("この操作は取り消せません。")
///     .action(DialogAction::new("キャンセル", close))
///     .action(DialogAction::new("削除", delete))
///     .on_dismiss(close)
/// ```
#[derive(IntoElement)]
pub struct Dialog {
    scrollbar: bool,
    id: ElementId,
    headline: SharedString,
    icon: Option<IconName>,
    supporting_text: Option<SharedString>,
    children: Vec<AnyElement>,
    actions: Vec<DialogAction>,
    on_dismiss: Option<DismissHandler>,
    alert: bool,
    inline: bool,
}

impl Dialog {
    /// スクロールバーの表示を切り替える（既定は表示）。
    pub fn scrollbar(mut self, visible: bool) -> Self {
        self.scrollbar = visible;
        self
    }

    /// ID と見出し（読み上げ用の名前を兼ねる）から作る。
    pub fn new(id: impl Into<ElementId>, headline: impl Into<SharedString>) -> Self {
        Self {
            scrollbar: true,
            id: id.into(),
            headline: headline.into(),
            icon: None,
            supporting_text: None,
            children: Vec::new(),
            actions: Vec::new(),
            on_dismiss: None,
            alert: false,
            inline: false,
        }
    }

    /// 見出しの上にアイコンを出す（見出しは中央揃えになる）。
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// 本文。
    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting_text = Some(text.into());
        self
    }

    /// 操作を足す（左から順に並ぶ）。
    pub fn action(mut self, action: DialogAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Esc、スクリムの押下で閉じるときの処理。
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }

    /// 確認・警告のダイアログにする（役割が `AlertDialog` になる）。
    pub fn alert(mut self, alert: bool) -> Self {
        self.alert = alert;
        self
    }

    /// 見本用に、スクリムなしでその場に描く（フォーカスは動かさない）。
    #[doc(hidden)]
    pub fn inline(mut self) -> Self {
        self.inline = true;
        self
    }

    /// アクセシビリティツリーに出す情報。
    pub fn a11y_node(&self) -> A11yNode {
        let role = if self.alert {
            Role::AlertDialog
        } else {
            Role::Dialog
        };
        A11yNode::new(role)
            .label(self.headline.clone())
            .modal(!self.inline)
    }
}

impl ParentElement for Dialog {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Dialog {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let node = self.a11y_node();
        let focus =
            (!self.inline).then(|| overlay_focus(&self.id, || InitialFocus::Within, window, cx));
        let theme = &cx.md3().clone();
        let centered = self.icon.is_some();
        let has_body = self.supporting_text.is_some() || !self.children.is_empty();

        let actions = (!self.actions.is_empty()).then(|| {
            let parent = Arc::new(self.id.clone());
            div()
                .flex()
                .flex_wrap()
                .justify_end()
                .gap(px(layout::DIALOG_BUTTON_SPACE))
                .children(self.actions.into_iter().enumerate().map(|(index, action)| {
                    let handler = action.handler;
                    let focus = focus.clone();
                    Button::new(
                        ElementId::NamedChild(parent.clone(), format!("action-{index}").into()),
                        action.label,
                    )
                    .variant(ButtonVariant::Text)
                    .on_click(move |_, window, cx| {
                        handler(window, cx);
                        if let Some(focus) = &focus {
                            restore_focus(focus, window, cx);
                        }
                    })
                }))
        });

        let mut body = div()
            .id(self.id.clone())
            .a11y(node)
            .flex()
            .flex_col()
            .min_w(px(layout::DIALOG_MIN_WIDTH))
            .max_w(px(layout::DIALOG_MAX_WIDTH))
            .max_h((window.viewport_size().height - px(layout::DIALOG_SPACE * 2.0)).max(px(0.0)))
            .p(px(layout::DIALOG_SPACE))
            .bg(theme.color(tokens::CONTAINER_COLOR))
            .corner_radius(theme.corner(tokens::CONTAINER_SHAPE))
            .shadow(theme.shadows(Elevation::from_dp(tokens::CONTAINER_ELEVATION)))
            .when_some(self.icon, |this, icon| {
                this.child(
                    div()
                        .flex()
                        .justify_center()
                        .pb(px(layout::DIALOG_ICON_BOTTOM_SPACE))
                        .child(
                            Icon::new(icon)
                                .size(px(tokens::ICON_SIZE))
                                .color(tokens::ICON_COLOR),
                        ),
                )
            })
            .child(
                div()
                    .flex_none()
                    .when(centered, |this| this.flex().justify_center().text_center())
                    .when(has_body, |this| {
                        this.pb(px(layout::DIALOG_TITLE_BOTTOM_SPACE))
                    })
                    .type_style(&theme.text_style(tokens::HEADLINE_FONT))
                    .text_color(theme.color(tokens::HEADLINE_COLOR))
                    .child(self.headline),
            )
            .child(crate::scroll::content(
                &self.id,
                self.supporting_text
                    .map(|text| {
                        div()
                            .type_style(&theme.text_style(tokens::SUPPORTING_TEXT_FONT))
                            .text_color(theme.color(tokens::SUPPORTING_TEXT_COLOR))
                            .child(text)
                            .into_any_element()
                    })
                    .into_iter()
                    .chain(self.children)
                    .collect(),
                self.scrollbar,
                window,
                cx,
            ))
            .when_some(actions, |this, actions| {
                this.child(
                    div()
                        .flex_none()
                        .pt(px(layout::DIALOG_TEXT_BOTTOM_SPACE))
                        .child(actions),
                )
            });

        let Some(focus) = focus else {
            return body.into_any_element();
        };
        let dismiss = dismiss_and_restore(self.on_dismiss, &focus);
        body = modal_container(body, &focus, dismiss.clone(), cx);
        let layer = scrim(
            ElementId::NamedChild(Arc::new(self.id), "scrim".into()),
            dismiss,
            cx,
        )
        .flex()
        .items_center()
        .justify_center()
        .child(body);
        window_layer(layer, MODAL_PRIORITY, window).into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialog_is_named_by_its_headline() {
        let node = Dialog::new("d", "削除しますか？").a11y_node();
        assert_eq!(node.role, Role::Dialog);
        assert_eq!(node.label.as_deref(), Some("削除しますか？"));
        assert_eq!(
            Dialog::new("d", "x").alert(true).a11y_node().role,
            Role::AlertDialog
        );
    }

    #[test]
    fn dialog_is_modal_unless_drawn_inline() {
        assert!(Dialog::new("d", "x").a11y_node().modal);
        assert!(!Dialog::new("d", "x").inline().a11y_node().modal);
    }
}
