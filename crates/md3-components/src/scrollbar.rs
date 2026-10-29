// Ported from AndroidX Compose (https://github.com/androidx/androidx)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Scrollbar.kt
// @ b8256a524b9fe4bb6cd0fc7c72aad9dd08877ca6
// Copyright 2026 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
//! スクロール領域の末尾の子として置く、装飾用スクロールバー。

use gpui::{
    App, Axis, Bounds, ElementId, HitboxBehavior, IntoElement, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Pixels, Point, RenderOnce, ScrollHandle, UniformListScrollHandle,
    Window, canvas, fill, point, prelude::*, px, size,
};
use md3_theme::{Md3ThemeExt, md3_tokens::component::scrollbar as tokens};
use std::{rc::Rc, time::Duration};

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

/// スクロールバー。供給元の領域内に最後の子として追加する。
///
/// 自身はサイズを取らず、供給元の表示領域を基準に描画する。通常の Div は
/// `.track_scroll(&handle)`、UniformList は外側のコンテナにこの部品を置く。
/// 役割とフォーカスを持たず、アクセシビリティツリーには現れない。
#[derive(IntoElement)]
pub struct Scrollbar {
    id: ElementId,
    source: Rc<dyn ScrollbarSource>,
    axis: Axis,
    interaction: ScrollbarInteraction,
    fade: bool,
}
impl Scrollbar {
    /// 安定 ID とスクロール位置の供給元から縦バーを作る。
    pub fn new(id: impl Into<ElementId>, source: impl ScrollbarSource) -> Self {
        Self {
            id: id.into(),
            source: Rc::new(source),
            axis: Axis::Vertical,
            interaction: ScrollbarInteraction::default(),
            fade: true,
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
    /// false ならスクロール可能な間は常に表示する。
    pub fn fade(mut self, fade: bool) -> Self {
        self.fade = fade;
        self
    }
}

#[derive(Clone, Copy, Debug)]
struct Geometry {
    start: f32,
    thumb: f32,
    travel: f32,
    max: f32,
}
impl Geometry {
    fn new(viewport: f32, max: f32, offset: f32) -> Option<Self> {
        let track = viewport - 2.0 * tokens::MAIN_AXIS_TRACK_INSET;
        if !viewport.is_finite()
            || !max.is_finite()
            || max <= 0.0
            || track < tokens::THUMB_MIN_LENGTH
        {
            return None;
        }
        // 短い領域では最小長と最大比率が矛盾するため、最大比率を優先して panic を避ける。
        let upper = track * tokens::THUMB_MAX_LENGTH_FRACTION;
        let thumb =
            (track * viewport / (viewport + max)).clamp(tokens::THUMB_MIN_LENGTH.min(upper), upper);
        let travel = track - thumb;
        let offset = if offset.is_finite() {
            offset.clamp(0.0, max)
        } else {
            0.0
        };
        Some(Self {
            start: tokens::MAIN_AXIS_TRACK_INSET + travel * offset / max,
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

struct State {
    clock: Box<dyn Fn(&App) -> Duration>,
    last_activity: Duration,
    last_offset: Pixels,
    hover: bool,
    drag: Option<(Pixels, Pixels)>,
}
fn opacity(elapsed: Duration, held: bool, fade: bool, reduced: bool) -> f32 {
    if held || !fade || elapsed < Duration::from_millis(tokens::FADE_DELAY_MILLIS) {
        return 1.0;
    }
    if reduced {
        return 0.0;
    }
    (1.0 - (elapsed.as_secs_f32() * 1000.0 - tokens::FADE_DELAY_MILLIS as f32)
        / tokens::FADE_DURATION_MILLIS as f32)
        .clamp(0.0, 1.0)
}

impl RenderOnce for Scrollbar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id, cx, |_, cx| {
            let start = cx.background_executor().now();
            State {
                clock: Box::new(move |cx| cx.background_executor().now().duration_since(start)),
                last_activity: Duration::ZERO,
                last_offset: px(0.0),
                hover: false,
                drag: None,
            }
        });
        let source = self.source;
        let pre_source = source.clone();
        let axis = self.axis;
        let draggable = self.interaction == ScrollbarInteraction::Draggable;
        let color = cx.md3().color(tokens::THUMB_COLOR);
        canvas(
            move |_, window, _| {
                let viewport = pre_source.viewport();
                let geometry = Geometry::new(
                    length(viewport, axis).into(),
                    pre_source.max_offset(axis).into(),
                    pre_source.offset(axis).into(),
                );
                // 12dp はデスクトップの操作領域としての実装判断。描画の太さは変更しない。
                let mut hit = viewport;
                match axis {
                    Axis::Vertical => {
                        hit.origin.x += (hit.size.width - px(12.0)).max(px(0.0));
                        hit.size.width = hit.size.width.min(px(12.0));
                    }
                    Axis::Horizontal => {
                        hit.origin.y += (hit.size.height - px(12.0)).max(px(0.0));
                        hit.size.height = hit.size.height.min(px(12.0));
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
                let alpha = state.update(cx, |state, cx| {
                    let now = (state.clock)(cx);
                    if !draggable {
                        state.drag = None;
                    }
                    if offset != state.last_offset || hovered || state.hover || state.drag.is_some()
                    {
                        state.last_activity = now;
                    }
                    state.last_offset = offset;
                    state.hover = hovered;
                    opacity(
                        now.saturating_sub(state.last_activity),
                        hovered || state.drag.is_some(),
                        self.fade,
                        cx.reduce_motion(),
                    )
                });
                if alpha > 0.0 {
                    let thickness = px(tokens::THICKNESS);
                    let inset = px(tokens::CROSS_AXIS_TRACK_INSET);
                    let thumb = match axis {
                        Axis::Vertical => Bounds::new(
                            point(
                                viewport.right() - inset - thickness,
                                viewport.top() + px(geometry.start),
                            ),
                            size(thickness, px(geometry.thumb)),
                        ),
                        Axis::Horizontal => Bounds::new(
                            point(
                                viewport.left() + px(geometry.start),
                                viewport.bottom() - inset - thickness,
                            ),
                            size(px(geometry.thumb), thickness),
                        ),
                    };
                    window.paint_quad(
                        fill(thumb, color.opacity(tokens::THUMB_OPACITY * alpha))
                            .corner_radii(thickness / 2.0),
                    );
                    if self.fade && !hovered && state.read(cx).drag.is_none() {
                        window.request_animation_frame();
                    }
                }
                let move_state = state.clone();
                let move_source = source.clone();
                let move_hitbox = hitbox.clone();
                window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                    if !phase.capture() {
                        return;
                    }
                    let active = move_state.read(cx).drag;
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
                        down_state.update(cx, |state, _| state.drag = Some((position, offset)));
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
                        state.update(cx, |state, cx| {
                            state.drag = None;
                            state.last_activity = (state.clock)(cx);
                        });
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn geometry_and_drag() {
        assert!(Geometry::new(100.0, 0.0, 0.0).is_none());
        assert!(Geometry::new(27.0, 100.0, 0.0).is_none());
        assert!(Geometry::new(f32::NAN, 100.0, 0.0).is_none());
        let g = Geometry::new(100.0, 900.0, 0.0).unwrap();
        assert_eq!(g.thumb, 24.0);
        assert_eq!(g.start, 2.0);
        assert_eq!(g.drag(0.0, g.travel), 900.0);
        assert_eq!(g.drag(450.0, -g.travel), 0.0);
        assert_eq!(Geometry::new(100.0, 900.0, 1000.0).unwrap().start, 74.0);
        assert_eq!(Geometry::new(100.0, 1.0, -1.0).unwrap().thumb, 96.0 * 0.9);
        assert!(Geometry::new(28.0, 100.0, 0.0).unwrap().travel > 0.0);
        let g = Geometry::new(204.0, 204.0, 102.0).unwrap();
        assert_eq!(g.thumb, 100.0);
        assert_eq!(g.start, 52.0);
    }
    #[test]
    fn fade_respects_motion_and_interaction() {
        assert_eq!(opacity(Duration::from_millis(399), false, true, true), 1.0);
        assert_eq!(opacity(Duration::from_millis(400), false, true, true), 0.0);
        assert_eq!(opacity(Duration::from_millis(525), false, true, false), 0.5);
        assert_eq!(opacity(Duration::from_secs(1), false, true, false), 0.0);
        assert_eq!(opacity(Duration::from_secs(1), true, true, true), 1.0);
        assert_eq!(opacity(Duration::from_secs(1), false, false, true), 1.0);
    }
}
