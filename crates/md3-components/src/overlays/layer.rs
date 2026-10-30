//! オーバーレイの見た目の共通部分。

use gpui::{App, Div, ElementId, Stateful};
use gpui_headless::overlay::DismissHandler;
use md3_theme::Md3ThemeExt;
use md3_theme::md3_tokens::component::scrim as tokens;

/// テーマの色のスクリム（モーダルの後ろを暗くする幕）。
pub(crate) fn scrim(id: ElementId, dismiss: Option<DismissHandler>, cx: &App) -> Stateful<Div> {
    let color = cx
        .md3()
        .color(tokens::CONTAINER_COLOR)
        .opacity(tokens::CONTAINER_OPACITY);
    gpui_headless::overlay::scrim(id, color, dismiss)
}
