//! gpui 用の見た目を持たない操作基盤
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod accessibility;
#[cfg(test)]
mod test_window;
pub mod button;
pub mod choice;
mod navigation;

pub use navigation::{FocusNext, FocusPrevious, init};

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
