//! ドラッグ中のポインタを部品の外でも追う仕組み

use std::{cell::Cell, rc::Rc};

use gpui::{
    App, Bounds, DispatchPhase, Element, ElementId, Entity, GlobalElementId, InspectorElementId,
    IntoElement, LayoutId, MouseButton, MouseMoveEvent, MouseUpEvent, Pixels, Point, Window,
    accesskit,
};

/// ポインタのドラッグの状態。描画をまたいで保つ
#[derive(Default)]
pub(crate) struct PointerState {
    /// 左ボタンを部品の上で押してから離すまでの間か
    pub(crate) dragging: bool,
}

/// ポインタの位置と、部品の位置を受け取るハンドラ
pub(crate) type DragHandler = Rc<dyn Fn(Point<Pixels>, Bounds<Pixels>, &mut Window, &mut App)>;

/// `inner` の位置を記録し、ドラッグ中のマウスの移動とボタンを離す操作を、部品の外でも受ける要素
///
/// ドラッグは、部品の押下のハンドラで [`PointerState::dragging`] を `true` にして始める
pub(crate) struct PointerLayer<E> {
    pub(crate) inner: E,
    /// 描画のたびに `inner` の位置を書く
    pub(crate) bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    /// ドラッグ中の移動で呼ぶ。値を変えられない場合は `None`
    pub(crate) drag: Option<DragHandler>,
    /// ドラッグ中に部品の外も含めて左ボタンを離したときに呼ぶ
    pub(crate) release: Option<DragHandler>,
    pub(crate) pointer: Entity<PointerState>,
}

impl<E: Element> IntoElement for PointerLayer<E> {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl<E: Element> Element for PointerLayer<E> {
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
        self.bounds.set(Some(bounds));
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
        self.inner.paint(
            id,
            inspector_id,
            bounds,
            request_layout,
            prepaint,
            window,
            cx,
        );
        // gpui の要素のマウスのリスナーは、ポインタが要素の上にある間しか移動を受けないため、ウィンドウ全体で受ける。
        // 後から描かれた要素が Bubble で伝播を止めても受けるよう Capture で受け、
        // ドラッグ中に値を変えられなくなっても押下の状態を戻す
        let drag = self.drag.clone();
        let pointer = self.pointer.clone();
        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
            if phase != DispatchPhase::Capture || !pointer.read(cx).dragging {
                return;
            }
            // ウィンドウの外でボタンを離した場合は離す操作が届かないため、移動のときにも確かめる
            if event.pressed_button != Some(MouseButton::Left) {
                pointer.update(cx, |pointer, _| pointer.dragging = false);
                return;
            }
            if let Some(drag) = &drag {
                drag(event.position, bounds, window, cx);
            }
        });
        let pointer = self.pointer.clone();
        let release = self.release.clone();
        window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
            if phase == DispatchPhase::Capture
                && event.button == MouseButton::Left
                && pointer.read(cx).dragging
            {
                pointer.update(cx, |pointer, _| pointer.dragging = false);
                if let Some(release) = &release {
                    release(event.position, bounds, window, cx);
                }
            }
        });
    }

    fn a11y_role(&self) -> Option<accesskit::Role> {
        self.inner.a11y_role()
    }

    fn a11y_synthetic_children(
        &mut self,
        prepaint: &mut Self::PrepaintState,
        builder: &mut gpui::A11ySubtreeBuilder,
    ) {
        self.inner.a11y_synthetic_children(prepaint, builder);
    }

    fn write_a11y_info(&self, node: &mut accesskit::Node) {
        self.inner.write_a11y_info(node);
    }
}
