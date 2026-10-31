//! タイプスケールを当てた文字。

use gpui::{App, IntoElement, RenderOnce, SharedString, Window, div, prelude::*};
use md3_theme::md3_color::ColorRole;
use md3_theme::{Md3ThemeExt, TypeScale};

use crate::StyledExt;

/// タイプスケールを当てた文字。色を指定しなければ親の文字色を引き継ぐ。
#[derive(IntoElement)]
pub struct Text {
    id: Option<gpui::ElementId>,
    scale: TypeScale,
    text: SharedString,
    color: Option<ColorRole>,
}

impl Text {
    /// 読み上げる文字に ID を指定する。
    pub fn id(mut self, id: impl Into<gpui::ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// アクセシビリティ情報。
    pub fn a11y_node(&self) -> Option<gpui_headless::A11yNode> {
        self.id.as_ref().map(|_| {
            gpui_headless::A11yNode::new(gpui::Role::Label)
                .label(self.text.clone())
                .value(self.text.clone())
        })
    }

    /// タイプスケールの種類と文字列から作る。
    pub fn new(scale: TypeScale, text: impl Into<SharedString>) -> Self {
        Self {
            id: None,
            scale,
            text: text.into(),
            color: None,
        }
    }

    /// 文字色のロールを指定する。
    pub fn color(mut self, role: ColorRole) -> Self {
        self.color = Some(role);
        self
    }
}

impl RenderOnce for Text {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.md3();
        let node = self.a11y_node();
        let element = div()
            .type_style(&theme.text_style(self.scale))
            .child(self.text);
        let element = match self.color {
            Some(role) => element.text_color(theme.color(role)),
            None => element,
        };
        match (self.id, node) {
            (Some(id), Some(node)) => {
                use gpui_headless::A11yExt;
                element.id(id).a11y(node).into_any_element()
            }
            _ => element.into_any_element(),
        }
    }
}
