//! 描画時の大きさで角丸を決める要素

use gpui::{
    App, Bounds, Corners, CornersRefinement, Div, Element, ElementId, GlobalElementId,
    InspectorElementId, IntoElement, LayoutId, Pixels, Styled, Window, px, size,
};
use md3_theme::SharedTheme;
use md3_theme::md3_tokens::shape::ShapeKey;

/// `inner` を形 `key` の角丸で描く要素
///
/// 割合の角は描画するときの大きさの短辺から求めるため、レイアウトで大きさが変わっても形を保つ。
/// [`Shaped::outset`] を指定すると、要素の縁から内側に `outset` だけ入った箱を `key` の形とし、
/// その形を外側へ `outset` だけ広げた角丸で描く。角丸以外の見た目とレイアウトは `inner` のまま
pub(crate) struct Shaped {
    inner: Div,
    theme: SharedTheme,
    key: ShapeKey,
    morph: Option<(ShapeKey, f32)>,
    outset: Pixels,
}

impl Shaped {
    pub(crate) fn new(inner: Div, theme: SharedTheme, key: ShapeKey) -> Self {
        Self {
            inner,
            theme,
            key,
            morph: None,
            outset: px(0.0),
        }
    }

    // フォーカスリングを描く部品の feature がすべて無効なときに未使用にならないよう、それらの feature で囲む
    #[cfg(feature = "button")]
    pub(crate) fn outset(mut self, outset: Pixels) -> Self {
        self.outset = outset;
        self
    }

    /// 形を `key` から `target` へ `phase` だけ近づける。`phase` が 0 なら `key`、1 なら `target` の角丸で描く
    #[cfg(feature = "icon-button")]
    pub(crate) fn morph(mut self, target: ShapeKey, phase: f32) -> Self {
        self.morph = Some((target, phase));
        self
    }
}

impl IntoElement for Shaped {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for Shaped {
    type RequestLayoutState = <Div as Element>::RequestLayoutState;
    type PrepaintState = <Div as Element>::PrepaintState;

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
        let mut radii = self.theme.corner_radii(self.key, shape_size);
        if let Some((target, phase)) = self.morph {
            // Compose の `rememberAnimatedShape` と同じく、角ごとの半径を線形に補間する
            let target = self.theme.corner_radii(target, shape_size);
            radii = Corners {
                top_left: radii.top_left + (target.top_left - radii.top_left) * phase,
                top_right: radii.top_right + (target.top_right - radii.top_right) * phase,
                bottom_right: radii.bottom_right
                    + (target.bottom_right - radii.bottom_right) * phase,
                bottom_left: radii.bottom_left + (target.bottom_left - radii.bottom_left) * phase,
            };
        }
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
