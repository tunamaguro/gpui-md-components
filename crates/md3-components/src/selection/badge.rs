//! Badge。

use gpui::{App, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, px};
use md3_theme::Md3ThemeExt;
use md3_theme::md3_tokens::component::badge as tokens;
use md3_theme::md3_tokens::layout;

use crate::StyledExt;

/// 件数や新着を示す小さな印。アイコンなどに重ねて置く。
///
/// 読み上げられないので、件数は重ねる先の要素のラベルにも含めること（例: 「通知、3 件」）。
#[derive(IntoElement)]
pub struct Badge {
    label: Option<SharedString>,
}

impl Badge {
    /// 文字のない小さな点。
    pub fn dot() -> Self {
        Self { label: None }
    }

    /// 件数などの文字を持つバッジ。
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: Some(label.into()),
        }
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.md3();
        match self.label {
            None => div()
                .size(px(tokens::SIZE))
                .corner_radius(theme.corner(tokens::SHAPE))
                .bg(theme.color(tokens::COLOR)),
            Some(label) => div()
                .min_h(px(tokens::LARGE_SIZE))
                .min_w(px(tokens::LARGE_SIZE))
                .px(px(layout::BADGE_HORIZONTAL_SPACE))
                .flex()
                .items_center()
                .justify_center()
                .corner_radius(theme.corner(tokens::LARGE_SHAPE))
                .bg(theme.color(tokens::LARGE_COLOR))
                .text_color(theme.color(tokens::LARGE_LABEL_TEXT_COLOR))
                .type_style(&theme.text_style(tokens::LARGE_LABEL_TEXT_FONT))
                .child(label),
        }
    }
}
