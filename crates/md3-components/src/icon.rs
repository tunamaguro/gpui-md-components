//! アイコン（md3-icons の SVG）。

use gpui::{App, Hsla, IntoElement, Pixels, RenderOnce, Window, prelude::*, px, svg};
use md3_icons::IconName;
use md3_theme::Md3ThemeExt;
use md3_theme::md3_color::ColorRole;

/// MD3 のアイコンの既定の大きさ（24dp）。
const DEFAULT_SIZE: f32 = 24.0;

/// アイコン。色を指定しなければ `OnSurface`。
///
/// 装飾ではないアイコンには、それを含む操作要素の側でラベルを付けること（アイコン自体はツリーに出ない）。
#[derive(IntoElement)]
pub struct Icon {
    name: IconName,
    size: Pixels,
    color: ColorRole,
    tint: Option<Hsla>,
}

impl Icon {
    /// アイコンを作る。
    pub fn new(name: IconName) -> Self {
        Self {
            name,
            size: px(DEFAULT_SIZE),
            color: ColorRole::OnSurface,
            tint: None,
        }
    }

    /// 大きさを変える。
    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    /// 色のロールを変える。
    pub fn color(mut self, role: ColorRole) -> Self {
        self.color = role;
        self
    }
}

impl Icon {
    /// 色を直接指定する（無効状態の半透明色など、ロールで表せない色）。ロールより優先する。
    pub fn tint(mut self, color: Hsla) -> Self {
        self.tint = Some(color);
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        svg()
            .data(self.name.svg())
            .flex_none()
            .size(self.size)
            .text_color(self.tint.unwrap_or_else(|| cx.md3().color(self.color)))
    }
}
