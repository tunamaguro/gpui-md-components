//! 面（背景色・影・角丸を持つ入れ物）。

use gpui::{
    AnyElement, App, Div, IntoElement, ParentElement, RenderOnce, StyleRefinement, Styled, Window,
    div,
};
use md3_theme::md3_color::ColorRole;
use md3_theme::{Elevation, Md3ThemeExt, Shape, content_color};

use crate::StyledExt;

/// MD3 の面。背景のロールの色で塗り、対応する前景色を文字色にする。
///
/// 高さ（elevation）は影で表す。MD3 の 2021 仕様の surface tint（高さに応じた色の重ね）は
/// surface container ロールに置き換えられたため使わない。
#[derive(IntoElement)]
pub struct Surface {
    base: Div,
    color: ColorRole,
    content: Option<ColorRole>,
    elevation: Elevation,
    shape: Shape,
}

impl Surface {
    /// 背景のロールを指定して作る。
    pub fn new(color: ColorRole) -> Self {
        Self {
            base: div(),
            color,
            content: None,
            elevation: Elevation::Level0,
            shape: Shape::None,
        }
    }

    /// 文字色のロールを指定する。指定しなければ背景に対応する前景色（無ければ `OnSurface`）。
    pub fn content_color(mut self, role: ColorRole) -> Self {
        self.content = Some(role);
        self
    }

    /// 高さを指定する。
    pub fn elevation(mut self, elevation: Elevation) -> Self {
        self.elevation = elevation;
        self
    }

    /// 角丸の段階を指定する。
    pub fn shape(mut self, shape: Shape) -> Self {
        self.shape = shape;
        self
    }
}

impl ParentElement for Surface {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Surface {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Surface {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.md3();
        let content = self
            .content
            .or_else(|| content_color(self.color))
            .unwrap_or(ColorRole::OnSurface);
        self.base
            .bg(theme.color(self.color))
            .text_color(theme.color(content))
            .corner_radius(theme.corner(self.shape))
            .shadow(theme.shadows(self.elevation))
    }
}
