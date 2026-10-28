//! タイプスケールを当てた文字。

use gpui::{App, IntoElement, RenderOnce, SharedString, Window, div, prelude::*};
use md3_theme::md3_color::ColorRole;
use md3_theme::{Md3ThemeExt, TypeScale};

use crate::StyledExt;

/// タイプスケールを当てた文字。色を指定しなければ親の文字色を引き継ぐ。
#[derive(IntoElement)]
pub struct Text {
    scale: TypeScale,
    text: SharedString,
    color: Option<ColorRole>,
}

impl Text {
    /// タイプスケールの種類と文字列から作る。
    pub fn new(scale: TypeScale, text: impl Into<SharedString>) -> Self {
        Self {
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
        let element = div()
            .type_style(&theme.text_style(self.scale))
            .child(self.text);
        match self.color {
            Some(role) => element.text_color(theme.color(role)),
            None => element,
        }
    }
}
