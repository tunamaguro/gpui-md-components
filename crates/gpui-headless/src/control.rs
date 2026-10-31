//! 操作要素（ボタン、チェックボックスなど）の共通部分。

use gpui::{
    App, Div, ElementId, FocusHandle, MouseButton, SharedString, Stateful, Window, div, prelude::*,
};
use std::rc::Rc;

use crate::{focus_handle, is_focus_visible};

/// 操作要素の土台の状態。
///
/// ホバー・押下の見た目には土台の `.hover()` / `.active()`、または子要素の
/// `.group_hover(state.group, ..)` / `.group_active(state.group, ..)` を使う。
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
    control_on(
        div().relative().flex_none(),
        id,
        disabled,
        tab_stop,
        window,
        cx,
        reveal_margin,
        group,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "内部の共通処理で、部品ごとの設定をそのまま受け取る"
)]
pub(crate) fn control_on(
    base: Div,
    id: &ElementId,
    disabled: bool,
    tab_stop: bool,
    window: &mut Window,
    cx: &mut App,
    reveal_margin: gpui::Pixels,
    group: SharedString,
) -> (Stateful<Div>, ControlState) {
    let mut base = base.id(id.clone()).group(group.clone());
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

#[derive(Clone)]
pub(crate) struct Roving {
    enabled: Rc<[bool]>,
    handles: Rc<[FocusHandle]>,
    target: Option<usize>,
}

impl Roving {
    pub(crate) fn new(
        items: impl IntoIterator<Item = (ElementId, bool)>,
        selected: impl IntoIterator<Item = usize>,
        window: &mut Window,
        cx: &mut App,
    ) -> Self {
        let (handles, enabled): (Vec<_>, Vec<_>) = items
            .into_iter()
            .map(|(id, disabled)| (focus_handle(id, window, cx), !disabled))
            .unzip();
        let target = selected
            .into_iter()
            .find(|&i| enabled.get(i).copied().unwrap_or(false))
            .or_else(|| enabled.iter().position(|&e| e));
        Self {
            enabled: enabled.into(),
            handles: handles.into(),
            target,
        }
    }

    pub(crate) fn is_tab_stop(&self, index: usize) -> bool {
        self.target == Some(index)
    }

    pub(crate) fn next(&self, from: usize, forward: bool) -> Option<usize> {
        step(&self.enabled, from, forward)
    }

    pub(crate) fn first(&self) -> Option<usize> {
        self.enabled.iter().position(|&e| e)
    }

    pub(crate) fn last(&self) -> Option<usize> {
        self.enabled.iter().rposition(|&e| e)
    }

    pub(crate) fn focus(&self, index: usize, window: &mut Window, cx: &mut App) {
        if let Some(handle) = self.handles.get(index) {
            window.focus(handle, cx);
        }
    }

    pub(crate) fn initial_focus(&self) -> Option<FocusHandle> {
        self.target
            .and_then(|index| self.handles.get(index).cloned())
    }
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

    struct RovingHarness {
        disabled: Vec<bool>,
        selected: Vec<usize>,
        expected: Option<usize>,
        rendered: Rc<std::cell::Cell<bool>>,
    }

    impl gpui::Render for RovingHarness {
        fn render(
            &mut self,
            window: &mut Window,
            cx: &mut gpui::Context<Self>,
        ) -> impl IntoElement {
            let roving = Roving::new(
                self.disabled
                    .iter()
                    .enumerate()
                    .map(|(i, &disabled)| (ElementId::Name(format!("item-{i}").into()), disabled)),
                self.selected.iter().copied(),
                window,
                cx,
            );
            assert_eq!(roving.target, self.expected);
            assert_eq!(roving.initial_focus().is_some(), self.expected.is_some());
            for i in 0..self.disabled.len() {
                assert_eq!(roving.is_tab_stop(i), self.expected == Some(i));
            }
            self.rendered.set(true);
            div()
        }
    }

    fn check_roving(
        cx: &mut gpui::TestAppContext,
        disabled: Vec<bool>,
        selected: Vec<usize>,
        expected: Option<usize>,
    ) {
        let rendered = Rc::new(std::cell::Cell::new(false));
        let (_, cx) = cx.add_window_view({
            let rendered = rendered.clone();
            move |_, _| RovingHarness {
                disabled,
                selected,
                expected,
                rendered,
            }
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        assert!(rendered.get());
    }

    #[gpui::test]
    fn disabled_selection_falls_back_to_first_enabled(cx: &mut gpui::TestAppContext) {
        check_roving(cx, vec![true, false, false], vec![0], Some(1));
    }

    #[gpui::test]
    fn all_disabled_has_no_target(cx: &mut gpui::TestAppContext) {
        check_roving(cx, vec![true, true], vec![0], None);
    }

    #[gpui::test]
    fn out_of_range_selections_are_skipped(cx: &mut gpui::TestAppContext) {
        check_roving(
            cx,
            vec![false, true, false],
            vec![usize::MAX, 1, 2],
            Some(2),
        );
    }
}
