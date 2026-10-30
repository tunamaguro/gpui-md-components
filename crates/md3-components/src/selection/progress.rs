//! Progress indicator（線・円）。

use std::f32::consts::{PI, TAU};
use std::time::Duration;

use gpui::{
    Animation, AnimationExt, App, Bounds, ElementId, Hsla, IntoElement, PathBuilder, Pixels, Point,
    RenderOnce, SharedString, Window, canvas, fill, point, prelude::*, px, size,
};
use md3_theme::Md3ThemeExt;
use md3_theme::md3_tokens::component::{
    circular_progress_indicator as circular, linear_progress_indicator as linear,
    progress_indicator as common,
};
use md3_theme::md3_tokens::layout;

use gpui_headless::Progress;

#[derive(Clone, Copy)]
struct ProgressPaint {
    active: Hsla,
    track: Hsla,
    stop: Hsla,
}

fn progress_paint(cx: &App) -> ProgressPaint {
    let theme = cx.md3();
    ProgressPaint {
        active: theme.color(common::ACTIVE_INDICATOR_COLOR),
        track: theme.color(common::TRACK_COLOR),
        stop: theme.color(common::STOP_COLOR),
    }
}

/// 線の進み具合。`value` が `None` なら不定（動き続ける）。
#[derive(IntoElement)]
pub struct LinearProgress {
    id: ElementId,
    value: Option<f32>,
    label: SharedString,
}

impl LinearProgress {
    /// ID、進み具合（0〜1、不定なら `None`）、ラベルから作る。
    pub fn new(
        id: impl Into<ElementId>,
        value: Option<f32>,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            value,
            label: label.into(),
        }
    }
}

impl RenderOnce for LinearProgress {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let paint = progress_paint(cx);
        let base = Progress::new(self.id, self.value, self.label)
            .build()
            .h(px(linear::HEIGHT))
            .w_full()
            .min_w(px(linear::HEIGHT * 4.0));
        match self.value {
            Some(value) => {
                let value = value.clamp(0.0, 1.0);
                base.child(
                    canvas(
                        |_, _, _| (),
                        move |bounds, (), window, _| {
                            paint_linear(bounds, 0.0, value, true, paint, window)
                        },
                    )
                    .size_full(),
                )
                .into_any_element()
            }
            // Compose の不定アニメーション（2 本の線が追いかける）を 1 本の線に簡略化している。
            None => base
                .with_animation(
                    "indeterminate",
                    Animation::new(Duration::from_millis(
                        layout::LINEAR_PROGRESS_ANIMATION_DURATION,
                    ))
                    .repeat(),
                    move |this, t| {
                        let head = (t * 1.5).min(1.0);
                        let tail = (t * 1.5 - 0.5).clamp(0.0, 1.0);
                        this.child(
                            canvas(
                                |_, _, _| (),
                                move |bounds, (), window, _| {
                                    paint_linear(bounds, tail, head, false, paint, window)
                                },
                            )
                            .size_full(),
                        )
                    },
                )
                .into_any_element(),
        }
    }
}

/// `[start, end]`（0〜1）を活性部分として描く。残りはすき間を空けてトラックにする。
fn paint_linear(
    bounds: Bounds<Pixels>,
    start: f32,
    end: f32,
    stop: bool,
    paint: ProgressPaint,
    window: &mut Window,
) {
    let thickness = px(linear::TRACK_THICKNESS);
    let gap = px(linear::TRACK_ACTIVE_SPACE);
    let radius = thickness / 2.0;
    let width = bounds.size.width;
    let y = bounds.origin.y + (bounds.size.height - thickness) / 2.0;
    let x = |f: f32| bounds.origin.x + width * f;
    let mut bar = |from: Pixels, to: Pixels, color: Hsla| {
        if to - from > px(0.5) {
            window.paint_quad(
                fill(
                    Bounds::new(point(from, y), size(to - from, thickness)),
                    color,
                )
                .corner_radii(radius),
            );
        }
    };
    let has_active = end > start;
    if has_active {
        bar(x(start), x(end), paint.active);
    }
    let before_end = if has_active { x(start) - gap } else { x(start) };
    bar(bounds.origin.x, before_end, paint.track);
    let after_start = if has_active { x(end) + gap } else { x(end) };
    bar(after_start, x(1.0), paint.track);
    if stop {
        let stop = px(linear::STOP_SIZE);
        let stop_x = x(1.0) - px(linear::STOP_TRAILING_SPACE) - stop;
        if stop_x > after_start {
            window.paint_quad(
                fill(
                    Bounds::new(
                        point(stop_x, y + (thickness - stop) / 2.0),
                        size(stop, stop),
                    ),
                    paint.stop,
                )
                .corner_radii(stop / 2.0),
            );
        }
    }
}

/// 円の進み具合。`value` が `None` なら不定（回り続ける）。
#[derive(IntoElement)]
pub struct CircularProgress {
    id: ElementId,
    value: Option<f32>,
    label: SharedString,
}

impl CircularProgress {
    /// ID、進み具合（0〜1、不定なら `None`）、ラベルから作る。
    pub fn new(
        id: impl Into<ElementId>,
        value: Option<f32>,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            value,
            label: label.into(),
        }
    }
}

impl RenderOnce for CircularProgress {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let paint = progress_paint(cx);
        let base = Progress::new(self.id, self.value, self.label)
            .build()
            .size(px(circular::SIZE));
        match self.value {
            Some(value) => {
                let value = value.clamp(0.0, 1.0);
                base.child(
                    canvas(
                        |_, _, _| (),
                        move |bounds, (), window, _| {
                            paint_circular(bounds, -PI / 2.0, value * TAU, true, paint, window)
                        },
                    )
                    .size_full(),
                )
                .into_any_element()
            }
            // Compose の不定アニメーション（回転と伸縮の組み合わせ）を、回転しながら伸び縮みする 1 本の弧に簡略化している。
            None => base
                .with_animation(
                    "indeterminate",
                    Animation::new(Duration::from_millis(
                        layout::CIRCULAR_PROGRESS_ANIMATION_DURATION,
                    ))
                    .repeat(),
                    move |this, t| {
                        let min = layout::CIRCULAR_PROGRESS_MIN_SWEEP;
                        let max = layout::CIRCULAR_PROGRESS_MAX_SWEEP;
                        let sweep = (min + (max - min) * (t * 4.0 * PI).sin().abs()) * TAU;
                        let start = t * 4.0 * TAU - PI / 2.0;
                        this.child(
                            canvas(
                                |_, _, _| (),
                                move |bounds, (), window, _| {
                                    paint_circular(bounds, start, sweep, true, paint, window)
                                },
                            )
                            .size_full(),
                        )
                    },
                )
                .into_any_element(),
        }
    }
}

/// `start` から時計回りに `sweep`（ラジアン）の弧を描き、残りをすき間を空けてトラックにする。
fn paint_circular(
    bounds: Bounds<Pixels>,
    start: f32,
    sweep: f32,
    track: bool,
    paint: ProgressPaint,
    window: &mut Window,
) {
    let thickness = px(circular::TRACK_THICKNESS);
    let diameter = bounds.size.width.min(bounds.size.height);
    let radius = (diameter - thickness) / 2.0;
    if radius <= px(0.0) {
        return;
    }
    let center = point(
        bounds.origin.x + bounds.size.width / 2.0,
        bounds.origin.y + bounds.size.height / 2.0,
    );
    // 線の端は丸める（stroke の端は四角いので、端に円を重ねる）。すき間は端の丸みの分も空ける。
    let gap = (px(circular::TRACK_ACTIVE_SPACE) + thickness) / radius;
    if sweep > 0.0 {
        paint_arc(
            center,
            radius,
            start,
            sweep,
            thickness,
            paint.active,
            window,
        );
    }
    if track {
        let track_sweep = TAU - sweep - if sweep > 0.0 { 2.0 * gap } else { 0.0 };
        if track_sweep > 0.0 {
            let track_start = start + sweep + if sweep > 0.0 { gap } else { 0.0 };
            paint_arc(
                center,
                radius,
                track_start,
                track_sweep,
                thickness,
                paint.track,
                window,
            );
        }
    }
}

fn paint_arc(
    center: Point<Pixels>,
    radius: Pixels,
    start: f32,
    sweep: f32,
    thickness: Pixels,
    color: Hsla,
    window: &mut Window,
) {
    let at = |angle: f32| {
        point(
            center.x + radius * angle.cos(),
            center.y + radius * angle.sin(),
        )
    };
    let sweep = sweep.min(TAU - 0.001);
    let mut builder = PathBuilder::stroke(thickness);
    builder.move_to(at(start));
    // 半周を超える弧は 2 つに分ける。
    let mid = start + sweep / 2.0;
    builder.arc_to(point(radius, radius), px(0.0), false, true, at(mid));
    builder.arc_to(
        point(radius, radius),
        px(0.0),
        false,
        true,
        at(start + sweep),
    );
    if let Ok(path) = builder.build() {
        window.paint_path(path, color);
    }
    for angle in [start, start + sweep] {
        let cap = at(angle);
        window.paint_quad(
            fill(
                Bounds::new(
                    point(cap.x - thickness / 2.0, cap.y - thickness / 2.0),
                    size(thickness, thickness),
                ),
                color,
            )
            .corner_radii(thickness / 2.0),
        );
    }
}
