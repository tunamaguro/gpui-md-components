//! ポインターで押している間を追う領域

use gpui::{
    AnyElement, App, Bounds, DispatchPhase, Div, Element, ElementId, Entity, GlobalElementId,
    InspectorElementId, InteractiveElement, Interactivity, IntoElement, LayoutId, MouseButton,
    MouseMoveEvent, MouseUpEvent, ParentElement, Pixels, RenderOnce, Stateful, StyleRefinement,
    Styled, Window, div,
};

/// ポインターの主ボタンで押している間を追う領域
///
/// 領域の中で主ボタンを押すと `state` を `true` にし、ウィンドウのどこで離しても `false` に戻す。
/// 変えるたびに `state` を通知する。子孫がイベントの伝播を止めても状態を追う。キーボードの操作は押している状態にしない。
/// 支援技術のクリックのうち、gpui がマウスの押下と離しとして配送するものは、ほかのポインターの操作と同じく追う。
/// 支援技術には役割を公開しない
///
/// 離す操作は領域を描いている間だけ受ける。押している間に領域を描かなくなると、`state` は `true` のまま残るため、
/// 描き続ける [`PressScope`] に `state` を渡すか、描かなくする側で `false` に戻す。
/// 領域より先に描いた要素が離す操作の伝播を Capture で止めると、ポインターを動かすまで `false` に戻らない。
/// 並べた領域では、先に描く兄弟が止めることがあるため、[`PressScope`] で包む
///
/// 見た目は [`Styled`] で呼び出し側が付ける
#[derive(IntoElement)]
pub struct PressArea {
    inner: Stateful<Div>,
    state: Entity<bool>,
    disabled: bool,
}

impl PressArea {
    /// 押している状態を `state` に書く領域を作る
    ///
    /// `id` は同じ親の中で一意にする
    pub fn new(id: impl Into<ElementId>, state: &Entity<bool>) -> Self {
        Self {
            inner: div().id(id),
            state: state.clone(),
            disabled: false,
        }
    }

    /// `true` なら押しても押している状態にしない
    ///
    /// 押している間に `true` にすると、`state` を `false` に戻す
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl RenderOnce for PressArea {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = self.state;
        if self.disabled && *state.read(cx) {
            state.update(cx, |pressed, cx| {
                *pressed = false;
                cx.notify();
            });
        }
        let inner = if self.disabled {
            self.inner
        } else {
            let state = state.clone();
            // 子孫が Bubble で伝播を止めても受けるよう、Capture で受ける
            self.inner.capture_any_mouse_down(move |event, _, cx| {
                if event.button == MouseButton::Left && !*state.read(cx) {
                    state.update(cx, |pressed, cx| {
                        *pressed = true;
                        cx.notify();
                    });
                }
            })
        };
        ReleaseLayer {
            inner,
            states: vec![state],
        }
    }
}

impl Styled for PressArea {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl ParentElement for PressArea {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.inner.extend(elements);
    }
}

/// 並べた [`PressArea`] の押している状態を、子孫のどの要素よりも先に、離す操作で戻す要素
///
/// 子孫の領域の `state` を渡す。子孫がどの順で描かれても、離す操作の伝播を止めても、渡した状態を `false` に戻す。
/// 支援技術には役割を公開しない
///
/// 見た目は [`Styled`] で呼び出し側が付ける
#[derive(IntoElement)]
pub struct PressScope {
    inner: Stateful<Div>,
    states: Vec<Entity<bool>>,
}

impl PressScope {
    /// `states` を、離す操作で `false` に戻す要素を作る
    ///
    /// `id` は同じ親の中で一意にする。子孫の要素の id はこの `id` の中で解決する
    pub fn new(id: impl Into<ElementId>, states: impl IntoIterator<Item = Entity<bool>>) -> Self {
        Self {
            inner: div().id(id),
            states: states.into_iter().collect(),
        }
    }
}

impl RenderOnce for PressScope {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        ReleaseLayer {
            inner: self.inner,
            states: self.states,
        }
    }
}

impl Styled for PressScope {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl ParentElement for PressScope {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.inner.extend(elements);
    }
}

impl InteractiveElement for PressScope {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

/// 押している間、ボタンを離す操作を領域の外でも受ける要素
struct ReleaseLayer<E> {
    inner: E,
    states: Vec<Entity<bool>>,
}

impl<E: Element> IntoElement for ReleaseLayer<E> {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl<E: Element> Element for ReleaseLayer<E> {
    type RequestLayoutState = E::RequestLayoutState;
    type PrepaintState = E::PrepaintState;

    fn id(&self) -> Option<ElementId> {
        self.inner.id()
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        self.inner.source_location()
    }

    fn request_layout(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        self.inner.request_layout(id, inspector_id, window, cx)
    }

    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        self.inner
            .prepaint(id, inspector_id, bounds, request_layout, window, cx)
    }

    fn paint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        // gpui の要素のマウスのリスナーは、ポインタが要素の上にある間しか受けないため、ウィンドウ全体で受ける。
        // 子孫が伝播を止めても受けるよう Capture で受け、子孫の Capture のリスナーより先に受けるよう子孫より先に登録する
        let release = {
            let states = self.states.clone();
            move |cx: &mut App| {
                for state in &states {
                    if *state.read(cx) {
                        state.update(cx, |pressed, cx| {
                            *pressed = false;
                            cx.notify();
                        });
                    }
                }
            }
        };
        let release_on_move = release.clone();
        window.on_mouse_event(move |event: &MouseMoveEvent, phase, _, cx| {
            // ウィンドウの外でボタンを離した場合は離す操作が届かないため、移動のときにも確かめる
            if phase == DispatchPhase::Capture && event.pressed_button != Some(MouseButton::Left) {
                release_on_move(cx);
            }
        });
        window.on_mouse_event(move |event: &MouseUpEvent, phase, _, cx| {
            if phase == DispatchPhase::Capture && event.button == MouseButton::Left {
                release(cx);
            }
        });
        self.inner.paint(
            id,
            inspector_id,
            bounds,
            request_layout,
            prepaint,
            window,
            cx,
        );
    }
}

#[cfg(test)]
mod tests {
    use gpui::{
        AppContext as _, Context, Modifiers, Render, TestAppContext, VisualTestContext, point, px,
    };

    use super::*;
    use crate::test_window::add_test_window;

    /// 離す操作の伝播を Bubble と Capture の両方で止める子
    fn blocker(id: &'static str) -> impl IntoElement {
        div()
            .id(id)
            .size_full()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_mouse_up(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .capture_any_mouse_up(|_, _, cx| cx.stop_propagation())
    }

    /// 左上 (10, 10) から幅 100 の領域。`scoped` なら 2 つ目の領域を右に並べて [`PressScope`] で包む
    struct View {
        disabled: bool,
        scoped: bool,
        states: [Entity<bool>; 2],
    }

    impl Render for View {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let area = |index: usize, id: &'static str| {
                PressArea::new(id, &self.states[index])
                    .disabled(self.disabled)
                    .w(px(100.0))
                    .h(px(40.0))
                    .child(blocker(if index == 0 { "first" } else { "second" }))
            };
            if self.scoped {
                PressScope::new("scope", self.states.clone())
                    .flex()
                    .mt(px(10.0))
                    .ml(px(10.0))
                    .child(area(0, "a"))
                    .child(area(1, "b"))
                    .into_any_element()
            } else {
                div()
                    .mt(px(10.0))
                    .ml(px(10.0))
                    .child(area(0, "a"))
                    .into_any_element()
            }
        }
    }

    fn open(cx: &mut TestAppContext, scoped: bool) -> (Entity<View>, &mut VisualTestContext) {
        add_test_window(cx, move |_, cx| View {
            disabled: false,
            scoped,
            states: [cx.new(|_| false), cx.new(|_| false)],
        })
    }

    fn pressed(view: &Entity<View>, cx: &mut VisualTestContext) -> [bool; 2] {
        view.update(cx, |view, cx| {
            view.states.clone().map(|state| *state.read(cx))
        })
    }

    fn mouse_down(cx: &mut VisualTestContext, x: f32) {
        cx.simulate_mouse_down(point(px(x), px(30.0)), MouseButton::Left, Modifiers::none());
        cx.run_until_parked();
    }

    fn mouse_up(cx: &mut VisualTestContext, x: f32) {
        cx.simulate_mouse_up(point(px(x), px(30.0)), MouseButton::Left, Modifiers::none());
        cx.run_until_parked();
    }

    #[gpui::test]
    fn 中で押すと押している状態になり_中で離すと戻る(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = open(cx, false);
        mouse_down(cx, 50.0);
        assert_eq!(pressed(&view, cx), [true, false]);
        mouse_up(cx, 50.0);
        assert_eq!(pressed(&view, cx), [false, false]);
    }

    #[gpui::test]
    fn 外で離しても戻る(cx: &mut TestAppContext) {
        let (view, cx) = open(cx, false);
        mouse_down(cx, 50.0);
        mouse_up(cx, 300.0);
        assert_eq!(pressed(&view, cx), [false, false]);
    }

    #[gpui::test]
    fn 外で押しても押している状態にならない(cx: &mut TestAppContext) {
        let (view, cx) = open(cx, false);
        mouse_down(cx, 300.0);
        assert_eq!(pressed(&view, cx), [false, false]);
    }

    #[gpui::test]
    fn 押している間に無効にすると戻り_無効な間は押しても変わらない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = open(cx, false);
        mouse_down(cx, 50.0);
        view.update(cx, |view, cx| {
            view.disabled = true;
            cx.notify();
        });
        cx.run_until_parked();
        assert_eq!(pressed(&view, cx), [false, false]);
        mouse_up(cx, 50.0);
        mouse_down(cx, 50.0);
        assert_eq!(pressed(&view, cx), [false, false]);
    }

    #[gpui::test]
    fn 範囲で包むと_先に描いた兄弟が離す操作を止めても戻る(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = open(cx, true);
        mouse_down(cx, 150.0);
        assert_eq!(pressed(&view, cx), [false, true]);
        // 先に描いた 1 つ目の領域の子が、離す操作の伝播を Capture で止める
        mouse_up(cx, 50.0);
        assert_eq!(pressed(&view, cx), [false, false]);
    }
}
