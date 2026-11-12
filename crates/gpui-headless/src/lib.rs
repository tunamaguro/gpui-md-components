//! gpui 用の見た目を持たない操作基盤
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod accessibility;
pub mod button;
pub mod choice;
mod focus_visible;
mod interaction;
mod navigation;
pub mod progress;
pub mod radio;
mod root;
mod roving;
pub mod slider;
pub mod tabs;
#[cfg(test)]
mod test_window;
pub mod text;
pub mod toggle;

pub use focus_visible::is_focus_visible;
pub use interaction::InteractionSource;
pub use root::{Root, root};

/// 部品を操作できるかどうか
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Availability {
    /// 操作でき、Tab で到達できる
    #[default]
    Enabled,
    /// 操作できず、Tab でも到達できない
    ///
    /// フォーカス中にこの状態になった場合は、フォーカスが外れるまでフォーカスを保つ
    Disabled,
    /// 操作できないが、Tab で到達できる
    ///
    /// 無効な理由を伝えたい場合や、周囲から存在を推測できず見つけてもらう必要がある場合に使う
    DisabledFocusable,
}

impl Availability {
    pub(crate) fn is_focusable(self) -> bool {
        matches!(
            self,
            Availability::Enabled | Availability::DisabledFocusable
        )
    }

    pub(crate) fn is_disabled(self) -> bool {
        matches!(
            self,
            Availability::Disabled | Availability::DisabledFocusable
        )
    }
}
/// この crate の部品が使うキー割り当てと操作を登録する。アプリの起動時に 1 回呼ぶ
pub fn init(cx: &mut gpui::App) {
    navigation::init(cx);
    focus_visible::init(cx);
}
