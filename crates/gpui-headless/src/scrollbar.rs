// Ported from AndroidX Compose (https://github.com/androidx/androidx)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Scrollbar.kt
// @ b8256a524b9fe4bb6cd0fc7c72aad9dd08877ca6
// Copyright 2026 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
//! スクロールバーの位置計算とポインタ操作。
use gpui::{
    App, Axis, Bounds, ElementId, HitboxBehavior, IntoElement, ListState, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, ScrollHandle,
    UniformListScrollHandle, Window, canvas, point, prelude::*, px, size,
};
use std::rc::Rc;

/// スクロール位置の供給元。位置は先頭を 0 とする正の距離。
///
/// setter の呼び出し後の再描画は Scrollbar が要求する。
pub trait ScrollbarSource: 'static {
    /// ウィンドウ座標系での表示領域。
    fn viewport(&self) -> Bounds<Pixels>;
    /// 軸方向の最大スクロール距離。
    fn max_offset(&self, axis: Axis) -> Pixels;
    /// 軸方向の現在位置。
    fn offset(&self, axis: Axis) -> Pixels;
    /// 軸方向の位置を更新する。他方の軸は保持する。
    fn set_offset(&self, axis: Axis, offset: Pixels);
    /// つまみのドラッグが始まった。
    fn drag_started(&self) {}
    /// つまみのドラッグが終わった。
    fn drag_ended(&self) {}
}

/// つまみのドラッグ。バーが描かれなくなっても `drag_ended` が呼ばれるよう、終わりを Drop に任せる。
struct Drag {
    source: Rc<dyn ScrollbarSource>,
    start: Pixels,
    offset: Pixels,
}

impl Drop for Drag {
    fn drop(&mut self) {
        self.source.drag_ended();
    }
}

fn coordinate(point: Point<Pixels>, axis: Axis) -> Pixels {
    match axis {
        Axis::Vertical => point.y,
        Axis::Horizontal => point.x,
    }
}
fn length(bounds: Bounds<Pixels>, axis: Axis) -> Pixels {
    match axis {
        Axis::Vertical => bounds.size.height,
        Axis::Horizontal => bounds.size.width,
    }
}
impl ScrollbarSource for ScrollHandle {
    fn viewport(&self) -> Bounds<Pixels> {
        self.bounds()
    }
    fn max_offset(&self, axis: Axis) -> Pixels {
        coordinate(self.max_offset(), axis)
    }
    fn offset(&self, axis: Axis) -> Pixels {
        -coordinate(self.offset(), axis)
    }
    fn set_offset(&self, axis: Axis, offset: Pixels) {
        let mut position = self.offset();
        let offset = -offset.clamp(
            px(0.0),
            ScrollbarSource::max_offset(self, axis).max(px(0.0)),
        );
        match axis {
            Axis::Vertical => position.y = offset,
            Axis::Horizontal => position.x = offset,
        }
        self.set_offset(position);
    }
}
impl ScrollbarSource for ListState {
    fn viewport(&self) -> Bounds<Pixels> {
        self.viewport_bounds()
    }
    fn max_offset(&self, axis: Axis) -> Pixels {
        coordinate(self.max_offset_for_scrollbar(), axis)
    }
    fn offset(&self, axis: Axis) -> Pixels {
        -coordinate(self.scroll_px_offset_for_scrollbar(), axis)
    }
    fn set_offset(&self, axis: Axis, offset: Pixels) {
        // list は縦にしか動かない。
        if axis == Axis::Vertical {
            self.set_offset_from_scrollbar(Point::new(px(0.0), -offset));
        }
    }
    // ドラッグ中に行の高さが変わっても位置が飛ばないよう、list に範囲を固定させる。
    fn drag_started(&self) {
        self.scrollbar_drag_started();
    }
    fn drag_ended(&self) {
        self.scrollbar_drag_ended();
    }
}
impl ScrollbarSource for UniformListScrollHandle {
    fn viewport(&self) -> Bounds<Pixels> {
        self.0.borrow().base_handle.bounds()
    }
    fn max_offset(&self, axis: Axis) -> Pixels {
        ScrollbarSource::max_offset(&self.0.borrow().base_handle, axis)
    }
    fn offset(&self, axis: Axis) -> Pixels {
        ScrollbarSource::offset(&self.0.borrow().base_handle, axis)
    }
    fn set_offset(&self, axis: Axis, offset: Pixels) {
        let mut state = self.0.borrow_mut();
        state.deferred_scroll_to_item = None;
        ScrollbarSource::set_offset(&state.base_handle, axis, offset);
    }
}

/// ポインタ操作の種類。キーボード操作はスクロール領域に任せる。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollbarInteraction {
    /// 位置を表示するだけ。
    Passive,
    /// つまみのドラッグとトラックのページ移動（Web でも有効）。
    #[default]
    Draggable,
}

#[derive(Clone, Copy, Debug)]
struct Geometry {
    start: f32,
    thumb: f32,
    travel: f32,
    max: f32,
}
impl Geometry {
    fn new(viewport: f32, max: f32, offset: f32, options: ScrollbarGeometry) -> Option<Self> {
        let track = viewport - 2.0 * f32::from(options.track_inset);
        if !viewport.is_finite()
            || !max.is_finite()
            || max <= 0.0
            || track < f32::from(options.min_thumb_length)
        {
            return None;
        }
        // 短い領域では最小長と最大比率が矛盾するため、最大比率を優先して panic を避ける。
        let upper = track * options.max_thumb_fraction;
        let thumb = (track * viewport / (viewport + max))
            .clamp(f32::from(options.min_thumb_length).min(upper), upper);
        let travel = track - thumb;
        let offset = if offset.is_finite() {
            offset.clamp(0.0, max)
        } else {
            0.0
        };
        Some(Self {
            start: f32::from(options.track_inset) + travel * offset / max,
            thumb,
            travel,
            max,
        })
    }
    fn drag(self, offset: f32, delta: f32) -> f32 {
        if self.travel <= 0.0 {
            return 0.0;
        }
        (offset + delta * self.max / self.travel).clamp(0.0, self.max)
    }
}

/// スクロールバーの幾何計算に使う寸法。
#[derive(Clone, Copy)]
pub struct ScrollbarGeometry {
    /// トラックの長手方向の端からの間隔。
    pub track_inset: Pixels,
    /// トラックの横方向の端からの間隔。
    pub cross_inset: Pixels,
    /// つまみの太さ。
    pub thickness: Pixels,
    /// つまみの最小長。
    pub min_thumb_length: Pixels,
    /// トラックの長さに対するつまみの最大比率。
    pub max_thumb_fraction: f32,
    /// ポインタの操作領域の幅。
    pub hit_width: Pixels,
}

/// スクロールバーの描画に使う状態。
#[derive(Clone, Copy)]
pub struct ScrollbarState {
    /// つまみのウィンドウ座標。
    pub thumb: Bounds<Pixels>,
    /// 現在のスクロール位置。
    pub offset: Pixels,
    /// ポインタが操作領域にあるか。
    pub hovered: bool,
    /// ドラッグ中か。
    pub dragging: bool,
}

/// 見た目を持たないスクロールバー。
pub struct Scrollbar {
    id: ElementId,
    source: Rc<dyn ScrollbarSource>,
    axis: Axis,
    interaction: ScrollbarInteraction,
    geometry: ScrollbarGeometry,
}
impl Scrollbar {
    /// ID、スクロール位置の供給元、寸法から作る。
    pub fn new(
        id: impl Into<ElementId>,
        source: impl ScrollbarSource,
        geometry: ScrollbarGeometry,
    ) -> Self {
        Self {
            id: id.into(),
            source: Rc::new(source),
            axis: Axis::Vertical,
            interaction: ScrollbarInteraction::default(),
            geometry,
        }
    }
    /// スクロール軸を選ぶ。
    pub fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }
    /// ポインタ操作を選ぶ。
    pub fn interaction(mut self, interaction: ScrollbarInteraction) -> Self {
        self.interaction = interaction;
        self
    }
    /// トラックを作り、各フレームのつまみの描画を呼び出す。
    pub fn build(
        self,
        window: &mut Window,
        cx: &mut App,
        paint: impl Fn(ScrollbarState, &mut Window, &mut App) + 'static,
    ) -> impl IntoElement {
        let state = window.use_keyed_state(self.id, cx, |_, _| State {
            hover: false,
            drag: None,
        });
        let source = self.source;
        let pre_source = source.clone();
        let axis = self.axis;
        let options = self.geometry;
        let draggable = self.interaction == ScrollbarInteraction::Draggable;
        canvas(
            move |_, window, _| {
                let viewport = pre_source.viewport();
                let geometry = Geometry::new(
                    length(viewport, axis).into(),
                    pre_source.max_offset(axis).into(),
                    pre_source.offset(axis).into(),
                    options,
                );
                let mut hit = viewport;
                match axis {
                    Axis::Vertical => {
                        hit.origin.x += (hit.size.width - options.hit_width).max(px(0.0));
                        hit.size.width = hit.size.width.min(options.hit_width);
                    }
                    Axis::Horizontal => {
                        hit.origin.y += (hit.size.height - options.hit_width).max(px(0.0));
                        hit.size.height = hit.size.height.min(options.hit_width);
                    }
                }
                let hitbox = geometry.map(|_| {
                    window.insert_hitbox(
                        hit,
                        if draggable {
                            HitboxBehavior::BlockMouseExceptScroll
                        } else {
                            HitboxBehavior::Normal
                        },
                    )
                });
                (viewport, geometry, hitbox)
            },
            move |_, (viewport, geometry, hitbox), window, cx| {
                let Some(geometry) = geometry else {
                    state.update(cx, |state, _| {
                        state.drag = None;
                        state.hover = false;
                    });
                    return;
                };
                let Some(hitbox) = hitbox else {
                    return;
                };
                let offset = source.offset(axis);
                let hovered = hitbox.is_hovered(window);
                let dragging = state.update(cx, |state, _| {
                    if !draggable {
                        state.drag = None;
                    }
                    state.hover = hovered;
                    state.drag.is_some()
                });
                let thumb = match axis {
                    Axis::Vertical => Bounds::new(
                        point(
                            viewport.right() - options.cross_inset - options.thickness,
                            viewport.top() + px(geometry.start),
                        ),
                        size(options.thickness, px(geometry.thumb)),
                    ),
                    Axis::Horizontal => Bounds::new(
                        point(
                            viewport.left() + px(geometry.start),
                            viewport.bottom() - options.cross_inset - options.thickness,
                        ),
                        size(px(geometry.thumb), options.thickness),
                    ),
                };
                paint(
                    ScrollbarState {
                        thumb,
                        offset,
                        hovered,
                        dragging,
                    },
                    window,
                    cx,
                );
                let move_state = state.clone();
                let move_source = source.clone();
                let move_hitbox = hitbox.clone();
                window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                    if !phase.capture() {
                        return;
                    }
                    let active = move_state
                        .read(cx)
                        .drag
                        .as_ref()
                        .map(|drag| (drag.start, drag.offset));
                    if let Some((start, offset)) = active {
                        if event.dragging() {
                            move_source.set_offset(
                                axis,
                                px(geometry.drag(
                                    offset.into(),
                                    (coordinate(event.position, axis) - start).into(),
                                )),
                            );
                            cx.stop_propagation();
                        } else {
                            move_state.update(cx, |state, _| state.drag = None);
                        }
                        window.refresh();
                    } else if move_hitbox.is_hovered(window) != move_state.read(cx).hover {
                        window.refresh();
                    }
                });
                if !draggable {
                    return;
                }
                let down_state = state.clone();
                window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
                    if !phase.bubble()
                        || event.button != MouseButton::Left
                        || !hitbox.is_hovered(window)
                    {
                        return;
                    }
                    let position = coordinate(event.position, axis);
                    let local = f32::from(position - coordinate(viewport.origin, axis));
                    let offset = source.offset(axis);
                    if local >= geometry.start && local <= geometry.start + geometry.thumb {
                        source.drag_started();
                        let drag = Drag {
                            source: source.clone(),
                            start: position,
                            offset,
                        };
                        down_state.update(cx, |state, _| state.drag = Some(drag));
                    } else {
                        let direction = if local < geometry.start { -1.0 } else { 1.0 };
                        source.set_offset(axis, offset + length(viewport, axis) * direction);
                    }
                    window.prevent_default();
                    cx.stop_propagation();
                    window.refresh();
                });
                window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
                    if phase.capture()
                        && event.button == MouseButton::Left
                        && state.read(cx).drag.is_some()
                    {
                        state.update(cx, |state, _| state.drag = None);
                        window.refresh();
                        cx.stop_propagation();
                    }
                });
            },
        )
        .absolute()
        .size(px(0.0))
    }
}
struct State {
    hover: bool,
    drag: Option<Drag>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_and_drag() {
        let options = ScrollbarGeometry {
            track_inset: px(2.0),
            cross_inset: px(0.0),
            thickness: px(4.0),
            min_thumb_length: px(24.0),
            max_thumb_fraction: 0.9,
            hit_width: px(12.0),
        };
        let geometry = |viewport, max, offset| Geometry::new(viewport, max, offset, options);
        assert!(geometry(100.0, 0.0, 0.0).is_none());
        assert!(geometry(27.0, 100.0, 0.0).is_none());
        assert!(geometry(f32::NAN, 100.0, 0.0).is_none());
        let g = geometry(100.0, 900.0, 0.0).unwrap();
        assert_eq!(g.thumb, 24.0);
        assert_eq!(g.start, 2.0);
        assert_eq!(g.drag(0.0, g.travel), 900.0);
        assert_eq!(g.drag(450.0, -g.travel), 0.0);
        assert_eq!(geometry(100.0, 900.0, 1000.0).unwrap().start, 74.0);
        assert_eq!(geometry(100.0, 1.0, -1.0).unwrap().thumb, 96.0 * 0.9);
        assert!(geometry(28.0, 100.0, 0.0).unwrap().travel > 0.0);
        let g = geometry(204.0, 204.0, 102.0).unwrap();
        assert_eq!(g.thumb, 100.0);
        assert_eq!(g.start, 52.0);
    }
}
