//! 基準の要素に付ける Tooltip。`overlays` feature の分岐をここに閉じる。

use gpui::{App, Div, ElementId, SharedString, Stateful, Window};

#[derive(Default)]
pub(crate) struct AnchorTooltip {
    #[cfg_attr(not(feature = "overlays"), allow(dead_code))]
    text: Option<SharedString>,
}

impl AnchorTooltip {
    #[cfg(feature = "overlays")]
    pub(crate) fn set(&mut self, text: SharedString) {
        self.text = Some(text);
    }

    pub(crate) fn apply(
        &self,
        base: Stateful<Div>,
        _id: &ElementId,
        _focus_visible: bool,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        #[cfg(feature = "overlays")]
        if let Some(text) = &self.text {
            return crate::overlays::TooltipExt::md3_focus_tooltip(
                base,
                _id,
                text.clone(),
                _focus_visible,
                _window,
                _cx,
            );
        }
        base
    }
}
