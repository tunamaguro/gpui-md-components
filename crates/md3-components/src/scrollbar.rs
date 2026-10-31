// Ported from AndroidX Compose (https://github.com/androidx/androidx)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Scrollbar.kt
// @ b8256a524b9fe4bb6cd0fc7c72aad9dd08877ca6
// Copyright 2026 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
//! スクロール領域の末尾の子として置く、装飾用スクロールバー。

use gpui::{
    App, Axis, ElementId, IntoElement, ListState, Pixels, RenderOnce, ScrollHandle,
    UniformListScrollHandle, Window, fill, px,
};
use gpui_headless::{ScrollbarGeometry, ScrollbarSource};
use md3_theme::{Md3ThemeExt, md3_tokens::component::scrollbar as tokens};
use std::time::Duration;

pub use crate::scroll::ScrollArea;

/// ポインタでの操作。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollbarInteraction {
    /// 位置を表示するだけ。
    Passive,
    /// つまみのドラッグとトラックのページ移動（Web でも有効）。
    #[default]
    Draggable,
}

impl From<ScrollbarInteraction> for gpui_headless::ScrollbarInteraction {
    fn from(interaction: ScrollbarInteraction) -> Self {
        match interaction {
            ScrollbarInteraction::Passive => Self::Passive,
            ScrollbarInteraction::Draggable => Self::Draggable,
        }
    }
}

/// スクロールバーが位置を読む先。`ScrollHandle`、`UniformListScrollHandle`、`ListState` から作る。
pub struct ScrollbarTarget(
    Box<dyn FnOnce(ElementId, ScrollbarGeometry) -> gpui_headless::Scrollbar>,
);

impl ScrollbarTarget {
    pub(crate) fn new(source: impl ScrollbarSource) -> Self {
        Self(Box::new(move |id, geometry| {
            gpui_headless::Scrollbar::new(id, source, geometry)
        }))
    }
}

impl From<ScrollHandle> for ScrollbarTarget {
    fn from(handle: ScrollHandle) -> Self {
        Self::new(handle)
    }
}

impl From<UniformListScrollHandle> for ScrollbarTarget {
    fn from(handle: UniformListScrollHandle) -> Self {
        Self::new(handle)
    }
}

impl From<ListState> for ScrollbarTarget {
    fn from(state: ListState) -> Self {
        Self::new(state)
    }
}

/// スクロールバー。供給元の領域内に最後の子として追加する。
///
/// 自身はサイズを取らず、供給元の表示領域を基準に描画する。通常の Div は
/// `.track_scroll(&handle)`、UniformList は外側のコンテナにこの部品を置く。
/// 役割とフォーカスを持たず、アクセシビリティツリーには現れない。
#[derive(IntoElement)]
pub struct Scrollbar {
    id: ElementId,
    target: ScrollbarTarget,
    axis: Axis,
    interaction: ScrollbarInteraction,
    fade: bool,
    thumb_opacity: f32,
    cross_inset: Pixels,
    hit_width: Pixels,
}
impl Scrollbar {
    /// 安定 ID とスクロール位置の供給元から縦バーを作る。
    pub fn new(id: impl Into<ElementId>, target: impl Into<ScrollbarTarget>) -> Self {
        Self {
            id: id.into(),
            target: target.into(),
            axis: Axis::Vertical,
            interaction: ScrollbarInteraction::default(),
            fade: true,
            thumb_opacity: tokens::THUMB_OPACITY,
            cross_inset: px(tokens::CROSS_AXIS_TRACK_INSET),
            // デスクトップの操作領域としての実装判断。
            hit_width: px(12.0),
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
    /// つまみの不透明度を変える。
    pub(crate) fn thumb_opacity(mut self, opacity: f32) -> Self {
        self.thumb_opacity = opacity;
        self
    }
    /// 表示領域の端からの間隔とポインタの操作領域の幅を変える。
    pub(crate) fn placement(mut self, cross_inset: Pixels, hit_width: Pixels) -> Self {
        self.cross_inset = cross_inset;
        self.hit_width = hit_width;
        self
    }
}

struct State {
    clock: Box<dyn Fn(&App) -> Duration>,
    last_activity: Duration,
    last_offset: Pixels,
    hover: bool,
    dragging: bool,
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
        let state = window.use_keyed_state(
            ElementId::NamedChild(self.id.clone().into(), "fade".into()),
            cx,
            |_, cx| {
                let start = cx.background_executor().now();
                State {
                    clock: Box::new(move |cx| cx.background_executor().now().duration_since(start)),
                    last_activity: Duration::ZERO,
                    last_offset: px(0.0),
                    hover: false,
                    dragging: false,
                }
            },
        );
        let geometry = ScrollbarGeometry {
            track_inset: px(tokens::MAIN_AXIS_TRACK_INSET),
            cross_inset: self.cross_inset,
            thickness: px(tokens::THICKNESS),
            min_thumb_length: px(tokens::THUMB_MIN_LENGTH),
            max_thumb_fraction: tokens::THUMB_MAX_LENGTH_FRACTION,
            hit_width: self.hit_width,
        };
        let color = cx.md3().color(tokens::THUMB_COLOR);
        (self.target.0)(self.id, geometry)
            .axis(self.axis)
            .interaction(self.interaction.into())
            .build(window, cx, move |bar, window, cx| {
                let alpha = state.update(cx, |state, cx| {
                    let now = (state.clock)(cx);
                    if bar.offset != state.last_offset
                        || bar.hovered
                        || state.hover
                        || bar.dragging
                        || state.dragging
                    {
                        state.last_activity = now;
                    }
                    state.last_offset = bar.offset;
                    state.hover = bar.hovered;
                    state.dragging = bar.dragging;
                    opacity(
                        now.saturating_sub(state.last_activity),
                        bar.hovered || bar.dragging,
                        self.fade,
                        cx.reduce_motion(),
                    )
                });
                if alpha > 0.0 {
                    window.paint_quad(
                        fill(bar.thumb, color.opacity(self.thumb_opacity * alpha))
                            .corner_radii(geometry.thickness / 2.0),
                    );
                    if self.fade && !bar.hovered && !bar.dragging {
                        window.request_animation_frame();
                    }
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
