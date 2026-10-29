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

/// フォーカスが移ってきた要素を、スクロール領域の見える範囲に入れる（WCAG 2.2 の 2.4.11）。
///
/// gpui の `Window::focus` も Tab の移動もスクロールしないため、スクロール領域の中で
/// フォーカスを動かす部品は、各項目の子にこれを置く。フォーカスが移ってきたときに一度だけ、
/// 項目が収まる最小の量だけ動かす（ホイールで外へ動かしても、フォーカスが同じなら引き戻さない）。
/// `last` は領域ごとに 1 つ（[`reveal_state`]）。
#[cfg(any(feature = "containment", feature = "overlays", feature = "navigation"))]
pub(crate) fn reveal_on_focus(
    scroll: ScrollHandle,
    focus: gpui::FocusHandle,
    last: RevealState,
) -> impl IntoElement {
    canvas(
        move |bounds, window, _| {
            let focused = focus.is_focused(window);
            let is_last = last.borrow().as_ref() == Some(&focus);
            if !focused {
                // 領域の外へ出たら忘れる（戻ってきたときにまた見せる）。
                if is_last {
                    *last.borrow_mut() = None;
                }
                return;
            }
            if is_last {
                return;
            }
            *last.borrow_mut() = Some(focus.clone());
            if let Some(offset) = reveal_offset(scroll.bounds(), bounds, scroll.offset()) {
                scroll.set_offset(offset);
                window.refresh();
            }
        },
        |_, _, _, _| {},
    )
    .absolute()
    .inset_0()
}

/// 最後に見せた項目のフォーカス。領域ごとに描画をまたいで保つ。
#[cfg(any(feature = "containment", feature = "overlays", feature = "navigation"))]
pub(crate) type RevealState = Rc<std::cell::RefCell<Option<gpui::FocusHandle>>>;

/// [`reveal_on_focus`] の状態を取り出す。
#[cfg(any(feature = "containment", feature = "overlays", feature = "navigation"))]
pub(crate) fn reveal_state(id: &ElementId, window: &mut Window, cx: &mut App) -> RevealState {
    window
        .use_keyed_state(
            ElementId::NamedChild(id.clone().into(), "reveal-on-focus".into()),
            cx,
            |_, _| Rc::new(std::cell::RefCell::new(None)),
        )
        .read(cx)
        .clone()
}

/// `item` を `viewport` に収める新しいオフセット。動かす必要がなければ `None`。
/// 項目が領域より高ければ上端を合わせる。
#[cfg(any(feature = "containment", feature = "overlays", feature = "navigation"))]
fn reveal_offset(
    viewport: gpui::Bounds<Pixels>,
    item: gpui::Bounds<Pixels>,
    offset: Point<Pixels>,
) -> Option<Point<Pixels>> {
    let delta = if item.top() < viewport.top() || item.size.height > viewport.size.height {
        viewport.top() - item.top()
    } else if item.bottom() > viewport.bottom() {
        viewport.bottom() - item.bottom()
    } else {
        return None;
    };
    (delta != px(0.0)).then(|| gpui::point(offset.x, offset.y + delta))
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
    #[cfg(any(feature = "containment", feature = "overlays", feature = "navigation"))]
    fn reveal_scrolls_the_minimum_amount() {
        use gpui::{Bounds, point};

        let viewport = Bounds::new(point(px(0.0), px(100.0)), gpui::size(px(200.0), px(300.0)));
        let offset = point(px(0.0), px(-50.0));
        let item = |top: f32, height: f32| {
            Bounds::new(point(px(0.0), px(top)), gpui::size(px(200.0), px(height)))
        };
        // 見えていれば動かさない。
        assert_eq!(reveal_offset(viewport, item(150.0, 48.0), offset), None);
        // 下にはみ出したら、下端を合わせる分だけ進める。
        assert_eq!(
            reveal_offset(viewport, item(380.0, 48.0), offset),
            Some(point(px(0.0), px(-78.0)))
        );
        // 上にはみ出したら、上端を合わせる分だけ戻す。
        assert_eq!(
            reveal_offset(viewport, item(80.0, 48.0), offset),
            Some(point(px(0.0), px(-30.0)))
        );
        // 領域より高い項目は上端を合わせる。
        assert_eq!(
            reveal_offset(viewport, item(120.0, 400.0), offset),
            Some(point(px(0.0), px(-70.0)))
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
