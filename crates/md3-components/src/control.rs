//! 操作要素の共通処理。

#[cfg(any(
    feature = "buttons",
    feature = "selection",
    feature = "inputs",
    feature = "overlays",
    feature = "navigation",
    feature = "containment",
    feature = "expressive"
))]
use gpui::{App, Div, ElementId, Stateful, Window, px};
#[cfg(any(
    feature = "buttons",
    feature = "selection",
    feature = "inputs",
    feature = "overlays",
    feature = "navigation",
    feature = "containment",
    feature = "expressive"
))]
use md3_theme::md3_tokens::state;

#[cfg(any(
    feature = "buttons",
    feature = "selection",
    feature = "inputs",
    feature = "overlays",
    feature = "navigation",
    feature = "containment",
    feature = "expressive"
))]
pub(crate) fn control(
    id: &ElementId,
    disabled: bool,
    tab_stop: bool,
    window: &mut Window,
    cx: &mut App,
) -> (Stateful<Div>, gpui_headless::ControlState) {
    gpui_headless::control(
        id,
        disabled,
        tab_stop,
        window,
        cx,
        px(state::FOCUS_INDICATOR_THICKNESS + state::FOCUS_INDICATOR_OUTER_OFFSET),
        format!("md3-{id}").into(),
    )
}
