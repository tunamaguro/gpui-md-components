//! グループの中で、矢印キーで項目の間のフォーカスを移す仕組み
//!
//! グループの項目のうち Tab で到達できるのは 1 つだけにし、矢印キーで項目の間を移る (APG の roving tabindex)

use gpui::{AnyElement, FocusHandle, Window};

use crate::Availability;

/// グループの子。項目と、項目でない要素を渡された順に並べる
pub(crate) enum GroupChild<I> {
    Item(I),
    Other(AnyElement),
}

/// フォーカスの移動先を決めるための、項目の状態
pub(crate) struct RovingItem {
    pub(crate) focus_handle: FocusHandle,
    pub(crate) availability: Availability,
}

/// Tab で到達できるようにする項目の位置を返す
///
/// 項目にフォーカスがあればその項目にする。なければ `preferred` の項目、最初のフォーカスできる項目の順に選ぶ。
/// どれもなければ `None`
pub(crate) fn tab_stop(
    items: &[RovingItem],
    preferred: Option<usize>,
    window: &Window,
) -> Option<usize> {
    items
        .iter()
        .position(|item| item.focus_handle.is_focused(window))
        .or_else(|| preferred.filter(|&ix| items[ix].availability.is_focusable()))
        .or_else(|| {
            items
                .iter()
                .position(|item| item.availability.is_focusable())
        })
}

/// フォーカスのある項目から `forward` の向きに進んだ、フォーカスできる項目の位置を返す
///
/// 端では反対の端に戻る。どの項目にもフォーカスがない場合と、移れる項目がほかにない場合は `None`
pub(crate) fn neighbor(items: &[RovingItem], forward: bool, window: &Window) -> Option<usize> {
    let current = items
        .iter()
        .position(|item| item.focus_handle.is_focused(window))?;
    let len = items.len();
    (1..len)
        .map(|step| {
            if forward {
                (current + step) % len
            } else {
                (current + len - step) % len
            }
        })
        .find(|&ix| items[ix].availability.is_focusable())
}
