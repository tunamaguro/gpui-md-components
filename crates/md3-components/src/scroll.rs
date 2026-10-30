//! スクロール領域とフォーカス追従。
#[cfg(feature = "overlays")]
use gpui::prelude::*;
#[cfg(feature = "inputs")]
use gpui::{App, ElementId, FocusHandle, IntoElement, Window};
#[cfg(all(feature = "overlays", not(feature = "inputs")))]
use gpui::{App, ElementId, IntoElement, Window};
#[cfg(feature = "overlays")]
use gpui_headless::{NestedScroll, ScrollScope, scroll_handle};

#[cfg(feature = "inputs")]
/// フォーカスした要素を祖先のスクロール領域に表示する。
pub(crate) fn reveal_on_focus(
    id: &ElementId,
    focus: FocusHandle,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement + use<> {
    gpui_headless::reveal_on_focus(id, focus, window, cx, crate::focus::reveal_margin())
}

/// オーバーレイの本文。ヘッダーと操作ボタンを残したまま縮小できる領域。
#[cfg(feature = "overlays")]
pub(crate) fn content(
    parent: &ElementId,
    children: Vec<gpui::AnyElement>,
    scrollbar: bool,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement + use<> {
    let id = ElementId::NamedChild(parent.clone().into(), "scroll-content".into());
    let scroll = scroll_handle(&id, window, cx);
    let wheel = NestedScroll::new(&id, scroll.clone(), window, cx);
    ScrollScope::new(
        scroll.clone(),
        gpui::div()
            .id(id.clone())
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&scroll)
            .on_scroll(wheel.on_scroll())
            .child(gpui::div().flex().flex_col().children(children))
            .child(wheel.capture())
            .when(scrollbar, |this| {
                this.child(crate::Scrollbar::new(
                    ElementId::NamedChild(id.into(), "scrollbar".into()),
                    scroll,
                ))
            }),
    )
}
