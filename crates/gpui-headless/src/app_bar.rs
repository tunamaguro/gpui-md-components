//! アプリバーとその見出しのアクセシビリティ情報。

use gpui::{Div, ElementId, Role, SharedString, Stateful, div, prelude::*};

use crate::{A11yExt, A11yNode};

/// アプリバーとタイトルの見出し。
pub struct AppBar {
    id: ElementId,
    title: SharedString,
}

impl AppBar {
    /// ID とタイトルから作る。
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
        }
    }

    /// アプリバーのアクセシビリティ情報。
    pub fn a11y_node(&self) -> A11yNode {
        A11yNode::new(Role::Header).label(self.title.clone())
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
