//! ダイアログの役割とフォーカス。

use gpui::{App, Div, ElementId, Entity, Role, SharedString, Stateful, Window, div, prelude::*};

use crate::overlay::{InitialFocus, OverlayFocus, overlay_focus};
use crate::{A11yExt, A11yNode};

/// 見た目を持たないダイアログ。
pub struct Dialog {
    id: ElementId,
    label: Option<SharedString>,
    description: Option<SharedString>,
    aria_label: Option<SharedString>,
    alert: bool,
    modal: bool,
    complementary: bool,
}

impl Dialog {
    /// ID から作る。
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            description: None,
            aria_label: None,
            alert: false,
            modal: true,
            complementary: false,
        }
    }

    /// 表示する見出しを既定の名前にする。
    pub fn headline(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// 補足の説明を指定する。
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// 読み上げる名前を指定する。表示文字を優先し、表示ラベルがある場合は名前に含める（WCAG 2.5.3）。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// 緊急の確認にする（支援技術に `AlertDialog` と伝える）。
    pub fn alert(mut self, alert: bool) -> Self {
        self.alert = alert;
        self
    }

    /// モーダルにするか。`false` ならフォーカスを閉じ込めない。
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    /// 本文の脇にある補足の領域にする（支援技術に `Complementary` と伝える）。モーダルにはならない。
    pub fn complementary(mut self) -> Self {
        self.complementary = true;
        self.modal = false;
        self
    }

    /// アクセシビリティツリーに出す情報。
    pub fn a11y_node(&self) -> A11yNode {
        let role = if self.complementary {
            Role::Complementary
        } else if self.alert {
            Role::AlertDialog
        } else {
            Role::Dialog
        };
        let mut node = A11yNode::new(role)
            .with_label(self.aria_label.clone().or_else(|| self.label.clone()))
            .modal(self.modal);
        if let Some(description) = &self.description {
            node = node.description(description.clone());
        }
        node
    }

    /// 役割を付けた土台と、モーダルならフォーカスの状態を返す。
    pub fn build(
        self,
        window: &mut Window,
        cx: &mut App,
    ) -> (Stateful<Div>, Option<Entity<OverlayFocus>>) {
        let node = self.a11y_node();
        let focus = self
            .modal
            .then(|| overlay_focus(&self.id, || InitialFocus::Within, window, cx));
        (div().id(self.id).a11y(node), focus)
    }
}
