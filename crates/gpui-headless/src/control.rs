//! 操作要素（ボタン、チェックボックスなど）の共通部分。

use gpui::{App, Div, ElementId, MouseButton, SharedString, Stateful, Window, div, prelude::*};

use crate::{focus_handle, is_focus_visible};

/// 操作要素の土台の状態。
pub struct ControlState {
    /// ホバーなどで参照するグループ名。
    pub group: SharedString,
    /// キーボードフォーカスを表示するか。
    pub focus_visible: bool,
    /// 無効状態か。
    pub disabled: bool,
}

/// 操作要素の土台を作る。
///
/// 指定されたグループ名を設定し、ID に対応するフォーカスハンドルを作る。
/// 無効でなければフォーカスできるようにし、`reveal_margin` を余白として表示位置を調整する。
/// `tab_stop` が `false` なら Tab では止まらない（グループ内を矢印キーで移る部品の、選択されていない項目）。
pub fn control(
    id: &ElementId,
    disabled: bool,
    tab_stop: bool,
    window: &mut Window,
    cx: &mut App,
    reveal_margin: gpui::Pixels,
    group: SharedString,
) -> (Stateful<Div>, ControlState) {
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
            .on_mouse_down(MouseButton::Left, {
                let handle = handle.clone();
                move |_, window, cx| crate::focus::clear_focus_visible(&handle, window, cx)
            })
            .cursor_pointer()
            .child(crate::reveal_on_focus(
                id,
                handle.clone(),
                window,
                cx,
                reveal_margin,
            ));
        is_focus_visible(&handle, window, cx)
    };
    (
        base,
        ControlState {
            group,
            focus_visible,
            disabled,
        },
    )
}

/// 矢印キーで移る先（無効な項目は飛ばし、端で反対側に回る）。
pub fn step(enabled: &[bool], from: usize, forward: bool) -> Option<usize> {
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
