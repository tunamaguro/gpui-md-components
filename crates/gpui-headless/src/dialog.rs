//! ダイアログの役割とフォーカス。

use gpui::{App, Div, ElementId, Entity, Role, SharedString, Stateful, Window, div, prelude::*};

use crate::overlay::{InitialFocus, OverlayFocus, overlay_focus};
use crate::{A11yExt, A11yNode};

/// 見た目を持たないダイアログ。
pub struct Dialog {
    id: ElementId,
    label: SharedString,
    alert: bool,
    modal: bool,
    complementary: bool,
}

impl Dialog {
    /// ID と読み上げ用の名前から作る。既定でモーダル。
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            alert: false,
            modal: true,
            complementary: false,
        }
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
        A11yNode::new(role)
            .label(self.label.clone())
            .modal(self.modal)
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
