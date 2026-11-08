//! 部品が共通で使う処理

use gpui::{App, ElementId, FocusHandle, Window};

/// `id` に結び付いた、部品が内部で持つフォーカスを返す
///
/// 同じ `id` で描画し続ける間は同じフォーカスを返す。`id` が変わるか 1 フレームでも描画されないと、新しいフォーカスになる
pub(crate) fn keyed_focus_handle(id: &ElementId, window: &mut Window, cx: &mut App) -> FocusHandle {
    window
        .use_keyed_state(id.clone(), cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone()
}
