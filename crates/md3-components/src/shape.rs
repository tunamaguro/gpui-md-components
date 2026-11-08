//! 描画時の大きさで角丸を決める要素

use gpui::{
    App, Bounds, CornersRefinement, Element, ElementId, GlobalElementId, InspectorElementId,
    IntoElement, LayoutId, Pixels, Styled, Window, px, size,
};
use md3_theme::SharedTheme;
use md3_theme::md3_tokens::shape::ShapeKey;

/// `inner` を形 `key` の角丸で描く要素
///
/// 割合の角は描画するときの大きさの短辺から求めるため、レイアウトで大きさが変わっても形を保つ。
/// [`Shaped::outset`] を指定すると、要素の縁から内側に `outset` だけ入った箱を `key` の形とし、
/// その形を外側へ `outset` だけ広げた角丸で描く。角丸以外の見た目とレイアウトは `inner` のまま
pub(crate) struct Shaped<E> {
    inner: E,
    theme: SharedTheme,
    key: ShapeKey,
    outset: Pixels,
}

impl<E: Element + Styled> Shaped<E> {
    pub(crate) fn new(inner: E, theme: SharedTheme, key: ShapeKey) -> Self {
        Self {
            inner,
            theme,
            key,
            outset: px(0.0),
        }
    }

    pub(crate) fn outset(mut self, outset: Pixels) -> Self {
        self.outset = outset;
        self
    }
}

impl<E: Element + Styled> IntoElement for Shaped<E> {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl<E: Element + Styled> Element for Shaped<E> {
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
        // 角丸はレイアウトに影響せず、gpui の Div は prepaint と paint でスタイルを計算し直すため、
        // 大きさが決まったここで設定すれば描画に反映される
        let shape_size = size(
            bounds.size.width - self.outset * 2.0,
            bounds.size.height - self.outset * 2.0,
        );
        let radii = self.theme.corner_radii(self.key, shape_size);
        let outset = self.outset;
        self.inner.style().corner_radii = CornersRefinement {
            top_left: Some((radii.top_left + outset).into()),
            top_right: Some((radii.top_right + outset).into()),
            bottom_right: Some((radii.bottom_right + outset).into()),
            bottom_left: Some((radii.bottom_left + outset).into()),
        };
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
    }
}
