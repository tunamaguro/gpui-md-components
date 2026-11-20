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

/// 前の描画でフォーカスのあった子と、表示領域の高さと、その子の表示領域からの上端と下端
#[cfg(any(feature = "navigation-rail", feature = "navigation-drawer"))]
pub(crate) type FocusedPlacement = (
    Option<usize>,
    gpui::Pixels,
    Option<(gpui::Pixels, gpui::Pixels)>,
);

/// `scroll` が追う並びで、`focused_child` 番目の子を上下に `margin` の余白を含めて表示範囲に入れる
///
/// この描画のレイアウトで決まった範囲を使うため、並びより後に描く要素の prepaint から呼ぶ。
/// フォーカスのある子・表示領域の高さ・その子の配置のどれかが前の呼び出し (`last`) から変わったときだけスクロールし、
/// スクロールしたら変えたオフセットで描き直すよう次のフレームを求める。`focused_child` が `None` ならスクロールしない
#[cfg(any(feature = "navigation-rail", feature = "navigation-drawer"))]
pub(crate) fn scroll_focused_into_view(
    scroll: &gpui::ScrollHandle,
    last: &gpui::Entity<FocusedPlacement>,
    focused_child: Option<usize>,
    margin: gpui::Pixels,
    window: &mut Window,
    cx: &mut App,
) {
    let viewport = scroll.bounds();
    let item = focused_child.and_then(|child| scroll.bounds_for_item(child));
    // 子の範囲はスクロールする前の位置なので、表示領域からの位置はスクロールで変わらず、配置が変わると変わる
    let placement = item.map(|item| (item.top() - viewport.top(), item.bottom() - viewport.top()));
    let key = (focused_child, viewport.size.height, placement);
    if key == *last.read(cx) {
        return;
    }
    last.update(cx, |last, _| *last = key);
    let Some(item) = item else {
        return;
    };
    // 今のオフセットを足して画面上の位置にする
    let mut offset = scroll.offset();
    let top = item.top() + offset.y - margin;
    let bottom = item.bottom() + offset.y + margin;
    if top < viewport.top() {
        offset.y += viewport.top() - top;
    } else if bottom > viewport.bottom() {
        offset.y -= bottom - viewport.bottom();
    }
    if offset != scroll.offset() {
        scroll.set_offset(offset);
        window.request_animation_frame();
    }
}
