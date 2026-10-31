//! 独立したボタンをまとめるグループ。

use crate::{A11yExt, A11yNode, Button, ControlState};
use gpui::{
    AnyElement, App, Div, ElementId, Role, SharedString, Stateful, Window, div, prelude::*,
};

/// 各ボタンがそれぞれ Tab で止まるグループ。
pub struct ButtonGroup {
    id: ElementId,
    label: Option<SharedString>,
    items: Vec<Button>,
}

impl ButtonGroup {
    /// ID から作る。
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            items: Vec::new(),
        }
    }

    /// 読み上げる名前を指定する。表示文字を優先し、表示ラベルがある場合は名前に含める（WCAG 2.5.3）。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        self.label = (!label.is_empty()).then_some(label);
        self
    }

    /// グループのアクセシビリティ情報。
    pub fn a11y_node(&self) -> A11yNode {
        A11yNode::new(Role::Group).with_label(self.label.clone())
    }

    /// ボタンを追加する。
    pub fn item(mut self, button: Button) -> Self {
        self.items.push(button);
        self
    }

    /// 各ボタンの描画を受け取り、グループを返す。
    pub fn build(
        self,
        window: &mut Window,
        cx: &mut App,
        mut render: impl FnMut(ButtonItem, &mut Window, &mut App) -> AnyElement,
    ) -> Stateful<Div> {
        window.with_element_namespace(self.id.clone(), |window| {
            let node = self.a11y_node();
            let mut row = div().id(self.id).a11y(node);
            for (index, button) in self.items.into_iter().enumerate() {
                let id = button.id().clone();
                let (base, state) = button.build(window, cx);
                let child = window.with_element_namespace(id, |window| {
                    render(ButtonItem { base, state, index }, window, cx)
                });
                row = row.child(child);
            }
            row
        })
    }
}

/// 項目の要素と描画に使う状態。
pub struct ButtonItem {
    /// 操作を付けた土台。
    pub base: Stateful<Div>,
    /// 共通の操作状態。
    pub state: ControlState,
    /// 項目の位置。
    pub index: usize,
}
