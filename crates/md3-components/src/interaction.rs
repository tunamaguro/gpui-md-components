//! 操作できる部品の hover とフォーカスを、別の部品に表示するための値

use gpui::{App, FocusHandle};

/// 操作できる部品の hover とフォーカスを、その部品の子孫の部品に表示するための値
///
/// 操作できる部品と、その子孫の見た目だけの部品の両方に同じ値を渡すと、操作できる部品の hover とフォーカスが
/// 見た目だけの部品に表示される。行全体を操作の対象にして、その中のチェックボックスに表示する場合などに使う
///
/// 呼び出し側の状態として作って持ち、描画のたびに同じ値を渡すこと。操作できる部品として渡すのは 1 つにすること
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
