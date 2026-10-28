//! Divider（区切り線）。

use gpui::{App, IntoElement, Pixels, RenderOnce, Window, div, prelude::*, px};
use md3_theme::Md3ThemeExt;
use md3_theme::md3_tokens::component::divider as tokens;

/// 区切り線の向き。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DividerAxis {
    /// 横線（上下に並んだ内容を区切る）。
    #[default]
    Horizontal,
    /// 縦線（左右に並んだ内容を区切る）。
    Vertical,
}

/// 区切り線（太さ 1dp、`OutlineVariant`）。装飾なのでアクセシビリティツリーには出さない。
#[derive(IntoElement)]
pub struct Divider {
    axis: DividerAxis,
    inset_start: Pixels,
    inset_end: Pixels,
}

impl Divider {
    /// 横線を作る。
    pub fn horizontal() -> Self {
        Self {
            axis: DividerAxis::Horizontal,
            inset_start: px(0.0),
            inset_end: px(0.0),
        }
    }

    /// 縦線を作る。
    pub fn vertical() -> Self {
        Self {
            axis: DividerAxis::Vertical,
            ..Self::horizontal()
        }
    }

    /// 始端（横線なら左、縦線なら上）を空ける（inset divider）。
    pub fn inset_start(mut self, inset: Pixels) -> Self {
        self.inset_start = inset;
        self
    }

    /// 終端（横線なら右、縦線なら下）を空ける。
    pub fn inset_end(mut self, inset: Pixels) -> Self {
        self.inset_end = inset;
        self
    }
}

impl RenderOnce for Divider {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let line = div().flex_none().bg(cx.md3().color(tokens::COLOR));
        let thickness = px(tokens::THICKNESS);
        match self.axis {
            DividerAxis::Horizontal => line
                .h(thickness)
                .ml(self.inset_start)
                .mr(self.inset_end)
                .self_stretch(),
            DividerAxis::Vertical => line
                .w(thickness)
                .mt(self.inset_start)
                .mb(self.inset_end)
                .self_stretch(),
        }
    }
}
