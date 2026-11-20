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
/// - 閉じたとき、フォーカスが箱の中の要素にあれば、[`DisclosurePanel::new`] の `trigger` へフォーカスを移す。
///   中の要素が消えてもフォーカスを失わない。閉じた描画の後にフォーカスが別の要素へ移されていれば戻さない
///
/// 見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける。付けたスタイルとハンドラは開いている間だけ働き、
/// 閉じている間は大きさを持たない箱としてレイアウトされる
#[derive(IntoElement)]
pub struct DisclosurePanel {
    id: ElementId,
    inner: Stateful<Div>,
    expanded: bool,
    trigger: FocusHandle,
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
        let focus = window
            .use_keyed_state(self.id.clone(), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        if self.expanded {
            return self.inner.track_focus(&focus);
        }
        // contains_focused は前のフレームの要素の木で判定するため、開いていた前のフレームの中にフォーカスがあったかが分かる。
        // 閉じたまま描き直す間は、前のフレームの箱がフォーカスを持たないため false になる
        if focus.contains_focused(window, cx) {
            let focused = window.focused(cx);
            let trigger = self.trigger;
            // 描画中のフォーカスの移動は再描画されないため、描き終えてから移す。
            // 消えた要素のフォーカスは `focused` が持ち続けるため、フォーカスが変わっていなければ消えた要素に残っている
            window.defer(cx, move |window, cx| {
                if window.focused(cx) == focused {
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
    use gpui::{Context, Entity, Modifiers, Render, TestAppContext, VisualTestContext, point, px};

    use super::*;
    use crate::button::Button;
    use crate::test_window::add_test_window;

    /// 開閉ボタンと、開いている間だけボタンを 1 つ入れた箱を縦に描くビュー。中のボタンを押すと閉じる
    struct PanelView {
        expanded: bool,
        trigger: FocusHandle,
        other: FocusHandle,
        /// 中のボタンのフォーカス。呼び出し側が持ち続ける
        inside: FocusHandle,
    }

    impl Render for PanelView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .child(
                    Button::new("trigger")
                        .track_focus(&self.trigger)
                        .size(px(20.)),
                )
                .child(Button::new("other").track_focus(&self.other).size(px(20.)))
                .child(
                    DisclosurePanel::new("panel", self.expanded, &self.trigger).child(
                        Button::new("inside")
                            .track_focus(&self.inside)
                            .size(px(20.))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.expanded = false;
                                cx.notify();
                            })),
                    ),
                )
        }
    }

    fn setup(cx: &mut TestAppContext) -> (Entity<PanelView>, &mut VisualTestContext) {
        add_test_window(cx, |_, cx| PanelView {
            expanded: true,
            trigger: cx.focus_handle(),
            other: cx.focus_handle(),
            inside: cx.focus_handle(),
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
        let (trigger, inside) =
            view.read_with(cx, |view, _| (view.trigger.clone(), view.inside.clone()));
        assert!(
            cx.update(|window, _| inside.is_focused(window)),
            "前提として 3 回目の Tab で箱の中のボタンにフォーカスがあること"
        );
        collapse(&view, cx);
        assert!(
            cx.update(|window, _| trigger.is_focused(window)),
            "呼び出し側が中のフォーカスを持ち続けていても開閉ボタンへ移すこと"
        );
    }

    #[gpui::test]
    fn 中のボタンのクリックで閉じても開閉ボタンへフォーカスを移す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        let trigger = view.read_with(cx, |view, _| view.trigger.clone());
        cx.update(|window, cx| window.focus(&trigger, cx));
        cx.run_until_parked();
        // 押下で中のボタンにフォーカスが移り、描画を挟まずに解放で閉じる
        cx.simulate_click(point(px(10.), px(50.)), Modifiers::none());
        cx.run_until_parked();
        assert!(
            !view.read_with(cx, |view, _| view.expanded),
            "前提としてクリックで閉じること"
        );
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
