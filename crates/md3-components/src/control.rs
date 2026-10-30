//! 操作要素（ボタン、チェックボックスなど）の共通部分。

use gpui::{App, Div, ElementId, SharedString, Stateful, Window, div, prelude::*};

use crate::{focus_handle, is_focus_visible};

/// 操作要素の土台の状態。部品を持つ feature がすべて無効だと使われない。
#[cfg_attr(
    not(any(
        feature = "buttons",
        feature = "selection",
        feature = "inputs",
        feature = "overlays",
        feature = "navigation",
        feature = "containment",
        feature = "expressive"
    )),
    allow(dead_code)
)]
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
#[cfg_attr(
    not(any(
        feature = "buttons",
        feature = "selection",
        feature = "inputs",
        feature = "overlays",
        feature = "navigation",
        feature = "containment",
        feature = "expressive"
    )),
    allow(dead_code)
)]
pub(crate) fn control(
    id: &ElementId,
    disabled: bool,
    tab_stop: bool,
    window: &mut Window,
    cx: &mut App,
) -> (Stateful<Div>, Control) {
    let group: SharedString = format!("md3-{id}").into();
    let mut base = div()
        .id(id.clone())
        .group(group.clone())
        .relative()
        .flex_none();
    let focus_visible = if disabled {
        false
    } else {
        let handle = focus_handle(id.clone(), window, cx).tab_stop(tab_stop);
        base = base
            .track_focus(&handle)
            .cursor_pointer()
            .child(crate::reveal_on_focus(id, handle.clone(), window, cx));
        is_focus_visible(&handle, window)
    };
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
