//! 開閉ボタンで表示を切り替える、見た目を持たない内容の箱

use gpui::{
    AnyElement, App, Div, ElementId, FocusHandle, InteractiveElement, Interactivity, IntoElement,
    ParentElement, RenderOnce, Stateful, StatefulInteractiveElement, StyleRefinement, Styled,
    Window, div,
};

/// 開閉ボタンで表示を切り替える、見た目を持たない内容の箱 (APG の Disclosure パターンの内容)
///
/// 開いているかどうかは呼び出し側が持ち、[`DisclosurePanel::new`] に毎回渡す。箱自身は状態を変えない。
/// 開いている間だけ子を描き、閉じている間は子を描かない空の箱になる。
/// 開閉ボタンには、開いているかどうかを `aria_expanded` で呼び出し側が付けること
///
/// - 閉じたとき、直前の描画でフォーカスが箱の中にあり、閉じた後にどの要素にもフォーカスが無ければ、
///   [`DisclosurePanel::new`] の `trigger` へフォーカスを移す。中の要素が消えてもフォーカスを失わない
///
/// 見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける
#[derive(IntoElement)]
pub struct DisclosurePanel {
    id: ElementId,
    inner: Stateful<Div>,
    expanded: bool,
    trigger: FocusHandle,
}

/// 箱のフォーカスと、直前の描画でフォーカスが箱の中にあったか
struct PanelState {
    focus: FocusHandle,
    focus_within: bool,
}

impl DisclosurePanel {
    /// `expanded` なら子を描く箱を作る。`trigger` は開閉ボタンのフォーカスにする
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>, expanded: bool, trigger: &FocusHandle) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            inner: div().id(id),
            expanded,
            trigger: trigger.clone(),
        }
    }
}

impl RenderOnce for DisclosurePanel {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| PanelState {
            focus: cx.focus_handle(),
            focus_within: false,
        });
        let focus = state.read(cx).focus.clone();
        if self.expanded {
            // contains_focused は前のフレームで判定する
            let within = focus.contains_focused(window, cx);
            state.update(cx, |state, _| state.focus_within = within);
            return self.inner.track_focus(&focus);
        }
        if state.read(cx).focus_within {
            state.update(cx, |state, _| state.focus_within = false);
            let trigger = self.trigger;
            // 描画中のフォーカスの移動は再描画されないため、描き終えてから移す。
            // 閉じる前に別の要素へフォーカスが移されていれば戻さない
            window.defer(cx, move |window, cx| {
                if window.focused(cx).is_none() {
                    window.focus(&trigger, cx);
                }
            });
        }
        div().id(self.id)
    }
}

impl Styled for DisclosurePanel {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl ParentElement for DisclosurePanel {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.inner.extend(elements);
    }
}

impl InteractiveElement for DisclosurePanel {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl StatefulInteractiveElement for DisclosurePanel {}

#[cfg(test)]
mod tests {
    use gpui::{Context, Entity, Render, TestAppContext, VisualTestContext, px};

    use super::*;
    use crate::button::Button;
    use crate::test_window::add_test_window;

    /// 開閉ボタンと、開いている間だけボタンを 1 つ入れた箱を描くビュー
    struct PanelView {
        expanded: bool,
        trigger: FocusHandle,
        other: FocusHandle,
    }

    impl Render for PanelView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .child(
                    Button::new("trigger")
                        .track_focus(&self.trigger)
                        .size(px(20.)),
                )
                .child(Button::new("other").track_focus(&self.other).size(px(20.)))
                .child(
                    DisclosurePanel::new("panel", self.expanded, &self.trigger)
                        .child(Button::new("inside").size(px(20.))),
                )
        }
    }

    fn setup(cx: &mut TestAppContext) -> (Entity<PanelView>, &mut VisualTestContext) {
        add_test_window(cx, |_, cx| PanelView {
            expanded: true,
            trigger: cx.focus_handle(),
            other: cx.focus_handle(),
        })
    }

    fn collapse(view: &Entity<PanelView>, cx: &mut VisualTestContext) {
        view.update(cx, |view, cx| {
            view.expanded = false;
            cx.notify();
        });
        cx.run_until_parked();
    }

    #[gpui::test]
    fn 中にフォーカスがあるまま閉じると開閉ボタンへフォーカスを移す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        cx.simulate_keystrokes("tab tab tab");
        let (trigger, other) =
            view.read_with(cx, |view, _| (view.trigger.clone(), view.other.clone()));
        assert!(
            cx.update(|window, cx| window.focused(cx).is_some()
                && !trigger.is_focused(window)
                && !other.is_focused(window)),
            "前提として 3 回目の Tab で箱の中のボタンにフォーカスがあること"
        );
        collapse(&view, cx);
        assert!(cx.update(|window, _| trigger.is_focused(window)));
    }

    #[gpui::test]
    fn 中にフォーカスが無ければ閉じてもフォーカスを動かさない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        let other = view.read_with(cx, |view, _| view.other.clone());
        cx.update(|window, cx| window.focus(&other, cx));
        cx.run_until_parked();
        collapse(&view, cx);
        assert!(cx.update(|window, _| other.is_focused(window)));
    }
}
