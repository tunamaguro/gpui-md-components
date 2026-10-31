//! Slider（連続値・離散値）。

use gpui::{
    App, Bounds, Corners, ElementId, Hsla, IntoElement, Pixels, RenderOnce, SharedString, Window,
    canvas, fill, point, prelude::*, px, size,
};
use md3_theme::md3_tokens::component::slider as tokens;
use md3_theme::md3_tokens::layout;
use md3_theme::{CornerRadius, Md3ThemeExt};

use crate::FocusRing;

/// スライダー。
///
/// 矢印で 1 ステップ、Page Up / Page Down で 10 ステップ、Home / End で端に移る。
/// 支援技術からの増減と値の設定も受ける。離散値は最小から刻みごとの値と最大値を選べる。
#[derive(IntoElement)]
pub struct Slider {
    inner: gpui_headless::Slider,
    disabled: bool,
}

impl Slider {
    /// ID と値から作る。範囲は 0〜1。
    pub fn new(id: impl Into<ElementId>, value: f32) -> Self {
        let id = id.into();
        Self {
            inner: gpui_headless::Slider::new(id.clone(), value)
                .reveal_margin(crate::focus::reveal_margin())
                .minimum_drag_width(px(1.0))
                .group(format!("md3-{id}")),
            disabled: false,
        }
    }
    /// 読み上げる名前を指定する。表示文字を優先し、表示ラベルがある場合は名前に含める（WCAG 2.5.3）。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.inner = self.inner.aria_label(label);
        self
    }
    /// 範囲を変える。
    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.inner = self.inner.range(min, max);
        self
    }
    /// 離散値にする。
    pub fn step(mut self, step: f32) -> Self {
        self.inner = self.inner.step(step);
        self
    }
    /// 無効にする。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.inner = self.inner.disabled(disabled);
        self.disabled = disabled;
        self
    }
    /// 値が変わったときの処理。
    pub fn on_change(mut self, handler: impl Fn(f32, &mut Window, &mut App) + 'static) -> Self {
        self.inner = self.inner.on_change(handler);
        self
    }
}

/// 描画する色。
#[derive(Clone, Copy)]
struct SliderPaint {
    active: Hsla,
    inactive: Hsla,
    handle: Hsla,
    stop: Hsla,
    handle_width: Pixels,
}

impl RenderOnce for Slider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (base, state) = self.inner.build(window, cx);
        let control = state.control;
        let fraction = state.fraction;
        let pressed = state.dragging;
        let theme = cx.md3();
        let paint = if self.disabled {
            SliderPaint {
                active: theme
                    .color(tokens::DISABLED_ACTIVE_TRACK_COLOR)
                    .opacity(tokens::DISABLED_ACTIVE_TRACK_OPACITY),
                inactive: theme
                    .color(tokens::DISABLED_INACTIVE_TRACK_COLOR)
                    .opacity(tokens::DISABLED_INACTIVE_TRACK_OPACITY),
                handle: theme
                    .color(tokens::DISABLED_HANDLE_COLOR)
                    .opacity(tokens::DISABLED_HANDLE_OPACITY),
                stop: theme
                    .color(tokens::DISABLED_STOP_COLOR)
                    .opacity(tokens::DISABLED_ACTIVE_TRACK_OPACITY),
                handle_width: px(tokens::DISABLED_HANDLE_WIDTH),
            }
        } else {
            SliderPaint {
                active: theme.color(tokens::ACTIVE_TRACK_COLOR),
                inactive: theme.color(tokens::INACTIVE_TRACK_COLOR),
                handle: theme.color(tokens::HANDLE_COLOR),
                // トークンの StopIndicatorColor は非活性トラックと同じ色で見えないため、
                // Compose（SliderDefaults.colors の inactiveTickColor）と同じく活性トラックの色を使う。
                stop: theme.color(tokens::ACTIVE_TRACK_COLOR),
                handle_width: if pressed || control.focus_visible {
                    px(tokens::PRESSED_HANDLE_WIDTH)
                } else {
                    px(tokens::HANDLE_WIDTH)
                },
            }
        };

        base.h(px(tokens::HANDLE_HEIGHT))
            .w_full()
            .min_w(px(tokens::HANDLE_HEIGHT * 2.0))
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, (), window, _| paint_slider(bounds, fraction, paint, window),
                )
                .size_full(),
            )
            .child(FocusRing::new(CornerRadius::Fixed(px(4.0))).visible(control.focus_visible))
    }
}

/// トラック・つまみ・終端の点を描く。
fn paint_slider(bounds: Bounds<Pixels>, fraction: f32, paint: SliderPaint, window: &mut Window) {
    let track_height = px(tokens::INACTIVE_TRACK_HEIGHT);
    let gap = px(tokens::ACTIVE_HANDLE_LEADING_SPACE);
    let outer = track_height / 2.0;
    let inner = px(layout::SLIDER_TRACK_INSIDE_CORNER);
    let left = bounds.origin.x;
    let right = bounds.origin.x + bounds.size.width;
    let center_y = bounds.origin.y + bounds.size.height / 2.0;
    let track_top = center_y - track_height / 2.0;
    let handle_x =
        left + paint.handle_width / 2.0 + (bounds.size.width - paint.handle_width) * fraction;

    let active_end = handle_x - paint.handle_width / 2.0 - gap;
    if active_end > left {
        let corners = Corners {
            top_left: outer,
            bottom_left: outer,
            top_right: inner,
            bottom_right: inner,
        };
        window.paint_quad(
            fill(
                Bounds::new(
                    point(left, track_top),
                    size(active_end - left, track_height),
                ),
                paint.active,
            )
            .corner_radii(corners),
        );
    }
    let inactive_start = handle_x + paint.handle_width / 2.0 + gap;
    if right > inactive_start {
        let corners = Corners {
            top_left: inner,
            bottom_left: inner,
            top_right: outer,
            bottom_right: outer,
        };
        window.paint_quad(
            fill(
                Bounds::new(
                    point(inactive_start, track_top),
                    size(right - inactive_start, track_height),
                ),
                paint.inactive,
            )
            .corner_radii(corners),
        );
        let stop = px(tokens::STOP_INDICATOR_SIZE);
        let stop_x = right - px(tokens::STOP_INDICATOR_TRAILING_SPACE) - stop;
        if stop_x > inactive_start {
            window.paint_quad(
                fill(
                    Bounds::new(point(stop_x, center_y - stop / 2.0), size(stop, stop)),
                    paint.stop,
                )
                .corner_radii(stop / 2.0),
            );
        }
    }
    let handle_height = px(tokens::HANDLE_HEIGHT);
    window.paint_quad(
        fill(
            Bounds::new(
                point(
                    handle_x - paint.handle_width / 2.0,
                    center_y - handle_height / 2.0,
                ),
                size(paint.handle_width, handle_height),
            ),
            paint.handle,
        )
        .corner_radii(paint.handle_width / 2.0),
    );
}
