//! 入れ子のスクロール領域から祖先へのホイール伝播を制御する。

use std::{cell::Cell, rc::Rc};

use gpui::{
    App, DispatchPhase, ElementId, IntoElement, Pixels, Point, ScrollEvent, ScrollHandle,
    TouchPhase, Window, canvas, prelude::*, px,
};

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
        })
}

#[cfg(test)]
mod tests {
    use super::*;

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
