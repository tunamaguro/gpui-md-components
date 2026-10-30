//! 入れ子のスクロール領域のホイール伝播と、キーボードフォーカスの表示を制御する。

use std::{cell::Cell, rc::Rc};

use gpui::{
    AnyElement, App, Bounds, DispatchPhase, Element, ElementId, FocusHandle, Global,
    GlobalElementId, InspectorElementId, IntoElement, LayoutId, Pixels, Point, ScrollEvent,
    ScrollHandle, TouchPhase, Window, canvas, prelude::*, px,
};
use md3_theme::md3_tokens::state;

#[derive(Default)]
struct ScopeStack(Vec<ScrollHandle>);

impl Global for ScopeStack {}

/// キーボードでフォーカスした子要素を見せるためのスクロール領域。
///
/// `track_scroll` した div を同じハンドルで包む。レイアウト用の箱は増やさない。
/// 入れ子なら内側から外側へ順に表示位置を調整する。遅延描画するオーバーレイは
/// 背後の領域を引き継がず、オーバーレイ内に作った領域だけを使う。
///
/// ```ignore
/// ScrollScope::new(handle.clone(), div().id("page")
///     .overflow_y_scroll().track_scroll(&handle).child(content))
/// ```
pub struct ScrollScope {
    handle: ScrollHandle,
    child: AnyElement,
}

impl ScrollScope {
    /// スクロール領域のハンドルと、その領域の要素から作る。
    pub fn new(handle: ScrollHandle, child: impl IntoElement) -> Self {
        Self {
            handle,
            child: child.into_any_element(),
        }
    }
}

impl IntoElement for ScrollScope {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

impl Element for ScrollScope {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        (self.child.request_layout(window, cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        // 同期的な prepaint の間だけ保持するため、別ウィンドウや遅延描画には漏れない。
        cx.default_global::<ScopeStack>()
            .0
            .push(self.handle.clone());
        self.child.prepaint(window, cx);
        cx.global_mut::<ScopeStack>().0.pop();
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.paint(window, cx);
    }
}

/// 描画をまたいで保持するスクロール位置。
#[cfg(any(feature = "containment", feature = "overlays", feature = "navigation"))]
pub(crate) fn scroll_handle(id: &ElementId, window: &mut Window, cx: &mut App) -> ScrollHandle {
    window
        .use_keyed_state(
            ElementId::NamedChild(id.clone().into(), "nested-scroll-handle".into()),
            cx,
            |_, _| ScrollHandle::new(),
        )
        .read(cx)
        .clone()
}

/// 入れ子のスクロール領域のホイール処理。
///
/// gpui の div はホイールをポインタの下のスクロール領域すべてに適用するため、
/// 入れ子にすると外側も一緒に動く。これを領域に付けると、内側が動いた操作は
/// 祖先に渡さず、端に達した操作だけ祖先に渡す。精密スクロールは開始から終了まで
/// 内側が持つ。縦・横どちらの領域にも使える。
///
/// ```ignore
/// let wheel = NestedScroll::new(&id, handle.clone(), window, cx);
/// div()
///     .id(id)
///     .overflow_y_scroll()
///     .track_scroll(&handle)
///     .on_scroll(wheel.on_scroll())
///     .child(content)
///     .child(wheel.capture())
/// ```
pub struct NestedScroll {
    handle: ScrollHandle,
    before: Rc<Cell<Point<Pixels>>>,
    gesture: Rc<Cell<(bool, bool)>>,
}

impl NestedScroll {
    /// 領域の安定 ID と、領域に `track_scroll` したハンドルから作る。
    pub fn new(id: &ElementId, handle: ScrollHandle, window: &mut Window, cx: &mut App) -> Self {
        let gesture = window
            .use_keyed_state(
                ElementId::NamedChild(id.clone().into(), "nested-scroll-gesture".into()),
                cx,
                |_, _| Rc::new(Cell::new((false, false))),
            )
            .read(cx)
            .clone();
        Self {
            before: Rc::new(Cell::new(handle.offset())),
            handle,
            gesture,
        }
    }

    /// 領域の子に置く。レイアウト・描画・ヒットテストに影響しない、capture リスナーの登録要素。
    pub fn capture(&self) -> impl IntoElement + use<> {
        let handle = self.handle.clone();
        let before = self.before.clone();
        canvas(
            |_, _, _| (),
            move |_, _, window, _| {
                window.on_mouse_event(move |_: &ScrollEvent, phase, _, _| {
                    if phase == DispatchPhase::Capture {
                        before.set(handle.offset());
                    }
                });
            },
        )
        .absolute()
        .size(px(0.0))
    }

    /// 領域の `on_scroll` に渡すリスナー。
    ///
    /// gpui@37faef8 は利用者、既定処理、子の順にリスナーを登録する。
    /// bubble は逆順なので、ここでは内側の既定処理が完了し、祖先はまだ動いていない。
    /// UniformList も同じ Interactivity::paint を使う。capture で保存した位置との差を
    /// 使うため、行高・ピクセル差分の変換や軸フィルターを複製する必要がない。
    pub fn on_scroll(&self) -> impl Fn(&ScrollEvent, &mut Window, &mut App) + 'static + use<> {
        let handle = self.handle.clone();
        let before = self.before.clone();
        let gesture = self.gesture.clone();
        move |event, _, cx| {
            let mut offset = handle.offset();
            let max = handle.max_offset();
            let start = before.get();
            // 縦だけの領域でも横のホイールは縦に振り替えられるため、両軸の差を見る。
            let moved = can_scroll(start.y, max.y, offset.y - start.y)
                || can_scroll(start.x, max.x, offset.x - start.x);
            offset.y = offset.y.clamp(-max.y.max(px(0.0)), px(0.0));
            offset.x = offset.x.clamp(-max.x.max(px(0.0)), px(0.0));
            handle.set_offset(offset);
            let (active, claimed) = gesture.get();
            let (consume, next) = gesture_step(
                active,
                claimed,
                event.delta.precise(),
                event.touch_phase,
                moved,
            );
            gesture.set(next);
            if consume {
                cx.stop_propagation();
            }
        }
    }
}

fn can_scroll(offset: Pixels, max: Pixels, delta: Pixels) -> bool {
    max > px(0.0) && ((delta < px(0.0) && offset > -max) || (delta > px(0.0) && offset < px(0.0)))
}

fn gesture_step(
    active: bool,
    claimed: bool,
    precise: bool,
    phase: TouchPhase,
    moved: bool,
) -> (bool, (bool, bool)) {
    let active = precise && (phase == TouchPhase::Started || active);
    let claimed = active && phase != TouchPhase::Started && claimed;
    let consume = moved || claimed;
    let ended = matches!(phase, TouchPhase::Ended | TouchPhase::Cancelled);
    (consume, (active && !ended, active && consume && !ended))
}

/// フォーカスが移ってきた要素を、スクロール領域の見える範囲に入れる（WCAG 2.2 の 2.4.11）。
///
/// `.relative()` な操作要素の子に置く。祖先の [`ScrollScope`] を内側から順に調整し、
/// 外側のフォーカスリングも収まる最小量だけ動かす。領域より大きい要素は先頭を合わせる。
/// キーボード入力によってフォーカスが移ったときだけ動くため、マウスクリックでは動かさず、
/// 同じ要素にフォーカスしたままホイールで外へ動かしても引き戻さない。
/// 描画・ヒットテスト・レイアウト上の大きさには影響しない。
pub fn reveal_on_focus(
    id: &ElementId,
    focus: FocusHandle,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement + use<> {
    let was_focused = window
        .use_keyed_state(
            ElementId::NamedChild(id.clone().into(), "reveal-on-focus".into()),
            cx,
            |_, _| Rc::new(Cell::new(false)),
        )
        .read(cx)
        .clone();
    canvas(
        move |mut bounds, window, cx| {
            let focused = focus.is_focused(window);
            let previous = was_focused.replace(focused);
            if !focused || previous || !window.last_input_was_keyboard() {
                return;
            }
            let mut changed = false;
            for scroll in cx.default_global::<ScopeStack>().0.iter().rev() {
                let before = scroll.offset();
                if let Some(offset) =
                    reveal_offset(scroll.bounds(), bounds, before, scroll.max_offset())
                {
                    scroll.set_offset(offset);
                    // 内側で実際に動いた量を外側の判定へ反映する（端での制限も含む）。
                    bounds.origin += scroll.offset() - before;
                    changed = true;
                }
            }
            if changed {
                // 描画中の refresh は無効。変更した位置は次のフレームで描く。
                window.on_next_frame(|window, _| window.refresh());
            }
        },
        |_, _, _, _| {},
    )
    .absolute()
    .inset_0()
}

/// `item` を `viewport` に収める新しいオフセット。動かす必要がなければ `None`。
/// 外側のリング分だけ広げ、各軸をスクロール可能な範囲に制限する。
fn reveal_offset(
    viewport: Bounds<Pixels>,
    item: Bounds<Pixels>,
    offset: Point<Pixels>,
    max_offset: Point<Pixels>,
) -> Option<Point<Pixels>> {
    let item = item.dilate(px(
        state::FOCUS_INDICATOR_THICKNESS + state::FOCUS_INDICATOR_OUTER_OFFSET
    ));
    let axis = |start: Pixels,
                end: Pixels,
                item_start: Pixels,
                item_end: Pixels,
                offset: Pixels,
                max: Pixels| {
        let delta = if item_start < start || item_end - item_start > end - start {
            start - item_start
        } else if item_end > end {
            end - item_end
        } else {
            px(0.0)
        };
        (offset + delta).clamp(-max.max(px(0.0)), px(0.0))
    };
    let next = gpui::point(
        axis(
            viewport.left(),
            viewport.right(),
            item.left(),
            item.right(),
            offset.x,
            max_offset.x,
        ),
        axis(
            viewport.top(),
            viewport.bottom(),
            item.top(),
            item.bottom(),
            offset.y,
            max_offset.y,
        ),
    );
    (next != offset).then_some(next)
}

/// オーバーレイの本文。ヘッダーと操作ボタンを残したまま縮小できる領域。
#[cfg(feature = "overlays")]
pub(crate) fn content(
    parent: &ElementId,
    children: Vec<gpui::AnyElement>,
    scrollbar: bool,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement + use<> {
    let id = ElementId::NamedChild(parent.clone().into(), "scroll-content".into());
    let scroll = scroll_handle(&id, window, cx);
    let wheel = NestedScroll::new(&id, scroll.clone(), window, cx);
    ScrollScope::new(
        scroll.clone(),
        gpui::div()
            .id(id.clone())
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&scroll)
            .on_scroll(wheel.on_scroll())
            .child(gpui::div().flex().flex_col().children(children))
            .child(wheel.capture())
            .when(scrollbar, |this| {
                this.child(crate::Scrollbar::new(
                    ElementId::NamedChild(id.into(), "scrollbar".into()),
                    scroll,
                ))
            }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(left: f32, top: f32, width: f32, height: f32) -> Bounds<Pixels> {
        Bounds::new(
            gpui::point(px(left), px(top)),
            gpui::size(px(width), px(height)),
        )
    }

    #[test]
    fn reveal_scrolls_both_axes_by_the_minimum_amount() {
        use gpui::point;
        let viewport = rect(100.0, 100.0, 200.0, 300.0);
        let offset = point(px(-50.0), px(-50.0));
        let max = point(px(500.0), px(500.0));
        assert_eq!(
            reveal_offset(viewport, rect(150.0, 150.0, 48.0, 48.0), offset, max),
            None
        );
        assert_eq!(
            reveal_offset(viewport, rect(280.0, 380.0, 48.0, 48.0), offset, max),
            Some(point(px(-83.0), px(-83.0)))
        );
        assert_eq!(
            reveal_offset(viewport, rect(80.0, 80.0, 48.0, 48.0), offset, max),
            Some(point(px(-25.0), px(-25.0)))
        );
        // 領域より大きい項目は、リングを含めた先頭を合わせる。
        assert_eq!(
            reveal_offset(viewport, rect(120.0, 120.0, 400.0, 400.0), offset, max),
            Some(point(px(-65.0), px(-65.0)))
        );
    }

    #[test]
    fn reveal_includes_the_outer_focus_ring() {
        use gpui::point;
        let viewport = rect(100.0, 100.0, 200.0, 300.0);
        let offset = point(px(-50.0), px(-50.0));
        let max = point(px(500.0), px(500.0));
        // 本体が収まっていても、外側の 3dp + 2dp の分を空ける。
        assert_eq!(
            reveal_offset(viewport, rect(100.0, 100.0, 48.0, 48.0), offset, max),
            Some(point(px(-45.0), px(-45.0)))
        );
        assert_eq!(
            reveal_offset(viewport, rect(252.0, 352.0, 48.0, 48.0), offset, max),
            Some(point(px(-55.0), px(-55.0)))
        );
        assert_eq!(
            reveal_offset(viewport, rect(105.0, 105.0, 48.0, 48.0), offset, max),
            None
        );
    }

    #[test]
    fn reveal_clamps_both_axes_to_the_scrollable_range() {
        use gpui::point;
        let viewport = rect(0.0, 0.0, 100.0, 100.0);
        let offset = point(px(-20.0), px(-30.0));
        let max = point(px(50.0), px(70.0));
        assert_eq!(
            reveal_offset(viewport, rect(-500.0, -500.0, 48.0, 48.0), offset, max),
            Some(point(px(0.0), px(0.0)))
        );
        assert_eq!(
            reveal_offset(viewport, rect(500.0, 500.0, 48.0, 48.0), offset, max),
            Some(-max)
        );
        assert_eq!(
            reveal_offset(viewport, rect(500.0, 500.0, 48.0, 48.0), -max, max),
            None
        );
        assert_eq!(
            reveal_offset(
                viewport,
                rect(500.0, 500.0, 48.0, 48.0),
                point(px(0.0), px(0.0)),
                point(px(0.0), px(0.0))
            ),
            None
        );
    }

    #[test]
    fn edges_pass_through_but_crossing_an_edge_is_consumed() {
        assert!(!can_scroll(px(0.0), px(100.0), px(10.0)));
        assert!(!can_scroll(px(-100.0), px(100.0), px(-10.0)));
        assert!(can_scroll(px(-95.0), px(100.0), px(-20.0)));
        assert!(can_scroll(px(-5.0), px(100.0), px(20.0)));
        assert!(can_scroll(px(0.0), px(100.0), px(-0.5)));
        assert!(can_scroll(px(-100.0), px(100.0), px(0.5)));
        assert!(!can_scroll(px(0.0), px(0.0), px(-10.0)));
        assert!(!can_scroll(px(-50.0), px(100.0), px(0.0)));
    }

    #[test]
    fn precise_gesture_stays_claimed_until_end() {
        let (_, (active, claimed)) = gesture_step(false, false, true, TouchPhase::Started, true);
        assert_eq!(
            gesture_step(active, claimed, true, TouchPhase::Moved, false),
            (true, (true, true))
        );
        assert_eq!(
            gesture_step(active, claimed, true, TouchPhase::Ended, false),
            (true, (false, false))
        );
        assert_eq!(
            gesture_step(active, claimed, true, TouchPhase::Cancelled, false),
            (true, (false, false))
        );
        assert_eq!(
            gesture_step(active, claimed, true, TouchPhase::Started, false),
            (false, (true, false))
        );
        assert_eq!(
            gesture_step(false, false, false, TouchPhase::Moved, true),
            (true, (false, false))
        );
        assert_eq!(
            gesture_step(false, false, true, TouchPhase::Moved, false),
            (false, (false, false))
        );
    }
}
