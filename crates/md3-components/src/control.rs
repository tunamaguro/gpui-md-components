//! 操作要素の共通処理。

use gpui::{App, Div, ElementId, Stateful, Window};

#[cfg_attr(not(feature = "containment"), allow(dead_code))]
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
        crate::focus::reveal_margin(),
        format!("md3-{id}").into(),
    )
}
