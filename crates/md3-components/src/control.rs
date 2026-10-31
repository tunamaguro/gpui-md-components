//! 独自の操作要素の土台。

use gpui::{App, Div, ElementId, SharedString, Stateful, Window};

/// [`control`] が返す描画用の状態。
pub struct ControlState {
    /// `.group_hover()` や [`StateLayer`](crate::StateLayer) に渡すグループ名。
    pub group: SharedString,
    /// キーボードフォーカスを表示するか。[`FocusRing`](crate::FocusRing) や
    /// [`StateLayer::focused`](crate::StateLayer::focused) に渡す。
    pub focus_visible: bool,
    /// 無効状態か。
    pub disabled: bool,
}

/// 独自の操作要素の土台を作る。
///
/// フォーカス、Tab の止まり場所、[`crate::scrollbar::ScrollArea`] でのフォーカス追従を付けた要素を返す。
/// 親には `.relative()` が付いているので、役割・見た目・[`FocusRing`](crate::FocusRing) はここに足す。
pub fn control(
    id: impl Into<ElementId>,
    disabled: bool,
    window: &mut Window,
    cx: &mut App,
) -> (Stateful<Div>, ControlState) {
    let id = id.into();
    let group = SharedString::from(format!("md3-{id}"));
    let (base, state) = gpui_headless::control(
        &id,
        disabled,
        true,
        window,
        cx,
        crate::focus::reveal_margin(),
        group,
    );
    (
        base,
        ControlState {
            group: state.group,
            focus_visible: state.focus_visible,
            disabled: state.disabled,
        },
    )
}
