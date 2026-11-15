//! ボタンから開くポップアップを置き、開閉に合わせてフォーカスを移す仕組み
//!
//! ポップアップの中身とキーの操作は使う側が作り、ここではボタンの下への配置、外での押下、開閉の前後のフォーカスを扱う

use std::{cell::Cell, rc::Rc};

use gpui::{
    AnyElement, App, AvailableSpace, Bounds, Div, Edges, Element, ElementId, Entity, FocusHandle,
    GlobalElementId, InspectorElementId, InteractiveElement, IntoElement, LayoutId, ParentElement,
    Pixels, Point, ScrollHandle, Size, Stateful, StatefulInteractiveElement, Styled, Window, div,
    point, px, size,
};

use crate::tooltip;

/// ポップアップをツールチップより奥に描くための、遅延描画の優先度
pub(crate) const PRIORITY: usize = tooltip::PRIORITY - 1;

pub(crate) type ExpandedChangeHandler = Rc<dyn Fn(&bool, &mut Window, &mut App)>;

/// 項目のフォーカスと、最後に描いたときの範囲
pub(crate) type FocusTarget = (FocusHandle, Rc<Cell<Bounds<Pixels>>>);

/// 開いたときにフォーカスを移す項目
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum OpenFocus {
    /// 使う側が決める既定の項目
    #[default]
    Default,
    /// 最初のフォーカスできる項目
    First,
    /// 最後のフォーカスできる項目
    Last,
}

/// フレームをまたいで持つポップアップの状態
pub(crate) struct State {
    /// 最後に描いたフレームで開いていたか
    expanded: bool,
    /// 次に開いたときにフォーカスを移す項目
    pub(crate) open_focus: OpenFocus,
    /// ポップアップのフォーカス。フォーカスがポップアップの中にあるかの判定に使う
    pub(crate) popup_focus: FocusHandle,
    pub(crate) scroll_handle: ScrollHandle,
    /// 最後に描いたボタンの範囲
    pub(crate) trigger_bounds: Bounds<Pixels>,
}

/// `id` の下にポップアップの状態を用意して返す
pub(crate) fn use_state(id: ElementId, window: &mut Window, cx: &mut App) -> Entity<State> {
    window.use_keyed_state(id, cx, |_, cx| State {
        expanded: false,
        open_focus: OpenFocus::Default,
        popup_focus: cx.focus_handle(),
        scroll_handle: ScrollHandle::new(),
        trigger_bounds: Bounds::default(),
    })
}

/// `popup` を、開いている間のポップアップの要素にする
///
/// `popup` にポップアップのフォーカスを付け、背後の要素への操作を止める。
/// 余白を含めた範囲でスクロールして切り取る要素で包み、ウィンドウの大きさまでに収める。
/// ポップアップとボタンの外でマウスのボタンを押すと、`on_expanded_change` で閉じるよう求める
pub(crate) fn container(
    popup: Stateful<Div>,
    state: &Entity<State>,
    paint_margin: Edges<Pixels>,
    on_expanded_change: Option<ExpandedChangeHandler>,
    window: &Window,
    cx: &App,
) -> AnyElement {
    let (popup_focus, scroll_handle) = {
        let state = state.read(cx);
        (state.popup_focus.clone(), state.scroll_handle.clone())
    };
    let viewport = window.viewport_size();
    let outside_state = state.clone();
    let close_outside = Rc::new(
        move |position: Point<Pixels>, window: &mut Window, cx: &mut App| {
            // ボタンの上の押下は、ボタンのクリックで閉じるよう求める
            if outside_state.read(cx).trigger_bounds.contains(&position) {
                return;
            }
            if let Some(close) = &on_expanded_change {
                close(&false, window, cx);
            }
        },
    );
    let popup_close_outside = close_outside.clone();
    let popup_scroll_handle = scroll_handle.clone();
    let popup =
        popup
            .track_focus(&popup_focus)
            .occlude()
            .on_mouse_down_out(move |event, window, cx| {
                // スクロールで切り取られた部分もポップアップの範囲に入るため、スクロールの範囲の外の押下はスクロールの要素で扱う
                if popup_scroll_handle.bounds().contains(&event.position) {
                    popup_close_outside(event.position, window, cx);
                }
            });
    // 余白の上にもポップアップの子を描けるよう、余白を含めた範囲でスクロールし、切り取る
    div()
        .id("scroll")
        .track_scroll(&scroll_handle)
        .overflow_y_scroll()
        .max_w(viewport.width)
        .max_h(viewport.height)
        .pt(paint_margin.top)
        .pr(paint_margin.right)
        .pb(paint_margin.bottom)
        .pl(paint_margin.left)
        .on_mouse_down_out(move |event, window, cx| close_outside(event.position, window, cx))
        .child(popup)
        .into_any_element()
}

/// 最後に描いたときの範囲が `item` の項目が、スクロールの範囲から `margin` の余白を除いた範囲に入るよう、
/// `scroll_handle` のポップアップを最小限だけスクロールする
pub(crate) fn scroll_into_view(
    scroll_handle: &ScrollHandle,
    margin: Edges<Pixels>,
    item: Bounds<Pixels>,
) {
    let viewport = scroll_handle.bounds();
    let top = viewport.top() + margin.top;
    let bottom = viewport.bottom() - margin.bottom;
    let mut offset = scroll_handle.offset();
    if item.top() < top {
        offset.y += top - item.top();
    } else if item.bottom() > bottom {
        offset.y -= item.bottom() - bottom;
    }
    scroll_handle.set_offset(offset);
}

/// ボタンの子に置き、ボタンの範囲を記録し、開いたポップアップを配置して、開閉に合わせてフォーカスを移す要素
pub(crate) struct Anchor {
    pub(crate) state: Entity<State>,
    pub(crate) expanded: bool,
    pub(crate) trigger_focus: FocusHandle,
    pub(crate) window_margin: Size<Pixels>,
    pub(crate) paint_margin: Edges<Pixels>,
    /// [`container`] で作った、開いている間のポップアップ
    pub(crate) popup: Option<AnyElement>,
    /// 開いたときにフォーカスを移す項目。`None` ならポップアップにフォーカスを移す
    pub(crate) open_target: Option<FocusTarget>,
}

impl IntoElement for Anchor {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for Anchor {
    /// ボタンと同じ範囲を占める要素
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut area = div().absolute().inset_0().into_any_element();
        let layout_id = area.request_layout(window, cx);
        (layout_id, area)
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        area: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        self.state
            .update(cx, |state, _| state.trigger_bounds = bounds);
        area.prepaint(window, cx);
        let Some(mut popup) = self.popup.take() else {
            return;
        };
        // ボタンの幅に縛られないよう、ウィンドウのルートと同じく独立に大きさを決める
        let popup_size = popup.layout_as_root(
            size(AvailableSpace::MaxContent, AvailableSpace::MaxContent),
            window,
            cx,
        );
        let margin = self.paint_margin;
        let origin = position(
            bounds,
            size(
                popup_size.width - margin.left - margin.right,
                popup_size.height - margin.top - margin.bottom,
            ),
            window.viewport_size(),
            self.window_margin,
        );
        window.defer_draw(
            popup,
            point(origin.x - margin.left, origin.y - margin.top),
            PRIORITY,
            None,
        );
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        area: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        area.paint(window, cx);
        // uniform_list などは測るために描かない要素もレイアウトするため、描いたときだけ呼ばれる paint で開閉を判定する
        let was_expanded = self.state.update(cx, |state, _| {
            state.open_focus = OpenFocus::Default;
            std::mem::replace(&mut state.expanded, self.expanded)
        });
        let (popup_focus, scroll_handle) = {
            let state = self.state.read(cx);
            (state.popup_focus.clone(), state.scroll_handle.clone())
        };
        if self.expanded && !was_expanded {
            let (focus_handle, bounds) = match self.open_target.take() {
                Some((focus_handle, bounds)) => (focus_handle, Some(bounds)),
                None => (popup_focus, None),
            };
            let margin = self.paint_margin;
            // 描画中のフォーカスの移動は再描画されないため、描き終えてから移す。項目の範囲も描き終えると決まる
            window.defer(cx, move |window, cx| {
                window.focus(&focus_handle, cx);
                if let Some(bounds) = bounds {
                    scroll_into_view(&scroll_handle, margin, bounds.get());
                }
            });
        } else if was_expanded && popup_focus.contains_focused(window, cx) {
            let expanded = self.expanded;
            let trigger_focus = self.trigger_focus.clone();
            let focused = window.focused(cx);
            // contains_focused は前のフレームで判定するため、描き終えてから確かめる。
            // それまでに別のポップアップを開くなどでフォーカスが移されたときは移さない
            window.defer(cx, move |window, cx| {
                if window.focused(cx) != focused {
                    return;
                }
                if !expanded {
                    window.focus(&trigger_focus, cx);
                } else if !popup_focus.contains_focused(window, cx) {
                    // フォーカスのある項目が描かれなくなったときは、キーを受け続けられるようポップアップに移す
                    window.focus(&popup_focus, cx);
                }
            });
        }
    }
}

/// `trigger` の範囲のボタンに対し、大きさ `popup` のポップアップを置く左上の位置を返す
///
/// 位置の候補と選び方は、Compose Material3 の `MenuAnchorPosition.Below` と `DropdownMenuPositionProvider`
/// (androidx の compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Menu.kt と
/// internal/MenuPosition.kt) を左から右へ書く向きについて移したもの
pub(crate) fn position(
    trigger: Bounds<Pixels>,
    popup: Size<Pixels>,
    viewport: Size<Pixels>,
    margin: Size<Pixels>,
) -> Point<Pixels> {
    // 余白の内側に収まる最初の候補を選ぶ。どれも収まらなければ、余白を除いた空きより大きいときは中央に置き、
    // そうでなければ最後の候補を余白の内側に寄せる
    let place = |candidates: &[Pixels], size: Pixels, space: Pixels, margin: Pixels| {
        candidates
            .iter()
            .copied()
            .find(|&candidate| candidate >= margin && candidate + size <= space - margin)
            .unwrap_or_else(|| {
                if size >= space - margin * 2. {
                    (space - size) / 2.
                } else {
                    let last = candidates[candidates.len() - 1];
                    last.max(margin).min(space - margin - size)
                }
            })
    };
    let center = trigger.center();
    let x = place(
        &[
            trigger.left(),
            trigger.right() - popup.width,
            if center.x < viewport.width / 2. {
                px(0.)
            } else {
                viewport.width - popup.width
            },
        ],
        popup.width,
        viewport.width,
        margin.width,
    );
    let y = place(
        &[
            trigger.bottom(),
            trigger.top() - popup.height,
            trigger.top() - popup.height / 2.,
            if center.y < viewport.height / 2. {
                px(0.)
            } else {
                viewport.height - popup.height
            },
        ],
        popup.height,
        viewport.height,
        margin.height,
    );
    point(x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds(x: f32, y: f32, w: f32, h: f32) -> Bounds<Pixels> {
        Bounds::new(point(px(x), px(y)), size(px(w), px(h)))
    }

    /// Compose の `MenuAnchorPosition.Below` の候補を、500x400 のウィンドウ、左右 8 と上下 48 の余白、
    /// 100x84 のポップアップについて評価した位置
    #[test]
    fn 位置は余白の内側に収まる最初の候補を選ぶ() {
        let viewport = size(px(500.), px(400.));
        let margin = size(px(8.), px(48.));
        let popup = size(px(100.), px(84.));
        let cases = [
            // ボタンの下に左端をそろえる
            (bounds(100., 100., 40., 20.), (100., 120.)),
            // 右に収まらなければ右端をそろえる。下に収まらなければ上に置く
            (bounds(420., 300., 40., 20.), (360., 216.)),
            // 左右どちらにそろえても収まらなければ、ボタンの中心に近い左の余白に寄せる
            (bounds(0., 100., 95., 20.), (8., 120.)),
            // 上下どちらにも収まらなければ、中心をボタンの上端にそろえる
            (bounds(100., 100., 40., 220.), (100., 58.)),
            // それでも収まらなければ、ボタンの中心に近い下の余白に寄せる
            (bounds(100., 60., 40., 300.), (100., 268.)),
        ];
        for (trigger, (x, y)) in cases {
            assert_eq!(
                position(trigger, popup, viewport, margin),
                point(px(x), px(y)),
                "{trigger:?}"
            );
        }
    }

    #[test]
    fn 余白を除いた空きより大きいポップアップは中央に置く() {
        let viewport = size(px(500.), px(400.));
        let margin = size(px(8.), px(48.));
        assert_eq!(
            position(
                bounds(0., 0., 40., 20.),
                size(px(490.), px(310.)),
                viewport,
                margin
            ),
            point(px(5.), px(45.))
        );
    }
}
