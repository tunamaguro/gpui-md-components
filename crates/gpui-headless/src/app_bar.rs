//! アプリバーとその見出しのアクセシビリティ情報。

use gpui::{Div, ElementId, Role, SharedString, Stateful, div, prelude::*};

use crate::{A11yExt, A11yNode};

/// アプリバーとタイトルの見出し。
pub struct AppBar {
    id: ElementId,
    title: SharedString,
    aria_label: Option<Option<SharedString>>,
}

impl AppBar {
    /// ID とタイトルから作る。
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            aria_label: None,
        }
    }

    /// 読み上げる名前を指定する。表示文字を優先し、表示ラベルがある場合は名前に含める（WCAG 2.5.3）。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        self.aria_label = Some((!label.is_empty()).then_some(label));
        self
    }

    /// アプリバーのアクセシビリティ情報。
    pub fn a11y_node(&self) -> A11yNode {
        A11yNode::new(Role::Header).with_label(
            self.aria_label
                .clone()
                .unwrap_or_else(|| Some(self.title.clone())),
        )
    }

    /// タイトルのアクセシビリティ情報。
    pub fn heading_a11y_node(&self) -> A11yNode {
        A11yNode::new(Role::Heading).label(self.title.clone())
    }

    /// アプリバーの土台を作る。
    pub fn build(&self) -> Stateful<Div> {
        div().id(self.id.clone()).a11y(self.a11y_node())
    }

    /// タイトルの土台を作る。
    pub fn build_heading(&self) -> Stateful<Div> {
        div()
            .id(ElementId::NamedChild(
                self.id.clone().into(),
                "title".into(),
            ))
            .a11y(self.heading_a11y_node())
    }
}
