//! 描画時の大きさで角丸を決める要素

use gpui::{
    App, Bounds, Corners, CornersRefinement, Div, Element, ElementId, GlobalElementId,
    InspectorElementId, IntoElement, LayoutId, Pixels, Styled, Window, px, size,
};
use md3_theme::SharedTheme;
use md3_theme::md3_tokens::shape::{CornerShape, ShapeKey};

/// `inner` を形 `key` の角丸で描く要素
///
/// 割合の角は描画するときの大きさの短辺から求めるため、レイアウトで大きさが変わっても形を保つ。
/// [`Shaped::outset`] を指定すると、要素の縁から内側に `outset` だけ入った箱を `key` の形とし、
/// その形を外側へ `outset` だけ広げた角丸で描く。角丸以外の見た目とレイアウトは `inner` のまま
pub(crate) struct Shaped {
    inner: Div,
    theme: SharedTheme,
    key: ShapeKey,
    /// 下の 2 つの角の形。`None` なら `key`
    bottom_key: Option<ShapeKey>,
    /// `key` の代わりに使う角ごとの形
    corner_shape: Option<CornerShape>,
    /// 形を変える動きの途中の形と進み。[`Shaped::morph_through`] を参照
    morph: Option<(Vec<CornerShape>, f32)>,
    outset: Pixels,
    /// 形の角丸を付ける角。`false` の角は直角にする
    rounded: Corners<bool>,
}

impl Shaped {
    pub(crate) fn new(inner: Div, theme: SharedTheme, key: ShapeKey) -> Self {
        Self {
            inner,
            theme,
            key,
            bottom_key: None,
            corner_shape: None,
            morph: None,
            outset: px(0.0),
            rounded: Corners {
                top_left: true,
                top_right: true,
                bottom_right: true,
                bottom_left: true,
            },
        }
    }

    // フォーカスリングを描く部品の feature がすべて無効なときに未使用にならないよう、それらの feature で囲む
    #[cfg(any(
        feature = "card",
        feature = "chip",
        feature = "fab",
        feature = "icon-button",
        feature = "list",
        feature = "menu",
        feature = "navigation-bar",
        feature = "navigation-rail",
        feature = "slider"
    ))]
    pub(crate) fn outset(mut self, outset: Pixels) -> Self {
        self.outset = outset;
        self
    }

    /// 形を `key` から `target` へ `phase` だけ近づける。`phase` が 0 なら `key`、1 なら `target` の角丸で描く
    #[cfg(feature = "icon-button")]
    pub(crate) fn morph(self, target: ShapeKey, phase: f32) -> Self {
        let target = target.resolve(&self.theme.settings().shapes);
        self.morph_through(vec![target], phase)
    }

    /// 形を、`phase` が 0 なら元の形、`i` (1 以上) なら `shapes[i - 1]` の角丸で描く
    ///
    /// 間の値は隣り合う 2 つの形の角ごとの半径を線形に補間する。範囲の外の値は端の 2 つの形から外挿する
    #[cfg(feature = "button")]
    pub(crate) fn morph_through(mut self, shapes: Vec<CornerShape>, phase: f32) -> Self {
        self.morph = Some((shapes, phase));
        self
    }

    /// `key` の代わりに、角ごとの形 `shape` で描く。[`Shaped::morph`] の動きは `shape` から始める
    #[cfg(feature = "button")]
    pub(crate) fn corner_shape(mut self, shape: CornerShape) -> Self {
        self.corner_shape = Some(shape);
        self
    }

    /// 下の 2 つの角だけを `key` の形にする
    #[cfg(feature = "menu")]
    pub(crate) fn bottom_corners(mut self, key: ShapeKey) -> Self {
        self.bottom_key = Some(key);
        self
    }

    /// 形の角丸を `rounded` が `true` の角だけに付け、ほかの角は直角にする
    #[cfg(feature = "segmented-button")]
    pub(crate) fn only_corners(mut self, rounded: Corners<bool>) -> Self {
        self.rounded = rounded;
        self
    }
}

impl Shaped {
    /// 大きさが `shape_size` の形の角の半径。[`Shaped::outset`] と、直角にする角は含めない
    pub(crate) fn radii(&self, shape_size: gpui::Size<Pixels>) -> Corners<Pixels> {
        let mut radii = match self.corner_shape {
            Some(shape) => md3_theme::shape_radii(shape, shape_size),
            None => self.theme.corner_radii(self.key, shape_size),
        };
        if let Some(bottom_key) = self.bottom_key {
            let bottom = self.theme.corner_radii(bottom_key, shape_size);
            radii.bottom_right = bottom.bottom_right;
            radii.bottom_left = bottom.bottom_left;
        }
        if let Some((shapes, phase)) = &self.morph
            && !shapes.is_empty()
        {
            // Compose の `rememberAnimatedShape` と同じく、角ごとの半径を線形に補間する
            let segment = (phase.floor().max(0.0) as usize).min(shapes.len() - 1);
            let from = match segment {
                0 => radii,
                _ => md3_theme::shape_radii(shapes[segment - 1], shape_size),
            };
            let to = md3_theme::shape_radii(shapes[segment], shape_size);
            let t = phase - segment as f32;
            radii = Corners {
                top_left: from.top_left + (to.top_left - from.top_left) * t,
                top_right: from.top_right + (to.top_right - from.top_right) * t,
                bottom_right: from.bottom_right + (to.bottom_right - from.bottom_right) * t,
                bottom_left: from.bottom_left + (to.bottom_left - from.bottom_left) * t,
            };
        }
        radii
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
        let radii = self.radii(shape_size);
        let outset = self.outset;
        let rounded = self.rounded;
        let radius = |radius: Pixels, rounded: bool| {
            Some((if rounded { radius } else { px(0.0) } + outset).into())
        };
        self.inner.style().corner_radii = CornersRefinement {
            top_left: radius(radii.top_left, rounded.top_left),
            top_right: radius(radii.top_right, rounded.top_right),
            bottom_right: radius(radii.bottom_right, rounded.bottom_right),
            bottom_left: radius(radii.bottom_left, rounded.bottom_left),
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
