//! Slider（連続値・離散値）。

use std::rc::Rc;

use gpui::{
    App, Bounds, Corners, ElementId, Entity, Hsla, IntoElement, KeyDownEvent, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, RenderOnce, Role, SharedString, Window,
    accesskit::{Action, ActionData},
    canvas, fill, point,
    prelude::*,
    px, size,
};
use md3_theme::md3_tokens::component::slider as tokens;
use md3_theme::md3_tokens::layout;
use md3_theme::{CornerRadius, Md3ThemeExt};

use crate::FocusRing;
use crate::a11y::{A11yExt, A11yNode, RangeValue};
use crate::control::control;

/// ドラッグの状態。描画をまたいで保つ。
#[derive(Default)]
struct DragState {
    bounds: Option<Bounds<Pixels>>,
    dragging: bool,
}

type ChangeHandler = Rc<dyn Fn(f32, &mut Window, &mut App) + 'static>;

/// スライダー。ラベル（読み上げ用の名前）が必須。
///
/// キー操作（APG の Slider）: 矢印で 1 ステップ、Page Up / Page Down で 10 ステップ、Home / End で端。
/// 支援技術からの増減（Increment / Decrement）と値の設定（SetValue）も受ける（WCAG 2.2 の 4.1.2）。
///
/// 離散値の値は `最小 + 刻み × n` と最大値。刻みで範囲を割り切れなくても最大値は選べる。
#[derive(IntoElement)]
pub struct Slider {
    id: ElementId,
    value: f32,
    min: f32,
    max: f32,
    step: Option<f32>,
    label: SharedString,
    disabled: bool,
    on_change: Option<ChangeHandler>,
}

impl Slider {
    /// ID、値、ラベルから作る。範囲は 0〜1。
    pub fn new(id: impl Into<ElementId>, value: f32, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            value,
            min: 0.0,
            max: 1.0,
            step: None,
            label: label.into(),
            disabled: false,
            on_change: None,
        }
    }

    /// 範囲を変える。
    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max.max(min);
        self
    }

    /// 離散値にする（値はこの刻みに丸められる）。
    pub fn step(mut self, step: f32) -> Self {
        self.step = (step > 0.0).then_some(step);
        self
    }

    /// 無効にする。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 値が変わったときの処理。
    pub fn on_change(mut self, handler: impl Fn(f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// アクセシビリティツリーに出す情報。
    pub fn a11y_node(&self) -> A11yNode {
        A11yNode::new(Role::Slider)
            .label(self.label.clone())
            .range(RangeValue {
                // 描画と同じく、範囲に収めて刻みに丸めた値を読ませる。
                value: f64::from(self.slider_range().clamp(self.value)),
                min: f64::from(self.min),
                max: f64::from(self.max),
                step: Some(f64::from(self.key_step())),
            })
            .disabled(self.disabled)
    }

    /// キー操作 1 回の変化量。離散値ならその刻み、連続値なら範囲の 1%。
    fn key_step(&self) -> f32 {
        self.step.unwrap_or((self.max - self.min) / 100.0)
    }

    fn slider_range(&self) -> SliderRange {
        SliderRange {
            min: self.min,
            max: self.max,
            step: self.step,
            key_step: self.key_step(),
        }
    }
}

/// 値の計算（純関数）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SliderRange {
    pub min: f32,
    pub max: f32,
    pub step: Option<f32>,
    pub key_step: f32,
}

impl SliderRange {
    /// 範囲に収め、離散値なら最も近い値（`最小 + 刻み × n` か最大値）に丸める。
    pub(crate) fn clamp(&self, value: f32) -> f32 {
        if value.is_nan() {
            return self.min;
        }
        let value = value.clamp(self.min, self.max);
        match self.step {
            Some(step) => {
                let snapped = ((value - self.min) / step).round().mul_add(step, self.min);
                if snapped >= self.max || self.max - value < (value - snapped).abs() {
                    self.max
                } else {
                    snapped
                }
            }
            None => value,
        }
    }

    /// `steps` ステップ進めた値（負なら戻す）。離散値なら刻みの格子の上を動き、端で止まる。
    pub(crate) fn step_by(&self, value: f32, steps: i32) -> f32 {
        let value = self.clamp(value);
        let Some(step) = self.step else {
            return self.clamp(value + self.key_step * steps as f32);
        };
        // 格子の上にない値（最大値など）からは、進む向きの最初の格子点を 1 ステップ目にする。
        const EPSILON: f32 = 1e-4;
        let position = (value - self.min) / step;
        let base = if steps > 0 {
            (position + EPSILON).floor()
        } else {
            (position - EPSILON).ceil()
        };
        let next = (base + steps as f32).mul_add(step, self.min);
        if next >= self.max {
            self.max
        } else {
            next.max(self.min)
        }
    }

    /// 0〜1 の位置にする。
    pub(crate) fn fraction(&self, value: f32) -> f32 {
        if self.max > self.min {
            ((value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// キーに対応する新しい値。対応しないキーなら `None`。
    pub(crate) fn key(&self, value: f32, key: &str) -> Option<f32> {
        match key {
            "right" | "up" => Some(self.step_by(value, 1)),
            "left" | "down" => Some(self.step_by(value, -1)),
            "pageup" => Some(self.step_by(value, 10)),
            "pagedown" => Some(self.step_by(value, -10)),
            "home" => Some(self.min),
            "end" => Some(self.max),
            _ => None,
        }
    }

    /// 支援技術からの操作に対応する新しい値。対応しない操作なら `None`。
    pub(crate) fn a11y_action(
        &self,
        value: f32,
        action: Action,
        data: Option<&ActionData>,
    ) -> Option<f32> {
        match (action, data) {
            (Action::Increment, _) => Some(self.step_by(value, 1)),
            (Action::Decrement, _) => Some(self.step_by(value, -1)),
            (Action::SetValue, Some(ActionData::NumericValue(next))) => {
                Some(self.clamp(*next as f32))
            }
            _ => None,
        }
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
        let node = self.a11y_node();
        let range = self.slider_range();
        let value = range.clamp(self.value);
        let fraction = range.fraction(value);
        let drag: Entity<DragState> = window.use_keyed_state(
            ElementId::NamedChild(std::sync::Arc::new(self.id.clone()), "drag".into()),
            cx,
            |_, _| DragState::default(),
        );
        let (base, control) = control(&self.id, self.disabled, true, window, cx);
        let theme = cx.md3();
        let pressed = drag.read(cx).dragging;
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

        let mut slider = base
            .a11y(node)
            .h(px(tokens::HANDLE_HEIGHT))
            .w_full()
            .min_w(px(tokens::HANDLE_HEIGHT * 2.0))
            .child(
                canvas(
                    {
                        let drag = drag.clone();
                        move |bounds, _, cx| drag.update(cx, |state, _| state.bounds = Some(bounds))
                    },
                    move |bounds, (), window, _| paint_slider(bounds, fraction, paint, window),
                )
                .size_full(),
            )
            .child(FocusRing::new(CornerRadius::Fixed(px(4.0))).visible(control.focus_visible));

        if let (false, Some(on_change)) = (self.disabled, self.on_change) {
            let value_at = move |x: Pixels, bounds: Bounds<Pixels>| {
                let width = bounds.size.width.max(px(1.0));
                let fraction = ((x - bounds.origin.x) / width).clamp(0.0, 1.0);
                range.clamp(range.min + fraction * (range.max - range.min))
            };
            slider = slider
                .on_mouse_down(MouseButton::Left, {
                    let drag = drag.clone();
                    let on_change = on_change.clone();
                    move |event: &MouseDownEvent, window, cx| {
                        let bounds = drag.update(cx, |state, cx| {
                            state.dragging = true;
                            cx.notify();
                            state.bounds
                        });
                        if let Some(bounds) = bounds {
                            on_change(value_at(event.position.x, bounds), window, cx);
                        }
                    }
                })
                .on_key_down({
                    let on_change = on_change.clone();
                    move |event: &KeyDownEvent, window, cx| {
                        if let Some(next) = range.key(value, &event.keystroke.key) {
                            cx.stop_propagation();
                            if next != value {
                                on_change(next, window, cx);
                            }
                        }
                    }
                });
            for action in [Action::Increment, Action::Decrement, Action::SetValue] {
                let on_change = on_change.clone();
                slider = slider.on_a11y_action(action, move |data, window, cx| {
                    if let Some(next) = range.a11y_action(value, action, data)
                        && next != value
                    {
                        on_change(next, window, cx);
                    }
                });
            }
            slider = slider.child(
                canvas(
                    |_, _, _| (),
                    move |_, (), window, _| {
                        let drag_move = drag.clone();
                        let on_change = on_change.clone();
                        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                            let state = drag_move.read(cx);
                            if !phase.bubble() || !state.dragging {
                                return;
                            }
                            if let (true, Some(bounds)) = (event.dragging(), state.bounds) {
                                on_change(value_at(event.position.x, bounds), window, cx);
                            }
                        });
                        window.on_mouse_event(move |_: &MouseUpEvent, phase, _, cx| {
                            if phase.bubble() {
                                drag.update(cx, |state, cx| {
                                    if state.dragging {
                                        state.dragging = false;
                                        cx.notify();
                                    }
                                });
                            }
                        });
                    },
                )
                .size_full(),
            );
        }
        slider
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

#[cfg(test)]
mod tests {
    use super::*;

    fn range(step: Option<f32>) -> SliderRange {
        SliderRange {
            min: 0.0,
            max: 10.0,
            step,
            key_step: step.unwrap_or(0.1),
        }
    }

    #[test]
    fn discrete_values_snap_to_step() {
        let range = range(Some(2.0));
        assert_eq!(range.clamp(3.1), 4.0);
        assert_eq!(range.clamp(-1.0), 0.0);
        assert_eq!(range.clamp(11.0), 10.0);
    }

    #[test]
    fn keys_follow_apg() {
        let range = range(Some(1.0));
        assert_eq!(range.key(5.0, "right"), Some(6.0));
        assert_eq!(range.key(5.0, "down"), Some(4.0));
        assert_eq!(range.key(5.0, "pageup"), Some(10.0));
        assert_eq!(range.key(5.0, "home"), Some(0.0));
        assert_eq!(range.key(5.0, "end"), Some(10.0));
        assert_eq!(range.key(5.0, "a"), None);
    }

    #[test]
    fn max_is_reachable_when_step_does_not_divide_range() {
        let range = SliderRange {
            min: 0.0,
            max: 10.0,
            step: Some(3.0),
            key_step: 3.0,
        };
        assert_eq!(range.key(5.0, "end"), Some(10.0));
        assert_eq!(range.clamp(9.6), 10.0, "最大値のほうが近い");
        assert_eq!(range.clamp(9.4), 9.0);
        assert_eq!(range.key(9.0, "right"), Some(10.0));
        assert_eq!(range.key(10.0, "left"), Some(9.0), "格子の 9 に戻る");
        assert_eq!(range.key(10.0, "pagedown"), Some(0.0));
    }

    #[test]
    fn assistive_actions_change_the_value() {
        let discrete = range(Some(1.0));
        assert_eq!(
            discrete.a11y_action(5.0, Action::Increment, None),
            Some(6.0)
        );
        assert_eq!(
            discrete.a11y_action(5.0, Action::Decrement, None),
            Some(4.0)
        );
        assert_eq!(
            discrete.a11y_action(5.0, Action::SetValue, Some(&ActionData::NumericValue(7.4))),
            Some(7.0)
        );
        assert_eq!(
            discrete.a11y_action(5.0, Action::SetValue, Some(&ActionData::NumericValue(99.0))),
            Some(10.0)
        );
        assert_eq!(discrete.a11y_action(5.0, Action::SetValue, None), None);
        assert_eq!(discrete.a11y_action(5.0, Action::Click, None), None);
        let continuous = range(None);
        assert_eq!(
            continuous.a11y_action(5.0, Action::Increment, None),
            Some(5.1)
        );
    }

    #[test]
    fn reported_value_matches_the_drawn_value() {
        let node = Slider::new("s", 3.1, "評価")
            .range(0.0, 10.0)
            .step(2.0)
            .a11y_node();
        assert_eq!(node.range.map(|r| r.value), Some(4.0));
    }

    #[test]
    fn slider_reports_range() {
        let node = Slider::new("s", 0.5, "音量").a11y_node();
        assert_eq!(node.role, Role::Slider);
        let range = node.range.expect("range");
        assert_eq!(range.value, 0.5);
        assert_eq!(range.max, 1.0);
    }
}
