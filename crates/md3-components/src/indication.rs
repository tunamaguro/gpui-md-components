//! 操作の状態を示す表示 (状態レイヤー、リップル、フォーカスリング)
//!
//! 部品は描画のたびに [`Indication::new`] で操作の状態を読み、headless の要素に [`Indication::track`] で
//! 操作を記録するリスナーを付け、読んだ状態から表示を組み立てる

use std::sync::Arc;
use std::time::Duration;

use gpui::{
    Animation, AnimationExt as _, App, Bounds, Corners, Div, ElementId, Entity, FocusHandle, Hsla,
    IntoElement, MouseButton, ParentElement, PathBuilder, Pixels, Point,
    StatefulInteractiveElement, Styled, Window, canvas, div, point, px,
};
use md3_theme::md3_tokens::focus_ring;
use md3_theme::md3_tokens::motion::CubicBezier;
use md3_theme::md3_tokens::ripple;
use md3_theme::md3_tokens::state;
use md3_theme::{Theme, dp};
use web_time::Instant;

/// 描画 1 回分の、部品の操作の状態
#[derive(Clone)]
pub(crate) struct Indication {
    interaction: Entity<Interaction>,
    focus_handle: FocusHandle,
    enabled: bool,
    /// ポインターが部品の上にあるか
    #[cfg_attr(
        not(any(feature = "button", feature = "checkbox", feature = "switch")),
        allow(dead_code)
    )]
    pub(crate) hovered: bool,
    /// 押されているか
    pub(crate) pressed: bool,
    /// キーボードで操作していて、部品にフォーカスがあるか
    pub(crate) focus_visible: bool,
    state_layer_opacity: f32,
}

impl Indication {
    /// `id` の部品の操作の状態を読む。描画のたびに呼ぶ
    ///
    /// フォーカスと操作の記録は、同じ `id` で描画し続ける間だけ保たれる。
    /// `enabled` が `false` の間は hover と押下を記録せず、記録済みのものも消す
    pub(crate) fn new(id: &ElementId, enabled: bool, window: &mut Window, cx: &mut App) -> Self {
        let id = Arc::new(id.clone());
        let focus_handle = window
            .use_keyed_state(
                ElementId::NamedChild(id.clone(), "md3-focus".into()),
                cx,
                |_, cx| cx.focus_handle(),
            )
            .read(cx)
            .clone();
        let interaction = window.use_keyed_state(
            ElementId::NamedChild(id, "md3-interaction".into()),
            cx,
            |_, _| Interaction {
                hovered: false,
                press: None,
                ripples: Vec::new(),
            },
        );
        let focused = focus_handle.is_focused(window);
        let now = cx.background_executor().now();
        interaction.update(cx, |this, _| {
            if !enabled {
                this.hovered = false;
                this.release_all(now);
            } else if this.press == Some(PressSource::Key) && !focused {
                // キーを離す前にフォーカスが外れると、離したことを受け取れない
                this.release_all(now);
            }
        });
        let focus_visible = focused && window.last_input_was_keyboard();
        let state = interaction.read(cx);
        let pressed = state.press.is_some();
        // 押下中はリップルが押下を示すため、リップルを出さないモーション低減の設定のときだけ押下の値を使う
        let state_layer_opacity = if pressed && cx.reduce_motion() {
            state::PRESSED_STATE_LAYER_OPACITY
        } else if focus_visible {
            state::FOCUS_STATE_LAYER_OPACITY
        } else if state.hovered {
            state::HOVER_STATE_LAYER_OPACITY
        } else {
            0.0
        };
        Self {
            hovered: state.hovered,
            interaction,
            focus_handle,
            enabled,
            pressed,
            focus_visible,
            state_layer_opacity,
        }
    }

    /// 部品のフォーカス。headless の部品の `track_focus` に渡す
    pub(crate) fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    /// `element` に、hover・マウスの左ボタンの押下・`keys` のキーの押下を記録するリスナーを付ける
    ///
    /// `keys` は修飾キーなしで押されたときだけ記録する。無効な部品には何も付けない
    pub(crate) fn track<E: StatefulInteractiveElement>(
        &self,
        element: E,
        keys: &'static [&'static str],
    ) -> E {
        if self.enabled {
            track(element, &self.interaction, keys)
        } else {
            element
        }
    }

    /// `element` の全面に、`layer` の色の状態レイヤーと `ripple` の色の押下の波紋を足す。
    /// どちらも角の半径が `radii` の形の内側に描く。無効な部品には何も足さない
    pub(crate) fn layers<E: ParentElement>(
        &self,
        element: E,
        layer: Hsla,
        ripple: Hsla,
        radii: Corners<Pixels>,
    ) -> E {
        if !self.enabled {
            return element;
        }
        element
            .child(state_layer(layer, self.state_layer_opacity, radii))
            .child(ripple_layer(self.interaction.clone(), ripple, radii))
    }

    /// キーボードのフォーカスがあるときに、`element` の外側に角の半径が `radii` のリングを足す
    pub(crate) fn focus_ring<E: ParentElement>(
        &self,
        element: E,
        theme: &Theme,
        radii: Corners<Pixels>,
    ) -> E {
        if self.focus_visible {
            element.child(focus_ring(theme, radii))
        } else {
            element
        }
    }
}

/// 部品が受けている操作の記録。[`track`] で付けたリスナーが更新して再描画させる
struct Interaction {
    hovered: bool,
    press: Option<PressSource>,
    ripples: Vec<Ripple>,
}

/// 押下を始めた入力
#[derive(Clone, Copy, PartialEq, Eq)]
enum PressSource {
    Pointer,
    Key,
}

/// 押下ごとの波紋
#[derive(Clone, Copy)]
struct Ripple {
    /// 押した位置 (ウィンドウ座標)。キーで押した場合は `None` で、部品の中心から広げる
    origin: Option<Point<Pixels>>,
    start: Instant,
    release: Option<Instant>,
}

impl Ripple {
    /// 消し終えたか。波紋は広がり始めてから [`ripple::MINIMUM_PRESS`] が過ぎるまで消し始めない
    fn is_finished(&self, now: Instant) -> bool {
        self.release.is_some_and(|release| {
            now >= release.max(self.start + ripple::MINIMUM_PRESS) + ripple::PRESSED_FADE_OUT
        })
    }
}

impl Interaction {
    fn press(&mut self, source: PressSource, origin: Option<Point<Pixels>>, cx: &App) {
        let now = cx.background_executor().now();
        self.release_all(now);
        self.ripples.retain(|ripple| !ripple.is_finished(now));
        self.press = Some(source);
        if !cx.reduce_motion() {
            self.ripples.push(Ripple {
                origin,
                start: now,
                release: None,
            });
        }
    }

    fn release(&mut self, source: PressSource, cx: &App) {
        if self.press == Some(source) {
            self.release_all(cx.background_executor().now());
        }
    }

    fn release_all(&mut self, now: Instant) {
        self.press = None;
        for ripple in &mut self.ripples {
            ripple.release.get_or_insert(now);
        }
    }
}

fn track<E: StatefulInteractiveElement>(
    element: E,
    interaction: &Entity<Interaction>,
    keys: &'static [&'static str],
) -> E {
    let on_hover = interaction.clone();
    let on_down = interaction.clone();
    let on_up = interaction.clone();
    let on_up_out = interaction.clone();
    let on_key_down = interaction.clone();
    let on_key_up = interaction.clone();
    element
        .on_hover(move |hovered, _, cx| {
            on_hover.update(cx, |this, cx| {
                this.hovered = *hovered;
                cx.notify();
            });
        })
        .on_mouse_down(MouseButton::Left, move |event, _, cx| {
            on_down.update(cx, |this, cx| {
                this.press(PressSource::Pointer, Some(event.position), cx);
                cx.notify();
            });
        })
        .on_mouse_up(MouseButton::Left, move |_, _, cx| {
            on_up.update(cx, |this, cx| {
                this.release(PressSource::Pointer, cx);
                cx.notify();
            });
        })
        .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
            on_up_out.update(cx, |this, cx| {
                this.release(PressSource::Pointer, cx);
                cx.notify();
            });
        })
        .on_key_down(move |event, _, cx| {
            let stroke = &event.keystroke;
            if event.is_held || stroke.modifiers.modified() || !keys.contains(&stroke.key.as_str())
            {
                return;
            }
            on_key_down.update(cx, |this, cx| {
                this.press(PressSource::Key, None, cx);
                cx.notify();
            });
        })
        .on_key_up(move |event, _, cx| {
            if !keys.contains(&event.keystroke.key.as_str()) {
                return;
            }
            on_key_up.update(cx, |this, cx| {
                this.release(PressSource::Key, cx);
                cx.notify();
            });
        })
}

/// 親の全面を覆う、`color` に `opacity` を掛けた状態レイヤー
fn state_layer(color: Hsla, opacity: f32, radii: Corners<Pixels>) -> Div {
    with_corners(
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .bg(color.opacity(opacity)),
        radii,
    )
}

/// 親の全面に、`interaction` の押下の波紋を `color` で描く。波紋は角の半径が `radii` の形の内側だけに描く
///
/// 波紋の大きさ・動き・時間は Material Web の `md-ripple` に合わせている
fn ripple_layer(
    interaction: Entity<Interaction>,
    color: Hsla,
    radii: Corners<Pixels>,
) -> impl IntoElement {
    canvas(
        |_, _, _| {},
        move |bounds, (), window, cx| {
            let now = cx.background_executor().now();
            let clip = rounded_rect(bounds, radii);
            let mut animating = false;
            for ripple in &interaction.read(cx).ripples {
                animating |= paint_ripple(ripple, bounds, &clip, color, now, window);
            }
            if animating {
                window.request_animation_frame();
            }
        },
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
}

/// 波紋の塗りを、中心から半径のこの割合までは不透明にする (`_ripple.scss` の `radial-gradient` の `65%`)
const RIPPLE_SOLID_RATIO: f32 = 0.65;
/// 波紋の塗りを、半径からこの長さを引いた位置までは不透明にする (`_ripple.scss` の `100% - 70px`)
const RIPPLE_SOLID_INSET: f32 = 70.0;
/// 縁のぼかしを近似する段の数
const RIPPLE_FADE_STEPS: usize = 4;
/// 円を近似する多角形の頂点の数
const CIRCLE_SEGMENTS: usize = 64;

/// 波紋を 1 つ描き、まだ表示が変わり続けるなら `true` を返す
fn paint_ripple(
    ripple: &Ripple,
    bounds: Bounds<Pixels>,
    clip: &[Point<f32>],
    color: Hsla,
    now: Instant,
    window: &mut Window,
) -> bool {
    if ripple.is_finished(now) {
        return false;
    }
    let elapsed = now.saturating_duration_since(ripple.start);
    let fade_out = ripple.release.map_or(1.0, |release| {
        let fade_start = release.max(ripple.start + ripple::MINIMUM_PRESS);
        1.0 - progress(
            now.saturating_duration_since(fade_start),
            ripple::PRESSED_FADE_OUT,
        )
    });
    let opacity = ripple::PRESSED_OPACITY * progress(elapsed, ripple::PRESSED_FADE_IN) * fade_out;
    let grow = ease(
        ripple::PRESS_GROW_EASING,
        progress(elapsed, ripple::PRESS_GROW),
    );

    // 寸法の求め方は ripple.ts の `determineRippleSize` と `getTranslationCoordinates` による
    let width = bounds.size.width.as_f32();
    let height = bounds.size.height.as_f32();
    let max_dim = width.max(height);
    let soft_edge =
        (max_dim * ripple::SOFT_EDGE_CONTAINER_RATIO).max(ripple::SOFT_EDGE_MINIMUM_SIZE.0);
    let initial = (max_dim * ripple::INITIAL_ORIGIN_SCALE).floor().max(1.0);
    let target = width.hypot(height) + ripple::PADDING.0 + soft_edge;
    let radius = lerp(initial, target, grow) / 2.0;
    let end = point(width / 2.0, height / 2.0);
    let start = ripple.origin.map_or(end, |origin| {
        point(
            (origin.x - bounds.origin.x).as_f32(),
            (origin.y - bounds.origin.y).as_f32(),
        )
    });
    let center = point(
        bounds.origin.x.as_f32() + lerp(start.x, end.x, grow),
        bounds.origin.y.as_f32() + lerp(start.y, end.y, grow),
    );
    let initial_radius = initial / 2.0;
    let solid = ((initial_radius - RIPPLE_SOLID_INSET) / initial_radius).max(RIPPLE_SOLID_RATIO);

    // 放射状のグラデーションを描けないため、半径の違う円を外側から重ね、縁から内側へ濃くなる段で近似する。
    // 外側から k 番目までの円が重なる帯の不透明度が opacity × (k + 1) / (段の数 + 1) になるよう、各円の不透明度を決める
    let mut covered = 0.0;
    for step in 0..=RIPPLE_FADE_STEPS {
        let goal = opacity * (step + 1) as f32 / (RIPPLE_FADE_STEPS + 1) as f32;
        let alpha = 1.0 - (1.0 - goal) / (1.0 - covered);
        covered = goal;
        let shrink = (1.0 - solid) * step as f32 / RIPPLE_FADE_STEPS as f32;
        let polygon = clip_convex(&circle(center, radius * (1.0 - shrink)), clip);
        if polygon.len() < 3 {
            continue;
        }
        let mut builder = PathBuilder::fill();
        let points: Vec<_> = polygon.iter().map(|p| point(px(p.x), px(p.y))).collect();
        builder.add_polygon(&points, true);
        if let Ok(path) = builder.build() {
            window.paint_path(path, color.opacity(alpha));
        }
    }
    ripple.release.is_some() || elapsed < ripple::PRESS_GROW
}

/// 角の半径が `radii` の形の外側に、キーボードのフォーカスを示すリングを描く
///
/// 表示し始めに太くなってから戻るアニメーションは、Material Web の `md-focus-ring` に合わせている
fn focus_ring(theme: &Theme, radii: Corners<Pixels>) -> impl IntoElement {
    let color = theme.color(focus_ring::COLOR);
    let offset = dp(focus_ring::OUTWARD_OFFSET);
    let grow = focus_ring::DURATION.mul_f32(focus_ring::GROW_FRACTION);
    let easing = |t| ease(focus_ring::EASING, t);
    div().absolute().border_color(color).with_animations(
        "md3-focus-ring",
        vec![
            Animation::new(grow).with_easing(easing),
            Animation::new(focus_ring::DURATION - grow).with_easing(easing),
        ],
        move |ring, index, t| {
            let active = dp(focus_ring::ACTIVE_WIDTH);
            let width = if index == 0 {
                active * t
            } else {
                active + (dp(focus_ring::WIDTH) - active) * t
            };
            let outset = -(offset + width);
            let ring = ring
                .top(outset)
                .left(outset)
                .right(outset)
                .bottom(outset)
                .border(width);
            with_corners(ring, radii.map(|r| *r + offset + width))
        },
    )
}

/// `element` の 4 つの角の半径を `radii` にする
pub(crate) fn with_corners<E: Styled>(element: E, radii: Corners<Pixels>) -> E {
    element
        .rounded_tl(radii.top_left)
        .rounded_tr(radii.top_right)
        .rounded_br(radii.bottom_right)
        .rounded_bl(radii.bottom_left)
}

/// `from` の半径から `to` の半径へ `t` だけ進めた半径。バネの行き過ぎで負になる場合は 0 にする
#[cfg(any(feature = "button", feature = "icon-button"))]
pub(crate) fn lerp_corners(from: Corners<Pixels>, to: Corners<Pixels>, t: f32) -> Corners<Pixels> {
    let lerp = |a: Pixels, b: Pixels| (a + (b - a) * t).max(px(0.0));
    Corners {
        top_left: lerp(from.top_left, to.top_left),
        top_right: lerp(from.top_right, to.top_right),
        bottom_right: lerp(from.bottom_right, to.bottom_right),
        bottom_left: lerp(from.bottom_left, to.bottom_left),
    }
}

/// `elapsed` が `duration` のどこまで進んだか (0〜1)
fn progress(elapsed: Duration, duration: Duration) -> f32 {
    if duration.is_zero() {
        1.0
    } else {
        (elapsed.as_secs_f32() / duration.as_secs_f32()).min(1.0)
    }
}

fn lerp(from: f32, to: f32, t: f32) -> f32 {
    from + (to - from) * t
}

/// 3 次ベジェ曲線のイージングで、進み具合 `x` (0〜1) に当たる値
pub(crate) fn ease(curve: CubicBezier, x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    let bezier = |t: f32, p1: f32, p2: f32| {
        let u = 1.0 - t;
        3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t
    };
    // x(t) は単調増加なので二分法で t を求める
    let (mut low, mut high) = (0.0_f32, 1.0_f32);
    for _ in 0..24 {
        let mid = (low + high) / 2.0;
        if bezier(mid, curve.x1, curve.x2) < x {
            low = mid;
        } else {
            high = mid;
        }
    }
    bezier((low + high) / 2.0, curve.y1, curve.y2)
}

/// 中心 `center`・半径 `radius` の円を近似する、角度の増える向きに並べた多角形
fn circle(center: Point<f32>, radius: f32) -> Vec<Point<f32>> {
    (0..CIRCLE_SEGMENTS)
        .map(|i| {
            let angle = std::f32::consts::TAU * i as f32 / CIRCLE_SEGMENTS as f32;
            point(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            )
        })
        .collect()
}

/// `bounds` の角を `radii` で丸めた形を近似する、角度の増える向きに並べた多角形
fn rounded_rect(bounds: Bounds<Pixels>, radii: Corners<Pixels>) -> Vec<Point<f32>> {
    const ARC_SEGMENTS: usize = 8;
    let left = bounds.origin.x.as_f32();
    let top = bounds.origin.y.as_f32();
    let right = left + bounds.size.width.as_f32();
    let bottom = top + bounds.size.height.as_f32();
    let limit = (right - left).min(bottom - top) / 2.0;
    let radius = |r: Pixels| r.as_f32().clamp(0.0, limit);
    // 角ごとの円弧の中心・半径・始めの角度 (y 軸は下向き)
    let corners = [
        (left, top, radius(radii.top_left), 1.0),
        (right, top, radius(radii.top_right), 1.5),
        (right, bottom, radius(radii.bottom_right), 0.0),
        (left, bottom, radius(radii.bottom_left), 0.5),
    ];
    let mut points: Vec<Point<f32>> = Vec::new();
    for (x, y, r, start) in corners {
        let cx = if x == left { x + r } else { x - r };
        let cy = if y == top { y + r } else { y - r };
        for i in 0..=ARC_SEGMENTS {
            let angle = std::f32::consts::PI * (start + 0.5 * i as f32 / ARC_SEGMENTS as f32);
            let p = point(cx + r * angle.cos(), cy + r * angle.sin());
            if points
                .last()
                .is_none_or(|last| (last.x - p.x).hypot(last.y - p.y) > 1e-3)
            {
                points.push(p);
            }
        }
    }
    points
}

/// 凸多角形 `subject` のうち、凸多角形 `clip` の内側の部分 (Sutherland–Hodgman 法)。
/// どちらも同じ向きに頂点を並べること
fn clip_convex(subject: &[Point<f32>], clip: &[Point<f32>]) -> Vec<Point<f32>> {
    let cross = |a: Point<f32>, b: Point<f32>, p: Point<f32>| {
        (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x)
    };
    let mut output = subject.to_vec();
    for (i, &a) in clip.iter().enumerate() {
        let b = clip[(i + 1) % clip.len()];
        let input = std::mem::take(&mut output);
        for (j, &p) in input.iter().enumerate() {
            let q = input[(j + 1) % input.len()];
            let (dp, dq) = (cross(a, b, p), cross(a, b, q));
            if dp >= 0.0 {
                output.push(p);
            }
            if (dp >= 0.0) != (dq >= 0.0) {
                let t = dp / (dp - dq);
                output.push(point(p.x + (q.x - p.x) * t, p.y + (q.y - p.y) * t));
            }
        }
        if output.is_empty() {
            break;
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use gpui::size;

    use super::*;

    #[test]
    fn イージングは両端を通り制御点の形に沿う() {
        let standard = ripple::PRESS_GROW_EASING;
        assert!(ease(standard, 0.0).abs() < 1e-4);
        assert!((ease(standard, 1.0) - 1.0).abs() < 1e-4);
        // cubic-bezier(0.2, 0, 0, 1) の x = 0.5 は、t ≈ 0.7817 で y ≈ 0.8778 (Python で独立に解いた値)
        assert!((ease(standard, 0.5) - 0.8778).abs() < 1e-3);
    }

    #[test]
    fn 円を角丸の矩形で切り取ると矩形の内側に収まる() {
        let bounds = Bounds::new(point(px(10.0), px(20.0)), size(px(100.0), px(40.0)));
        let clip = rounded_rect(bounds, Corners::all(px(20.0)));
        let clipped = clip_convex(&circle(point(10.0, 20.0), 30.0), &clip);
        assert!(clipped.len() >= 3);
        for p in &clipped {
            assert!(p.x >= 10.0 - 1e-3 && p.x <= 110.0 + 1e-3, "{p:?}");
            assert!(p.y >= 20.0 - 1e-3 && p.y <= 60.0 + 1e-3, "{p:?}");
            // 左上の角の丸みの外側には出ない
            let (dx, dy) = (p.x - 30.0, p.y - 40.0);
            if p.x < 30.0 && p.y < 40.0 {
                assert!(dx.hypot(dy) <= 20.0 + 1e-2, "{p:?}");
            }
        }
    }

    #[test]
    fn 形を覆う円は形そのものになる() {
        let bounds = Bounds::new(point(px(0.0), px(0.0)), size(px(40.0), px(40.0)));
        let clip = rounded_rect(bounds, Corners::all(px(0.0)));
        let clipped = clip_convex(&circle(point(20.0, 20.0), 100.0), &clip);
        let area = clipped
            .iter()
            .zip(clipped.iter().cycle().skip(1))
            .map(|(a, b)| a.x * b.y - b.x * a.y)
            .sum::<f32>()
            .abs()
            / 2.0;
        assert!((area - 1600.0).abs() < 1.0, "{area}");
    }
}
