//! ポインターで押している間を追う領域

use std::rc::Rc;

use gpui::{
    AnyElement, App, Bounds, DispatchPhase, Div, Element, ElementId, Entity, GlobalElementId,
    InspectorElementId, InteractiveElement, IntoElement, LayoutId, MouseButton, MouseMoveEvent,
    MouseUpEvent, ParentElement, Pixels, RenderOnce, Stateful, StyleRefinement, Styled, Window,
    div,
};

/// 押している状態が変わったときに呼ぶハンドラ
type PressHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// ポインターの主ボタンで押している間を追う領域
///
/// 領域の中で主ボタンを押すと押している状態になり、ウィンドウのどこで離しても押していない状態に戻る。
/// 子孫がイベントの伝播を止めても状態を追う。キーボードの操作は押している状態にしない。
/// 支援技術のクリックのうち、gpui がマウスの押下と離しとして配送するものは、ほかのポインターの操作と同じく追う。
/// 支援技術には役割を公開しない
///
/// 見た目は [`Styled`] で呼び出し側が付ける
#[derive(IntoElement)]
pub struct PressArea {
    id: ElementId,
    inner: Stateful<Div>,
    disabled: bool,
    on_press_change: Option<PressHandler>,
}

impl PressArea {
    /// 押していない状態の領域を作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。`id` が変わるか 1 フレームでも描画されないと、
    /// 押している状態を失う
    pub fn new(id: impl Into<ElementId>) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            inner: div().id(id),
            disabled: false,
            on_press_change: None,
        }
    }

    /// `true` なら押しても押している状態にしない
    ///
    /// 押している間に `true` にすると、押していない状態に戻し、[`PressArea::on_press_change`] のハンドラを呼ぶ
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 押している状態が変わったときに、変わった後の状態でハンドラを呼ぶ
    pub fn on_press_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_press_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for PressArea {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let pressed = window.use_keyed_state(self.id.clone(), cx, |_, _| false);
        let handler = self.on_press_change;
        if self.disabled && *pressed.read(cx) {
            pressed.update(cx, |pressed, _| *pressed = false);
            if let Some(handler) = &handler {
                handler(false, window, cx);
            }
        }
        let inner = if self.disabled {
            self.inner
        } else {
            let pressed = pressed.clone();
            let handler = handler.clone();
            // 子孫が Bubble で伝播を止めても受けるよう、Capture で受ける
            self.inner.capture_any_mouse_down(move |event, window, cx| {
                if event.button != MouseButton::Left || *pressed.read(cx) {
                    return;
                }
                pressed.update(cx, |pressed, _| *pressed = true);
                if let Some(handler) = &handler {
                    handler(true, window, cx);
                }
            })
        };
        ReleaseLayer {
            inner,
            pressed,
            handler,
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

/// 押している間、ボタンを離す操作を領域の外でも受ける要素
struct ReleaseLayer<E> {
    inner: E,
    pressed: Entity<bool>,
    handler: Option<PressHandler>,
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
            let pressed = self.pressed.clone();
            let handler = self.handler.clone();
            move |window: &mut Window, cx: &mut App| {
                pressed.update(cx, |pressed, _| *pressed = false);
                if let Some(handler) = &handler {
                    handler(false, window, cx);
                }
            }
        };
        let pressed = self.pressed.clone();
        let release_on_move = release.clone();
        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
            // ウィンドウの外でボタンを離した場合は離す操作が届かないため、移動のときにも確かめる
            if phase == DispatchPhase::Capture
                && *pressed.read(cx)
                && event.pressed_button != Some(MouseButton::Left)
            {
                release_on_move(window, cx);
            }
        });
        let pressed = self.pressed.clone();
        window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
            if phase == DispatchPhase::Capture
                && event.button == MouseButton::Left
                && *pressed.read(cx)
            {
                release(window, cx);
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
    use std::cell::RefCell;

    use gpui::{Context, Modifiers, Render, TestAppContext, VisualTestContext, point, px};

    use super::*;
    use crate::test_window::add_test_window;

    struct View {
        disabled: bool,
        changes: Rc<RefCell<Vec<bool>>>,
    }

    impl Render for View {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let changes = self.changes.clone();
            div().size_full().child(
                PressArea::new("area")
                    .disabled(self.disabled)
                    .on_press_change(move |pressed, _, _| changes.borrow_mut().push(pressed))
                    .w(px(100.0))
                    .h(px(40.0))
                    .mt(px(10.0))
                    .ml(px(10.0))
                    // 子孫が伝播を止めても押している状態を追う
                    .child(
                        div()
                            .id("child")
                            .size_full()
                            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                            .on_mouse_up(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                            .capture_any_mouse_up(|_, _, cx| cx.stop_propagation()),
                    ),
            )
        }
    }

    fn open(
        cx: &mut TestAppContext,
    ) -> (Entity<View>, &mut VisualTestContext, Rc<RefCell<Vec<bool>>>) {
        let changes = Rc::new(RefCell::new(Vec::new()));
        let state = changes.clone();
        let (view, cx) = add_test_window(cx, move |_, _| View {
            disabled: false,
            changes: state,
        });
        (view, cx, changes)
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
        let (_, cx, changes) = open(cx);
        mouse_down(cx, 50.0);
        mouse_up(cx, 50.0);
        assert_eq!(*changes.borrow(), vec![true, false]);
    }

    #[gpui::test]
    fn 外で離しても戻る(cx: &mut TestAppContext) {
        let (_, cx, changes) = open(cx);
        mouse_down(cx, 50.0);
        mouse_up(cx, 300.0);
        assert_eq!(*changes.borrow(), vec![true, false]);
    }

    #[gpui::test]
    fn 外で押しても押している状態にならない(cx: &mut TestAppContext) {
        let (_, cx, changes) = open(cx);
        mouse_down(cx, 300.0);
        mouse_up(cx, 50.0);
        assert!(changes.borrow().is_empty());
    }

    #[gpui::test]
    fn 押している間に無効にすると戻り_無効な間は押しても変わらない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx, changes) = open(cx);
        mouse_down(cx, 50.0);
        view.update(cx, |view, cx| {
            view.disabled = true;
            cx.notify();
        });
        cx.run_until_parked();
        mouse_up(cx, 50.0);
        mouse_down(cx, 50.0);
        mouse_up(cx, 50.0);
        assert_eq!(*changes.borrow(), vec![true, false]);
    }
}
