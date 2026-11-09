//! 操作できる部品の hover とフォーカスを、別の部品に表示するための値

use gpui::{App, FocusHandle};

/// 操作できる部品の hover とフォーカスを、その部品の子孫の部品に表示するための値
///
/// [`crate::CheckboxRow`] に渡すと、行の hover とフォーカスが [`crate::CheckboxRow::checkbox`] で置いた
/// [`crate::CheckboxIndicator`] に表示される
///
/// 呼び出し側の状態として作って持ち、描画のたびに同じ値を渡すこと。1 つの値を渡す行は 1 つにすること
#[derive(Debug, Clone)]
pub struct InteractionSource(gpui_headless::InteractionSource);

impl InteractionSource {
    /// 新しいフォーカスを持つ値を作る
    pub fn new(cx: &mut App) -> Self {
        Self(gpui_headless::InteractionSource::new(cx))
    }

    /// 操作できる部品が使うフォーカス
    ///
    /// フォーカスの移動や判定に使う
    pub fn focus_handle(&self) -> &FocusHandle {
        self.0.focus_handle()
    }

    pub(crate) fn inner(&self) -> &gpui_headless::InteractionSource {
        &self.0
    }
}
