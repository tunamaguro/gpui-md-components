//! 操作要素（ボタン、チェックボックスなど）の共通部分。

use gpui::{App, Div, ElementId, SharedString, Stateful, Window, div, prelude::*};

use crate::{focus_handle, is_focus_visible};

/// 見本で状態を固定するための指定。gallery 用で、公開 API ではない。
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ForcedState {
    /// ホバー。
    Hovered,
    /// キーボードフォーカス。
    Focused,
    /// 押下。
    Pressed,
}

/// 操作要素の土台の状態。
pub(crate) struct Control {
    /// 状態レイヤーが参照するグループ名。
    pub group: SharedString,
    /// フォーカスリングを出すか。
    pub focus_visible: bool,
}

/// 操作要素の土台を作る。
///
/// ID からグループ名とフォーカスハンドルを作り、無効でなければフォーカスできるようにする。
/// `tab_stop` が `false` なら Tab では止まらない（グループ内を矢印キーで移る部品の、選択されていない項目）。
pub(crate) fn control(
    id: &ElementId,
    disabled: bool,
    tab_stop: bool,
    forced: Option<ForcedState>,
    window: &mut Window,
    cx: &mut App,
) -> (Stateful<Div>, Control) {
    let group: SharedString = format!("md3-{id}").into();
    let mut base = div()
        .id(id.clone())
        .group(group.clone())
        .relative()
        .flex_none();
    let mut focus_visible = forced == Some(ForcedState::Focused);
    if !disabled {
        let handle = focus_handle(id.clone(), window, cx).tab_stop(tab_stop);
        focus_visible |= is_focus_visible(&handle, window);
        base = base.track_focus(&handle).cursor_pointer();
    }
    (
        base,
        Control {
            group,
            focus_visible,
        },
    )
}

/// 矢印キーで移る先（無効な項目は飛ばし、端で反対側に回る）。
#[cfg_attr(not(any(feature = "buttons", feature = "selection")), allow(dead_code))]
pub(crate) fn step(enabled: &[bool], from: usize, forward: bool) -> Option<usize> {
    let count = enabled.len();
    (1..=count)
        .map(|offset| {
            if forward {
                (from + offset) % count
            } else {
                (from + count - offset % count) % count
            }
        })
        .find(|&index| enabled.get(index).copied().unwrap_or(false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrow_skips_disabled_and_wraps() {
        let enabled = [true, false, true];
        assert_eq!(step(&enabled, 0, true), Some(2));
        assert_eq!(step(&enabled, 2, true), Some(0));
        assert_eq!(step(&enabled, 0, false), Some(2));
    }
}
