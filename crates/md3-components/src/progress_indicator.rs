// 寸法・色・描く位置の計算・不確定のときの動きは Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ProgressIndicator.kt
//         (`LinearProgressIndicator`、`CircularProgressIndicator`、`drawLinearIndicator`、
//         `drawCircularIndicator`、`ProgressIndicatorDefaults`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2022 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 の進捗インジケーター

use std::f32::consts::PI;
use std::time::Duration;

use gpui::{
    Animation, AnimationExt, App, Bounds, Corners, ElementId, FillOptions, FillRule, Hsla,
    IntoElement, ParentElement, PathBuilder, PathStyle, Pixels, Point, Refineable, RenderOnce,
    SharedString, StatefulInteractiveElement, StyleRefinement, Styled, Window, canvas, div, fill,
    point, prelude::FluentBuilder, px,
};
use gpui_headless::progress::{Progress as HeadlessProgress, ProgressBar};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::{
    circular_progress_indicator, linear_progress_indicator, progress_indicator,
};
use md3_theme::md3_tokens::supplement::progress_indicator as supplement;
use md3_theme::{SharedTheme, ThemeExt, dp, easing};

/// 進捗インジケーターが表す進み具合
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Progress {
    /// 進み具合が分からない。線が動き続け、モーション低減の設定では止まる
    Indeterminate,
    /// 全体のうち進んだ割合。0.0 で未着手、1.0 で完了
    ///
    /// 0.0〜1.0 の外の値はその範囲に収め、NaN は 0.0 として扱う
    Determinate(f32),
}

impl From<Progress> for HeadlessProgress {
    fn from(value: Progress) -> Self {
        match value {
            Progress::Indeterminate => HeadlessProgress::Indeterminate,
            Progress::Determinate(fraction) => HeadlessProgress::Determinate(fraction),
        }
    }
}

/// 進捗インジケーターの形。既定の大きさと、進み具合を描く向きが決まる
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ProgressIndicatorVariant {
    /// 左から右へ伸びる直線。既定の大きさは 240×4dp で、線の太さは高さになる
    #[default]
    Linear,
    /// 12 時の位置から時計回りに伸びる円。既定の大きさは 40×40dp で、線の太さは 4dp
    Circular,
}

/// 進捗インジケーターの色の上書き。`None` の項目は既定の色のまま
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ProgressIndicatorColors {
    /// 進んだ部分と、直線の終端の点の色
    pub indicator: Option<ColorRole>,
    /// 進んでいない部分の色
    ///
    /// 既定では、不確定の円はこの部分を描かず、それ以外は [`ColorRole::SecondaryContainer`] で描く。
    /// 指定すると、不確定の円でもこの色で描く
    pub track: Option<ColorRole>,
}

/// MD3 の進捗インジケーター
///
/// 処理の進み具合か、終わるまで待つ必要があることを示す。
/// 支援技術には role `progressbar` として公開され、進み具合が分かるときは割合を 0〜100 の値として公開する。
/// フォーカスも操作も持たない。アクセシブルな名前は [`ProgressIndicator::aria_label`] で与えること
///
/// [`Styled`] で指定したスタイルはインジケーターの箱に付き、形で決まる既定の大きさより優先する。
/// 直線は幅で長さを、高さで線の太さを変える。円は幅と高さを同じにすること
///
/// 進み具合が分からないときの動きは、モーション低減の設定 ([`App::reduce_motion`]) では止まり、
/// 線の位置が変わらない 1 枚の絵になる
#[derive(IntoElement)]
pub struct ProgressIndicator {
    id: ElementId,
    progress: Progress,
    variant: ProgressIndicatorVariant,
    colors: ProgressIndicatorColors,
    style: StyleRefinement,
    aria_label: Option<SharedString>,
    theme: Option<SharedTheme>,
}

impl ProgressIndicator {
    /// `progress` を表す [`ProgressIndicatorVariant::Linear`] の進捗インジケーターを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。進み具合が分からないときの動きは `id` ごとに続く
    pub fn new(id: impl Into<ElementId>, progress: Progress) -> Self {
        Self {
            id: id.into(),
            progress,
            variant: ProgressIndicatorVariant::default(),
            colors: ProgressIndicatorColors::default(),
            style: StyleRefinement::default(),
            aria_label: None,
            theme: None,
        }
    }

    /// 形を設定する
    pub fn variant(mut self, variant: ProgressIndicatorVariant) -> Self {
        self.variant = variant;
        self
    }

    /// 既定の色を `colors` で上書きする
    pub fn colors(mut self, colors: ProgressIndicatorColors) -> Self {
        self.colors = colors;
        self
    }

    /// `label` をアクセシブルな名前にする
    ///
    /// 何の進み具合かを表す名前を与えること
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl RenderOnce for ProgressIndicator {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let track = match (self.colors.track, self.variant, self.progress) {
            (Some(role), _, _) => Some(role),
            // `circularIndeterminateTrackColor` は透明
            (None, ProgressIndicatorVariant::Circular, Progress::Indeterminate) => None,
            (None, _, _) => Some(progress_indicator::TRACK_COLOR),
        };
        let painter = Painter {
            variant: self.variant,
            indicator: theme.color(
                self.colors
                    .indicator
                    .unwrap_or(progress_indicator::ACTIVE_INDICATOR_COLOR),
            ),
            track: track.map(|role| theme.color(role)),
        };
        let reduce_motion = cx.reduce_motion();

        let mut bar = ProgressBar::new(self.id, self.progress.into())
            .relative()
            .flex_none()
            .when_some(self.aria_label, |this, label| this.aria_label(label));
        bar = match self.variant {
            ProgressIndicatorVariant::Linear => bar
                .w(dp(supplement::LINEAR_WIDTH))
                .h(dp(linear_progress_indicator::HEIGHT)),
            ProgressIndicatorVariant::Circular => bar.size(dp(circular_progress_indicator::SIZE)),
        };
        bar.style().refine(&self.style);

        match HeadlessProgress::from(self.progress).fraction() {
            Some(fraction) => bar.child(painter.canvas(Frame::Determinate(fraction))),
            None if reduce_motion => {
                bar.child(painter.canvas(Frame::Indeterminate(self.variant.static_time())))
            }
            None => {
                let cycle = self.variant.indeterminate_cycle();
                bar.child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .with_animation(
                            "progress-indicator-motion",
                            Animation::new(cycle).repeat(),
                            move |this, delta| {
                                this.child(
                                    painter.canvas(Frame::Indeterminate(cycle.mul_f32(delta))),
                                )
                            },
                        ),
                )
            }
        }
    }
}

impl Styled for ProgressIndicator {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ProgressIndicatorVariant {
    /// 不確定のときの動きの 1 周期
    fn indeterminate_cycle(self) -> Duration {
        match self {
            Self::Linear => supplement::LINEAR_INDETERMINATE_DURATION,
            Self::Circular => supplement::CIRCULAR_INDETERMINATE_DURATION,
        }
    }

    /// モーション低減の設定で、不確定のときに描く周期の中の時刻
    ///
    /// 周期の始まりは直線が線を描かないため、1 本目の線が中ほどにある時刻にする。円は弧が見える始まりにする
    fn static_time(self) -> Duration {
        match self {
            Self::Linear => supplement::LINEAR_INDETERMINATE_DURATION / 2,
            Self::Circular => Duration::ZERO,
        }
    }
}

/// 1 回の描画で描く状態
#[derive(Clone, Copy, Debug, PartialEq)]
enum Frame {
    /// 0.0〜1.0 に収めた進んだ割合
    Determinate(f32),
    /// 不確定の動きの周期の始まりからの経過時間
    Indeterminate(Duration),
}

/// 描く部分
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Part {
    Indicator,
    Track,
}

/// 直線の 1 本の線。両端は長さに対する割合で、`start` と `end` の大小は問わない
#[derive(Clone, Copy, Debug, PartialEq)]
struct Bar {
    start: f32,
    end: f32,
    part: Part,
}

/// 円の 1 本の弧。角度は度で、3 時の位置を 0 として時計回りに増える
#[derive(Clone, Copy, Debug, PartialEq)]
struct Arc {
    start: f32,
    sweep: f32,
    part: Part,
}

/// `delay` から `duration` の間に `ease` で 0 から 1 へ進み、その前後は 0 と 1 にとどまる値
///
/// Compose の `keyframes` で、`0f at delay using ease` と `1f at delay + duration` を指定した値
fn keyframe(time: Duration, delay: Duration, duration: Duration, ease: impl Fn(f32) -> f32) -> f32 {
    if time <= delay {
        0.0
    } else if time >= delay + duration {
        1.0
    } else {
        ease((time - delay).as_secs_f32() / duration.as_secs_f32())
    }
}

/// 長さ `width`、太さ `height` の直線で描く線。丸い端の分の間隔も含めて Compose と同じ位置にする
fn linear_bars(width: f32, height: f32, frame: Frame) -> Vec<Bar> {
    // 丸い端は線の外へ太さの半分ずつはみ出すため、線どうしの間隔に太さを足す。
    // 太さが長さを超えると Compose は角の端で描く (`drawLinearIndicator`) ため、足さない
    let gap = dp_value(linear_progress_indicator::TRACK_ACTIVE_SPACE)
        + if height > width { 0.0 } else { height };
    let gap = gap / width;
    let bar = |start: f32, end: f32, part: Part| Bar { start, end, part };
    match frame {
        Frame::Determinate(progress) => {
            let mut bars = Vec::new();
            let track_start = progress + progress.min(gap);
            if track_start <= 1.0 {
                bars.push(bar(track_start, 1.0, Part::Track));
            }
            bars.push(bar(0.0, progress, Part::Indicator));
            bars
        }
        Frame::Indeterminate(time) => {
            let ease = easing(supplement::LINEAR_INDETERMINATE_EASING);
            let first_head = keyframe(
                time,
                supplement::LINEAR_FIRST_LINE_HEAD_DELAY,
                supplement::LINEAR_FIRST_LINE_HEAD_DURATION,
                ease,
            );
            let first_tail = keyframe(
                time,
                supplement::LINEAR_FIRST_LINE_TAIL_DELAY,
                supplement::LINEAR_FIRST_LINE_TAIL_DURATION,
                ease,
            );
            let second_head = keyframe(
                time,
                supplement::LINEAR_SECOND_LINE_HEAD_DELAY,
                supplement::LINEAR_SECOND_LINE_HEAD_DURATION,
                ease,
            );
            let second_tail = keyframe(
                time,
                supplement::LINEAR_SECOND_LINE_TAIL_DELAY,
                supplement::LINEAR_SECOND_LINE_TAIL_DURATION,
                ease,
            );

            let mut bars = Vec::new();
            // 1 本目より先の線路
            if first_head < 1.0 - gap {
                let start = if first_head > 0.0 {
                    first_head + gap
                } else {
                    0.0
                };
                bars.push(bar(start, 1.0, Part::Track));
            }
            if first_head - first_tail > 0.0 {
                bars.push(bar(first_head, first_tail, Part::Indicator));
            }
            // 1 本目と 2 本目の間の線路
            if first_tail > gap {
                let start = if second_head > 0.0 {
                    second_head + gap
                } else {
                    0.0
                };
                let end = if first_tail < 1.0 {
                    first_tail - gap
                } else {
                    1.0
                };
                bars.push(bar(start, end, Part::Track));
            }
            if second_head - second_tail > 0.0 {
                bars.push(bar(second_head, second_tail, Part::Indicator));
            }
            // 2 本目より後の線路
            if second_tail > gap {
                let end = if second_tail < 1.0 {
                    second_tail - gap
                } else {
                    1.0
                };
                bars.push(bar(0.0, end, Part::Track));
            }
            bars
        }
    }
}

/// 直径 `diameter`、線の太さ `stroke` の円で描く弧。丸い端の分の間隔も含めて Compose と同じ角度にする
fn circular_arcs(diameter: f32, stroke: f32, frame: Frame) -> Vec<Arc> {
    // 直線と同じく、丸い端の分だけ太さを足す
    let gap = dp_value(circular_progress_indicator::TRACK_ACTIVE_SPACE) + stroke;
    let gap_sweep = gap / (PI * diameter) * 360.0;
    // 線路は進んだ部分の両側に間隔を空けた残り。進んだ部分が一周に近いと角度が負になり、
    // Compose (Skia の drawArc) は始まりから反時計回りに描くため、同じ範囲を時計回りの弧にして描く
    let arcs = |start: f32, sweep: f32| {
        let gap = sweep.min(gap_sweep);
        let track_start = start + sweep + gap;
        let track_sweep = 360.0 - sweep - gap * 2.0;
        let track = Arc {
            start: track_start + track_sweep.min(0.0),
            sweep: track_sweep.abs(),
            part: Part::Track,
        };
        let indicator = Arc {
            start,
            sweep,
            part: Part::Indicator,
        };
        [track, indicator]
            .into_iter()
            .filter(|arc| arc.sweep > 0.0)
            .collect()
    };
    match frame {
        // 12 時の位置から描く
        Frame::Determinate(progress) => arcs(270.0, progress * 360.0),
        Frame::Indeterminate(time) => {
            let cycle = supplement::CIRCULAR_INDETERMINATE_DURATION;
            let global = supplement::CIRCULAR_GLOBAL_ROTATION_DEGREES * time.as_secs_f32()
                / cycle.as_secs_f32();
            // 追加の回転は、区間ごとに 90 度ずつ線形に回り、次の区間まで止まる。
            // Compose は最初に回り終わる 300 ms のキーフレームに EmphasizedDecelerate のイージングを指定しているが、
            // `using` はその時刻から始まる止まっている区間に効き、回る区間は既定の線形になる
            let quarter = supplement::CIRCULAR_ADDITIONAL_ROTATION_DEGREES / 4.0;
            let interval = supplement::CIRCULAR_ADDITIONAL_ROTATION_INTERVAL;
            let step = (time.as_secs_f32() / interval.as_secs_f32())
                .floor()
                .min(3.0);
            let moving = (time.as_secs_f32() - interval.as_secs_f32() * step)
                / supplement::CIRCULAR_ADDITIONAL_ROTATION_DURATION.as_secs_f32();
            let additional = quarter * (step + moving.min(1.0));
            // 弧の長さは周期の前半に線形に伸び、後半にイージングで縮む
            let (min, max) = (
                supplement::CIRCULAR_INDETERMINATE_MIN_PROGRESS,
                supplement::CIRCULAR_INDETERMINATE_MAX_PROGRESS,
            );
            let half = cycle.as_secs_f32() / 2.0;
            let t = time.as_secs_f32();
            let progress = if t < half {
                min + (max - min) * t / half
            } else {
                max - (max - min)
                    * easing(supplement::CIRCULAR_INDETERMINATE_EASING)((t - half) / half)
            };
            arcs(global + additional, progress * 360.0)
        }
    }
}

fn dp_value(value: md3_theme::md3_tokens::Dp) -> f32 {
    f32::from(dp(value))
}

/// 進捗インジケーターを描く色と形
#[derive(Clone, Copy)]
struct Painter {
    variant: ProgressIndicatorVariant,
    indicator: Hsla,
    track: Option<Hsla>,
}

impl Painter {
    fn color(&self, part: Part) -> Option<Hsla> {
        match part {
            Part::Indicator => Some(self.indicator),
            Part::Track => self.track,
        }
    }

    /// 親いっぱいに広がり、`frame` の状態を描く要素
    fn canvas(self, frame: Frame) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, (), window, _| match self.variant {
                ProgressIndicatorVariant::Linear => self.paint_linear(bounds, frame, window),
                ProgressIndicatorVariant::Circular => self.paint_circular(bounds, frame, window),
            },
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full()
    }

    fn paint_linear(self, bounds: Bounds<Pixels>, frame: Frame, window: &mut Window) {
        let width = f32::from(bounds.size.width);
        let height = f32::from(bounds.size.height);
        let round = height <= width;
        for bar in linear_bars(width, height, frame) {
            let Some(color) = self.color(bar.part) else {
                continue;
            };
            let (left, right) = if round {
                // 丸い端が要素からはみ出さないよう、線の中心を内側に収める (`drawLinearIndicator`)
                if bar.start == bar.end {
                    continue;
                }
                let cap = height / 2.0;
                let clamp = |fraction: f32| (fraction * width).clamp(cap, width - cap);
                let (a, b) = (clamp(bar.start), clamp(bar.end));
                (a.min(b) - cap, a.max(b) + cap)
            } else {
                let (a, b) = (bar.start * width, bar.end * width);
                (a.min(b), a.max(b))
            };
            let quad = fill(
                Bounds::new(
                    bounds.origin + point(px(left), px(0.0)),
                    gpui::size(px(right - left), bounds.size.height),
                ),
                color,
            );
            window.paint_quad(if round {
                quad.corner_radii(Corners::all(px(height / 2.0)))
            } else {
                quad
            });
        }

        if let Frame::Determinate(_) = frame {
            // 終端の点 (`ProgressIndicatorDefaults.drawStopIndicator`)。線より大きくしない。
            // 形は線端の指定で決まり、線が角の端になる縦横比でも円のまま
            let stop = dp_value(linear_progress_indicator::STOP_SIZE).min(height);
            let offset = ((height - stop) / 2.0).min(dp_value(supplement::STOP_TRAILING_SPACE_MAX));
            window.paint_quad(
                fill(
                    Bounds::new(
                        bounds.origin + point(px(width - stop - offset), px((height - stop) / 2.0)),
                        gpui::size(px(stop), px(stop)),
                    ),
                    self.indicator,
                )
                .corner_radii(Corners::all(px(stop / 2.0))),
            );
        }
    }

    fn paint_circular(self, bounds: Bounds<Pixels>, frame: Frame, window: &mut Window) {
        let diameter = f32::from(bounds.size.width);
        let stroke = dp_value(circular_progress_indicator::TRACK_THICKNESS);
        let center = bounds.center();
        // 線の中心を通る円の半径。線が要素の内側に収まるよう太さの半分だけ小さくする (`drawCircularIndicator`)
        let radius = (diameter - stroke) / 2.0;
        for arc in circular_arcs(diameter, stroke, frame) {
            let Some(color) = self.color(arc.part) else {
                continue;
            };
            if let Some(path) = arc_path(center, radius, stroke, arc.start, arc.sweep) {
                window.paint_path(path, color);
            }
        }
    }
}

/// 中心 `center`、半径 `radius` の円の上を `start` 度から時計回りに `sweep` 度だけ進む、太さ `stroke` の丸い端の弧
///
/// `sweep` は正の値にすること。一周以上なら端のない輪にする
fn arc_path(
    center: Point<Pixels>,
    radius: f32,
    stroke: f32,
    start: f32,
    sweep: f32,
) -> Option<gpui::Path<Pixels>> {
    let at = |angle: f32, r: f32| {
        let (sin, cos) = angle.sin_cos();
        center + point(px(r * cos), px(r * sin))
    };
    let (outer, inner) = (radius + stroke / 2.0, radius - stroke / 2.0);
    // 弧は 2 度ごと、端の半円は 15 度ごとの折れ線で近似する
    let arc_points = |from: f32, sweep: f32, r: f32| {
        let steps = (sweep.abs() / 2.0).ceil().max(1.0) as usize;
        (0..=steps).map(move |i| at((from + sweep * i as f32 / steps as f32).to_radians(), r))
    };
    // 回り込んで重なった部分も塗るよう、向きの和で塗る
    let mut builder = PathBuilder::fill().with_style(PathStyle::Fill(
        FillOptions::default().with_fill_rule(FillRule::NonZero),
    ));
    if sweep >= 360.0 {
        builder.add_polygon(&arc_points(0.0, 360.0, outer).collect::<Vec<_>>(), true);
        builder.add_polygon(&arc_points(0.0, -360.0, inner).collect::<Vec<_>>(), true);
    } else {
        let end = start + sweep;
        let cap = |angle: f32, from: f32| {
            let middle = at(angle.to_radians(), radius);
            let (sin, cos) = angle.to_radians().sin_cos();
            (0..=12).map(move |i| {
                // 半径の向き (0 度) と進む向き (90 度) を軸に、`from` 度から 180 度の半円を描く。
                // 終わりの端は外側から進む向きを回って内側へ、始まりの端は内側から戻る向きを回って外側へ描く
                let theta = (from + 180.0 * i as f32 / 12.0).to_radians();
                let (radial, tangent) = (theta.cos() * stroke / 2.0, theta.sin() * stroke / 2.0);
                middle
                    + point(
                        px(radial * cos - tangent * sin),
                        px(radial * sin + tangent * cos),
                    )
            })
        };
        let points: Vec<_> = arc_points(start, sweep, outer)
            .chain(cap(end, 0.0))
            .chain(arc_points(end, -sweep, inner))
            .chain(cap(start, 180.0))
            .collect();
        builder.add_polygon(&points, true);
    }
    builder.build().ok()
}

#[cfg(test)]
mod tests {
    use gpui::{AnyElement, Context, Render, TestAppContext};

    use super::*;

    struct ElementView(Box<dyn Fn() -> AnyElement>);

    impl Render for ElementView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            (self.0)()
        }
    }

    /// 矩形の論理ピクセルの x, y, 幅, 高さと塗り、左上の角の半径
    type Quad = ((f32, f32, f32, f32), gpui::Background, f32);

    /// `element` を描いたときの矩形と、進んだ部分・線路の色
    fn painted_rects(
        cx: &mut TestAppContext,
        element: fn() -> ProgressIndicator,
    ) -> (Vec<Quad>, Hsla, Hsla) {
        let (_, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            ElementView(Box::new(move || div().child(element()).into_any_element()))
        });
        cx.run_until_parked();
        cx.update(|window, cx| {
            let scale = window.scale_factor();
            let rects = window
                .painted_quads()
                .into_iter()
                .map(|quad| {
                    let b = quad.bounds;
                    (
                        (
                            b.origin.x.0 / scale,
                            b.origin.y.0 / scale,
                            b.size.width.0 / scale,
                            b.size.height.0 / scale,
                        ),
                        quad.background,
                        quad.corner_radii.top_left.0 / scale,
                    )
                })
                .collect();
            let theme = cx.md3_theme();
            (
                rects,
                theme.color(ColorRole::Primary),
                theme.color(ColorRole::SecondaryContainer),
            )
        })
    }

    #[gpui::test]
    fn 決定的な直線は丸い端を内側に収めて描く(cx: &mut TestAppContext) {
        let (rects, primary, track) = painted_rects(cx, || {
            ProgressIndicator::new("progress", Progress::Determinate(0.5))
        });
        // 線路は 128〜240 の線の中心を 2〜238 に収めて端を足す。進んだ部分は 0〜120 を 2〜120 に収める。
        // 終端の点は 4dp で、線と同じ太さのため端との間隔は 0
        assert_eq!(
            rects,
            vec![
                ((126.0, 0.0, 114.0, 4.0), track.into(), 2.0),
                ((0.0, 0.0, 122.0, 4.0), primary.into(), 2.0),
                ((236.0, 0.0, 4.0, 4.0), primary.into(), 2.0)
            ]
        );
    }

    #[gpui::test]
    fn 指定した大きさは形の既定より優先する(cx: &mut TestAppContext) {
        let (rects, _, _) = painted_rects(cx, || {
            ProgressIndicator::new("progress", Progress::Determinate(1.0))
                .w(px(100.0))
                .h(px(8.0))
        });
        let rects: Vec<_> = rects.into_iter().map(|(rect, _, _)| rect).collect();
        // 終端の点は 4dp のまま、線の中に (8 - 4) / 2 = 2 の間隔を空けて置く
        assert_eq!(rects, vec![(0.0, 0.0, 100.0, 8.0), (94.0, 2.0, 4.0, 4.0)]);
    }

    #[gpui::test]
    fn 高さが幅を超えると線は角の端になり_終端の点は円のまま(
        cx: &mut TestAppContext,
    ) {
        let (rects, _, _) = painted_rects(cx, || {
            ProgressIndicator::new("progress", Progress::Determinate(1.0))
                .w(px(16.0))
                .h(px(24.0))
        });
        let rects: Vec<_> = rects
            .into_iter()
            .map(|(rect, _, radius)| (rect, radius))
            .collect();
        // 線路の始まりは 1 + 4 / 16 で、端を超えるため描かない。
        // 終端の点は端との間隔が上限の 6dp になり、16 - 4 - 6 = 6 に置く
        assert_eq!(
            rects,
            vec![((0.0, 0.0, 16.0, 24.0), 0.0), ((6.0, 10.0, 4.0, 4.0), 2.0)]
        );
    }

    #[gpui::test]
    fn モーション低減では不確定の直線が線を描いた状態で止まる(
        cx: &mut TestAppContext,
    ) {
        cx.update(|cx| cx.set_reduce_motion(true));
        let (rects, primary, _) = painted_rects(cx, || {
            ProgressIndicator::new("progress", Progress::Indeterminate)
        });
        assert!(
            rects.iter().any(|(_, color, _)| *color == primary.into()),
            "進んだ部分を描くこと: {rects:?}"
        );
    }

    /// 期待値は、12 時から 3 時までの線の中心 (半径 18) に、太さ 4 の丸い端を足した範囲から手で求めた。
    /// 始まりの端は (20, 2) を中心に左へ、終わりの端は (38, 20) を中心に下へ 2 だけはみ出す
    #[test]
    fn 弧のパスは丸い端を含む範囲を塗る() {
        let center = point(px(20.0), px(20.0));
        let bounds = |path: gpui::Path<Pixels>| {
            let b = path.bounds;
            (
                f32::from(b.origin.x),
                f32::from(b.origin.y),
                f32::from(b.size.width),
                f32::from(b.size.height),
            )
        };
        let close = |actual: (f32, f32, f32, f32), expected: (f32, f32, f32, f32)| {
            let pairs = [
                (actual.0, expected.0),
                (actual.1, expected.1),
                (actual.2, expected.2),
                (actual.3, expected.3),
            ];
            assert!(
                pairs.iter().all(|(a, e)| (a - e).abs() < 1e-3),
                "{actual:?} は {expected:?} に近いこと"
            );
        };
        let quarter = arc_path(center, 18.0, 4.0, 270.0, 90.0).expect("パスを作れること");
        close(bounds(quarter), (18.0, 0.0, 22.0, 22.0));

        let ring = arc_path(center, 18.0, 4.0, 270.0, 360.0).expect("パスを作れること");
        // 輪は内側を塗らない。塗る面積は π × (20² - 16²) ≈ 452.39 で、内側まで塗ると π × 20² ≈ 1256.6 になる。
        // 頂点は 3 つずつ三角形を成す
        let area: f32 = ring
            .vertices
            .chunks(3)
            .map(|triangle| {
                let p = |i: usize| {
                    let v = triangle[i].xy_position;
                    (f32::from(v.x), f32::from(v.y))
                };
                let ((ax, ay), (bx, by), (cx, cy)) = (p(0), p(1), p(2));
                ((bx - ax) * (cy - ay) - (cx - ax) * (by - ay)).abs() / 2.0
            })
            .sum();
        assert!((area - 452.39).abs() < 1.0, "輪の面積: {area}");
        close(bounds(ring), (0.0, 0.0, 40.0, 40.0));
    }

    fn ms(value: u64) -> Duration {
        Duration::from_millis(value)
    }

    fn assert_close(actual: f32, expected: f32, message: &str) {
        assert!(
            (actual - expected).abs() < 1e-4,
            "{message}: {actual} は {expected} に近いこと"
        );
    }

    /// 期待値は Compose の `LinearProgressIndicator` (決定的) の式に、240×4dp と
    /// `TrackActiveSpace` 4dp を手で代入して求めた。間隔の割合は (4 + 4) / 240
    #[test]
    fn 決定的な直線は進んだ部分と間隔を空けた線路を描く() {
        let gap = 8.0 / 240.0;
        let bars = linear_bars(240.0, 4.0, Frame::Determinate(0.5));
        assert_eq!(bars.len(), 2);
        assert_close(bars[0].start, 0.5 + gap, "線路の始まり");
        assert_eq!((bars[0].end, bars[0].part), (1.0, Part::Track));
        assert_eq!(
            (bars[1].start, bars[1].end, bars[1].part),
            (0.0, 0.5, Part::Indicator)
        );

        // 進み具合が間隔より小さいときは、間隔も進み具合に合わせて縮む
        let bars = linear_bars(240.0, 4.0, Frame::Determinate(0.01));
        assert_close(bars[0].start, 0.02, "小さい進み具合の線路の始まり");

        // 完了では線路を描かない
        let bars = linear_bars(240.0, 4.0, Frame::Determinate(1.0));
        assert_eq!(
            bars,
            vec![Bar {
                start: 0.0,
                end: 1.0,
                part: Part::Indicator
            }]
        );
    }

    /// 期待値は Compose の `keyframes` の定義 (イージングはその時刻から始まる区間に効き、既定は線形) と、
    /// `EasingEmphasizedAccelerateCubicBezier` (0.3, 0, 0.8, 0.15) を Python で二分法により評価した値から求めた
    #[test]
    fn 不確定の直線は周期の中の時刻で線の端が決まる() {
        // 500ms: 1 本目の先端は時間の 0.5、末端は 0.25 まで進み、2 本目は動いていない
        let bars = linear_bars(240.0, 4.0, Frame::Indeterminate(ms(500)));
        assert_eq!(bars.len(), 3, "{bars:?}");
        assert_close(bars[0].start, 0.187_331_5, "1 本目より先の線路");
        assert_close(bars[1].start, 0.153_998_2, "1 本目の先端");
        assert_close(bars[1].end, 0.035_342_6, "1 本目の末端");
        assert_close(
            bars[2].end,
            0.002_009_3,
            "1 本目と 2 本目の間の線路の終わり",
        );
        assert_eq!(bars[2].start, 0.0);

        // 周期の始まりは線を描かず、線路だけが全体に伸びる
        let bars = linear_bars(240.0, 4.0, Frame::Indeterminate(Duration::ZERO));
        assert_eq!(
            bars,
            vec![Bar {
                start: 0.0,
                end: 1.0,
                part: Part::Track
            }]
        );
    }

    /// 期待値は Compose の `CircularProgressIndicator` (決定的) の式に、直径 40dp、太さ 4dp、
    /// `TrackActiveSpace` 4dp を手で代入して求めた。間隔の角度は (4 + 4) / (π × 40) × 360
    #[test]
    fn 決定的な円は_12_時から時計回りに描く() {
        let gap = 8.0 / (PI * 40.0) * 360.0;
        let arcs = circular_arcs(40.0, 4.0, Frame::Determinate(0.25));
        assert_eq!(arcs.len(), 2);
        assert_close(arcs[0].start, 270.0 + 90.0 + gap, "線路の始まり");
        assert_close(arcs[0].sweep, 360.0 - 90.0 - gap * 2.0, "線路の角度");
        assert_eq!(
            arcs[1],
            Arc {
                start: 270.0,
                sweep: 90.0,
                part: Part::Indicator
            }
        );

        // 未着手は線路だけの輪になる
        let arcs = circular_arcs(40.0, 4.0, Frame::Determinate(0.0));
        assert_eq!(
            arcs,
            vec![Arc {
                start: 270.0,
                sweep: 360.0,
                part: Part::Track
            }]
        );

        // 0.95 では線路の角度が 360 - 342 - 2 × 22.918 = -27.837 度になり、
        // Compose は 270 + 342 + 22.918 = 634.918 度から反時計回りに描く
        let arcs = circular_arcs(40.0, 4.0, Frame::Determinate(0.95));
        assert_close(arcs[0].start, 634.918_3 - 27.836_6, "逆向きの線路の始まり");
        assert_close(arcs[0].sweep, 27.836_6, "逆向きの線路の角度");
        assert_eq!(arcs[0].part, Part::Track);
    }

    #[test]
    fn 不確定の円は回りながら弧の長さが変わる() {
        // 周期の始まりは回らず、弧は最小の長さ
        let arcs = circular_arcs(40.0, 4.0, Frame::Indeterminate(Duration::ZERO));
        let indicator = arcs
            .iter()
            .find(|arc| arc.part == Part::Indicator)
            .expect("弧を描くこと");
        assert_close(indicator.start, 0.0, "始まりの角度");
        assert_close(indicator.sweep, 36.0, "最小の弧");

        // 1650ms: 一定の回転 1080 × 1650 / 6000 = 297 度に、追加の回転 90 + 90 × 150 / 300 = 135 度を足す。
        // 弧は 0.1 + 0.77 × 1650 / 3000 だけ伸びる
        let arcs = circular_arcs(40.0, 4.0, Frame::Indeterminate(ms(1650)));
        let indicator = arcs
            .iter()
            .find(|arc| arc.part == Part::Indicator)
            .expect("弧を描くこと");
        assert_close(indicator.start, 297.0 + 135.0, "1650ms の角度");
        assert_close(indicator.sweep, (0.1 + 0.77 * 0.55) * 360.0, "1650ms の弧");

        // 4500ms: 追加の回転は 270 度で止まり、弧は後半のイージングで縮む。
        // 弧は `EasingStandardCubicBezier` (0.2, 0, 0, 1) を Python で評価した値 0.8778336 から
        // (0.87 - 0.77 × 0.8778336) × 360 度
        let arcs = circular_arcs(40.0, 4.0, Frame::Indeterminate(ms(4500)));
        let indicator = arcs
            .iter()
            .find(|arc| arc.part == Part::Indicator)
            .expect("弧を描くこと");
        assert_close(indicator.start, 810.0 + 270.0, "4500ms の角度");
        assert_close(indicator.sweep, 69.864_525, "4500ms の弧");
    }
}
