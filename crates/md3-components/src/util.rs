//! 部品が共通で使う処理

use gpui::{App, ElementId, FocusHandle, Window};
#[cfg(any(feature = "menu", feature = "select"))]
use gpui::{BoxShadow, Edges, Pixels};

/// `id` に結び付いた、部品が内部で持つフォーカスを返す
///
/// 同じ `id` で描画し続ける間は同じフォーカスを返す。`id` が変わるか 1 フレームでも描画されないと、新しいフォーカスになる
pub(crate) fn keyed_focus_handle(id: &ElementId, window: &mut Window, cx: &mut App) -> FocusHandle {
    window
        .use_keyed_state(id.clone(), cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone()
}

/// `shadow` の影を、切り取られずに描くために要素の周りに取る余白
///
/// gpui は影を、形の縁から、ぼかしの半径の 3 倍と広がりを足した幅だけ外側まで描く
#[cfg(any(feature = "menu", feature = "select"))]
pub(crate) fn shadow_margin(shadow: &[BoxShadow]) -> Edges<Pixels> {
    shadow
        .iter()
        .fold(Edges::<Pixels>::default(), |margin, shadow| {
            let extent = shadow.blur_radius * 3.0 + shadow.spread_radius;
            Edges {
                top: margin.top.max(extent - shadow.offset.y),
                right: margin.right.max(extent + shadow.offset.x),
                bottom: margin.bottom.max(extent + shadow.offset.y),
                left: margin.left.max(extent - shadow.offset.x),
            }
        })
}
