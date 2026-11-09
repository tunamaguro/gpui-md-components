//! グループの中で、矢印キーで項目の間のフォーカスを移す仕組み
//!
//! グループの項目のうち Tab で到達できるのは 1 つだけにし、矢印キーで項目の間を移る (APG の roving tabindex)

use std::collections::HashMap;

use gpui::{App, ElementId, Entity, FocusHandle, KeyBinding, Window, actions};

actions!(
    gpui_headless,
    [
        /// グループの中で、次の項目へフォーカスを移す。末尾の次は先頭に戻る
        NextItem,
        /// グループの中で、前の項目へフォーカスを移す。先頭の前は末尾に戻る
        PreviousItem,
    ]
);

/// [`crate::radio::RadioGroup`] の key context
pub(crate) const RADIO_GROUP_CONTEXT: &str = "GpuiHeadlessRadioGroup";
/// [`crate::toggle::ToggleGroup`] の key context
pub(crate) const TOGGLE_GROUP_CONTEXT: &str = "GpuiHeadlessToggleGroup";

/// グループの中で項目を移るキー割り当てを登録する。[`crate::init`] から 1 回呼ぶ
///
/// RadioGroup では上下左右のすべての矢印キーで、ToggleGroup では左右の矢印キーで移る
pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("down", NextItem, Some(RADIO_GROUP_CONTEXT)),
        KeyBinding::new("right", NextItem, Some(RADIO_GROUP_CONTEXT)),
        KeyBinding::new("up", PreviousItem, Some(RADIO_GROUP_CONTEXT)),
        KeyBinding::new("left", PreviousItem, Some(RADIO_GROUP_CONTEXT)),
        KeyBinding::new("right", NextItem, Some(TOGGLE_GROUP_CONTEXT)),
        KeyBinding::new("left", PreviousItem, Some(TOGGLE_GROUP_CONTEXT)),
    ]);
}

/// グループが内部で作った、項目の id ごとのフォーカス
pub(crate) type GroupFocusHandles = HashMap<ElementId, FocusHandle>;

/// グループ `group_id` が `ids` の項目に使うフォーカスを返す
///
/// `provided` が `Some` の項目はそれを使う。`None` の項目には、グループが id ごとに作ったフォーカスを、
/// 同じ id で描画され続ける間は同じものを返す。描画されなくなった id のフォーカスは捨てる
pub(crate) fn focus_handles(
    group_id: &ElementId,
    ids: impl IntoIterator<Item = (ElementId, Option<FocusHandle>)>,
    window: &mut Window,
    cx: &mut App,
) -> Vec<FocusHandle> {
    let state: Entity<GroupFocusHandles> =
        window.use_keyed_state(group_id.clone(), cx, |_, _| HashMap::new());
    state.update(cx, |handles, cx| {
        let mut kept = HashMap::new();
        let result = ids
            .into_iter()
            .map(|(id, provided)| {
                provided.unwrap_or_else(|| {
                    let handle = handles
                        .remove(&id)
                        .unwrap_or_else(|| cx.focus_handle().tab_index(0));
                    kept.insert(id, handle.clone());
                    handle
                })
            })
            .collect();
        *handles = kept;
        result
    })
}

/// フォーカスの移動先を決めるための、項目の状態
#[derive(Clone)]
pub(crate) struct RovingItem {
    pub(crate) focus_handle: FocusHandle,
    /// フォーカスを受け取れるか。フォーカス中に無効になった項目も含む
    pub(crate) focusable: bool,
    pub(crate) disabled: bool,
}

/// Tab で到達できるようにする項目の位置を返す
///
/// 項目にフォーカスがあればその項目にする。なければ `preferred` の項目、最初の有効な項目、
/// 最初のフォーカスできる項目の順に選ぶ。どれもなければ `None`
pub(crate) fn tab_stop(
    items: &[RovingItem],
    preferred: Option<usize>,
    window: &Window,
) -> Option<usize> {
    items
        .iter()
        .position(|item| item.focus_handle.is_focused(window))
        .or_else(|| preferred.filter(|&ix| items[ix].focusable))
        .or_else(|| {
            items
                .iter()
                .position(|item| item.focusable && !item.disabled)
        })
        .or_else(|| items.iter().position(|item| item.focusable))
}

/// フォーカスのある項目から `forward` の向きに進んだ、フォーカスできる項目の位置を返す
///
/// 端では反対の端に戻る。グループの項目にフォーカスがない場合と、移れる項目がほかにない場合は `None`
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
        .find(|&ix| items[ix].focusable)
}
