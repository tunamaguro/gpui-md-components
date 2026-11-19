// 寸法・色・形の並び・描く位置と大きさの計算・動きは Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/LoadingIndicator.kt
//         (`LoadingIndicator`、`ContainedLoadingIndicator`、`LoadingIndicatorImpl`、`LoadingIndicatorDefaults`、
//         `buildMorphSequence`、`calculateScaleFactor`、`processPath`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2024 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// バネが止まる時刻の推定 (`morph_settle_time`) は Jetpack Compose の animation-core から移植している
// Source: compose/animation/animation-core/src/commonMain/kotlin/androidx/compose/animation/core/SpringEstimation.kt
//         (`estimateUnderDamped`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2020 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の loading_indicator にある

//! MD3 Expressive のローディングインジケーター

mod material_shapes;
mod shapes;

use std::sync::OnceLock;
use std::time::Duration;

use gpui::{
    Animation, AnimationExt, App, Bounds, ElementId, Hsla, IntoElement, ParentElement, PathBuilder,
    Pixels, Refineable, RenderOnce, SharedString, SpringState, StatefulInteractiveElement,
    StyleRefinement, Styled, Window, canvas, div, prelude::FluentBuilder, px,
};
use gpui_headless::progress::{Progress as HeadlessProgress, ProgressBar};
use md3_theme::md3_tokens::component::loading_indicator;
use md3_theme::md3_tokens::supplement::loading_indicator as supplement;
use md3_theme::{SharedTheme, ThemeExt, dp, spring};

use crate::Progress;
use crate::shape::Shaped;
use material_shapes::{MaterialShape, rotate};
use shapes::{Morph, Point};

/// ローディングインジケーターの見た目の種類
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum LoadingIndicatorVariant {
    /// 容器を持たず、形だけを描く
    #[default]
    Uncontained,
    /// 形を円形の容器の中に描く。背景の上で目立たせたいときに使う
    Contained,
}

/// MD3 Expressive のローディングインジケーター
///
/// 形が変わりながら回り、処理を待っていることを示す。進み具合が分かるときは、進むにつれて円から別の形へ変わる。
/// 支援技術には role `progressbar` として公開され、進み具合が分かるときは割合を 0〜100 の値として公開する。
/// フォーカスも操作も持たない。アクセシブルな名前は [`LoadingIndicator::aria_label`] で与えること
///
/// 既定の大きさは 48×48dp で、形は大きさの短い方の辺に合わせて描く。[`Styled`] で指定したスタイルは既定の大きさより優先する
///
/// 進み具合が分からないときの動きは、モーション低減の設定 ([`App::reduce_motion`]) では止まり、最初の形の 1 枚の絵になる
#[derive(IntoElement)]
pub struct LoadingIndicator {
    id: ElementId,
    progress: Progress,
    variant: LoadingIndicatorVariant,
    aria_label: Option<SharedString>,
    style: StyleRefinement,
    theme: Option<SharedTheme>,
}

impl LoadingIndicator {
    /// `progress` を表す [`LoadingIndicatorVariant::Uncontained`] のローディングインジケーターを作る
    ///
    /// `id` は同じ親の中で一意にすること
    pub fn new(id: impl Into<ElementId>, progress: Progress) -> Self {
        Self {
            id: id.into(),
            progress,
            variant: LoadingIndicatorVariant::default(),
            aria_label: None,
            style: StyleRefinement::default(),
            theme: None,
        }
    }

    /// 見た目の種類を変える
    pub fn variant(mut self, variant: LoadingIndicatorVariant) -> Self {
        self.variant = variant;
        self
    }

    /// 支援技術に伝える名前を付ける
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

impl RenderOnce for LoadingIndicator {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let reduce_motion = cx.reduce_motion();
        let (container, indicator) = match self.variant {
            LoadingIndicatorVariant::Uncontained => {
                (None, loading_indicator::ACTIVE_INDICATOR_COLOR)
            }
            LoadingIndicatorVariant::Contained => (
                Some(loading_indicator::CONTAINED_CONTAINER_COLOR),
                loading_indicator::CONTAINED_ACTIVE_COLOR,
            ),
        };
        let indicator = theme.color(indicator);
        let size = gpui::size(
            dp(loading_indicator::CONTAINER_WIDTH),
            dp(loading_indicator::CONTAINER_HEIGHT),
        );

        let mut bar = ProgressBar::new(self.id, self.progress.into())
            .when_some(self.aria_label, |this, label| this.aria_label(label))
            .relative()
            .flex_none()
            .w(size.width)
            .h(size.height)
            .overflow_hidden()
            .when_some(container, |this, role| {
                // 容器の形は描画時の大きさから決め、大きさを変えても円形を保つ
                this.child(Shaped::new(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .bg(theme.color(role)),
                    theme.clone(),
                    loading_indicator::CONTAINER_SHAPE,
                ))
            });
        bar.style().refine(&self.style);

        match HeadlessProgress::from(self.progress).fraction() {
            Some(fraction) => bar.child(shape_canvas(indicator, determinate_frame(fraction))),
            None if reduce_motion => bar.child(shape_canvas(
                indicator,
                indeterminate_frame(Duration::ZERO, Duration::ZERO),
            )),
            None => bar.child(
                // 全体の回転と形の変化は周期が違うため、別のアニメーションで時刻を求める
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full()
                    .with_animation(
                        "loading-indicator-rotation",
                        Animation::new(supplement::GLOBAL_ROTATION_DURATION).repeat(),
                        move |this, rotation_delta| {
                            let rotation_time =
                                supplement::GLOBAL_ROTATION_DURATION.mul_f32(rotation_delta);
                            let morph_cycle = indeterminate_morph_cycle();
                            this.child(
                                div()
                                    .absolute()
                                    .top_0()
                                    .left_0()
                                    .size_full()
                                    .with_animation(
                                        "loading-indicator-morph",
                                        Animation::new(morph_cycle).repeat(),
                                        move |this, morph_delta| {
                                            let frame = indeterminate_frame(
                                                rotation_time,
                                                morph_cycle.mul_f32(morph_delta),
                                            );
                                            this.child(shape_canvas(indicator, frame))
                                        },
                                    ),
                            )
                        },
                    ),
            ),
        }
    }
}

impl Styled for LoadingIndicator {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

/// 変化ごとに 90 度ずつ進む回転の目標が一周する変化の数
const QUARTER_TURNS: usize = 4;

/// 形の列と回転の目標の両方が始めに戻るまでの時間
fn indeterminate_morph_cycle() -> Duration {
    // 形の数 (7) は 4 と互いに素なので、形の数 × 4 回の変化で両方がそろう
    supplement::MORPH_INTERVAL * (indeterminate_sequence().morphs.len() * QUARTER_TURNS) as u32
}

/// 形の変化の列と、回しても容器からはみ出さないための縮小率
struct Sequence {
    morphs: Vec<Morph>,
    scale_factor: f32,
}

impl Sequence {
    /// `polygons` を順に変える列。`circular` なら最後の形から最初の形へ戻る変化も加える (`buildMorphSequence`)
    fn new(shapes: Vec<shapes::RoundedPolygon>, circular: bool) -> Self {
        let normalized: Vec<_> = shapes.iter().map(|p| p.normalized()).collect();
        let count = if circular {
            normalized.len()
        } else {
            normalized.len().saturating_sub(1)
        };
        let morphs = (0..count)
            .map(|i| Morph::new(&normalized[i], &normalized[(i + 1) % normalized.len()]))
            .collect();
        // 回したときに占める範囲に対する形の大きさの比のうち最小のもの (`calculateScaleFactor`)。
        // 細長い形で全体が小さくなりすぎないよう、縦横の大きい方の比を使う
        let scale_factor = shapes.iter().fold(1.0f32, |scale, polygon| {
            let [l, t, r, b] = polygon.calculate_bounds();
            let [ml, mt, mr, mb] = polygon.calculate_max_bounds();
            scale.min(((r - l) / (mr - ml)).max((b - t) / (mb - mt)))
        });
        let active_indicator_scale = loading_indicator::ACTIVE_SIZE.0
            / loading_indicator::CONTAINER_WIDTH
                .0
                .min(loading_indicator::CONTAINER_HEIGHT.0);
        Self {
            morphs,
            scale_factor: scale_factor * active_indicator_scale,
        }
    }
}

/// 進み具合が分からないときの形の列 (`IndeterminateIndicatorPolygons`)
fn indeterminate_sequence() -> &'static Sequence {
    static SEQUENCE: OnceLock<Sequence> = OnceLock::new();
    SEQUENCE.get_or_init(|| {
        Sequence::new(
            [
                MaterialShape::SoftBurst,
                MaterialShape::Cookie9Sided,
                MaterialShape::Pentagon,
                MaterialShape::Pill,
                MaterialShape::Sunny,
                MaterialShape::Cookie4Sided,
                MaterialShape::Oval,
            ]
            .into_iter()
            .map(MaterialShape::polygon)
            .collect(),
            true,
        )
    })
}

/// 進み具合が分かるときの形の列 (`DeterminateIndicatorPolygons`)
fn determinate_sequence() -> &'static Sequence {
    static SEQUENCE: OnceLock<Sequence> = OnceLock::new();
    SEQUENCE.get_or_init(|| {
        Sequence::new(
            vec![
                // 円を回しておくと、同じ角度だけ回っている SoftBurst へなめらかに変わる
                MaterialShape::Circle
                    .polygon()
                    .transformed(rotate(360.0 / 20.0)),
                MaterialShape::SoftBurst.polygon(),
            ],
            false,
        )
    })
}

/// ある時点に描く形
#[derive(Clone, Copy, Debug, PartialEq)]
struct Frame {
    sequence: &'static Sequence,
    morph_index: usize,
    morph_progress: f32,
    /// 時計回りの回転の角度 (度)
    rotation: f32,
}

impl std::fmt::Debug for Sequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sequence")
            .field("morphs", &self.morphs.len())
            .finish()
    }
}

impl PartialEq for Sequence {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

/// 進んだ割合 `fraction` (0.0〜1.0) の形。進むにつれて円から SoftBurst へ変わり、反時計回りに半回転する
fn determinate_frame(fraction: f32) -> Frame {
    let sequence = determinate_sequence();
    let len = sequence.morphs.len();
    let morph_index = ((len as f32 * fraction) as usize).min(len.saturating_sub(1));
    // 最後の変化で進みが 1 のときは、剰余で 0 にならないよう 1 のままにする
    let morph_progress = if fraction == 1.0 && morph_index == len.saturating_sub(1) {
        1.0
    } else {
        (fraction * len as f32) % 1.0
    };
    Frame {
        sequence,
        morph_index,
        morph_progress,
        rotation: -fraction * 180.0,
    }
}

/// 全体の回転の周期の中の時刻 `rotation_time` と、形の変化の周期の中の時刻 `morph_time` の形
///
/// 650ms ごとに次の形へバネで変わり始め、変わるあいだに 90 度回る。全体は 4666ms で 1 回転する
fn indeterminate_frame(rotation_time: Duration, morph_time: Duration) -> Frame {
    let sequence = indeterminate_sequence();
    let len = sequence.morphs.len().max(1);
    let interval = (morph_time.as_millis() / supplement::MORPH_INTERVAL.as_millis()) as usize;
    let in_interval = morph_time - supplement::MORPH_INTERVAL * interval as u32;
    let global_rotation =
        rotation_time.as_secs_f32() / supplement::GLOBAL_ROTATION_DURATION.as_secs_f32() * 360.0;
    // バネが止まると次の形の始まりへ移り、回転の目標を 90 度進める
    let (morph_index, morph_progress, target_angle) = if in_interval < morph_settle_time() {
        let state = spring(supplement::MORPH_SPRING).step(
            SpringState {
                position: 0.0,
                velocity: 0.0,
            },
            1.0,
            in_interval.as_secs_f32(),
        );
        (
            interval % len,
            state.position,
            90.0 * ((interval + 1) % QUARTER_TURNS) as f32,
        )
    } else {
        (
            (interval + 1) % len,
            0.0,
            90.0 * ((interval + 2) % QUARTER_TURNS) as f32,
        )
    };
    Frame {
        sequence,
        morph_index,
        // バネの行き過ぎもそのまま形の変化に使う
        morph_progress,
        rotation: morph_progress * 90.0 + target_angle + global_rotation,
    }
}

/// 形の変化のバネが止まったとみなすまでの時間
fn morph_settle_time() -> Duration {
    // 0 から 1 へ初速 0 で動く減衰不足のバネで、振幅の包絡線が閾値を下回る時刻 (`estimateUnderDamped`)。
    // 移植元に合わせて、変位と速度は閾値で割った値で求め、ミリ秒に切り捨てる
    let stiffness = f64::from(supplement::MORPH_SPRING.stiffness);
    let damping = 2.0 * f64::from(supplement::MORPH_SPRING.damping_ratio) * stiffness.sqrt();
    let real = -damping * 0.5;
    let imaginary = (4.0 * stiffness - damping * damping).sqrt() * 0.5;
    let p0 = 1.0 / f64::from(supplement::MORPH_VISIBILITY_THRESHOLD);
    let c2 = -real * p0 / imaginary;
    let amplitude = (p0 * p0 + c2 * c2).sqrt();
    Duration::from_millis(((1.0 / amplitude).ln() / real * 1000.0) as u64)
}

/// 親いっぱいに広がり、`frame` の形を `color` で塗る要素
fn shape_canvas(color: Hsla, frame: Frame) -> impl IntoElement {
    canvas(
        |_, _, _| {},
        move |bounds, (), window, _| {
            if let Some(path) = shape_path(bounds, frame) {
                window.paint_path(path, color);
            }
        },
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
}

/// `bounds` の中央の正方形に `frame` の形を描くパス (`processPath`)
fn shape_path(bounds: Bounds<Pixels>, frame: Frame) -> Option<gpui::Path<Pixels>> {
    let morph = frame.sequence.morphs.get(frame.morph_index)?;
    let side = f32::from(bounds.size.width.min(bounds.size.height));
    let scale = side * frame.sequence.scale_factor;
    let cubics = morph.as_cubics(frame.morph_progress);
    // 拡大したパスの、制御点まで含めた範囲の中心を描く場所の中心に合わせ、その中心のまわりに回す
    let (mut left, mut top, mut right, mut bottom) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for cubic in &cubics {
        for p in [
            cubic.anchor0(),
            cubic.control0(),
            cubic.control1(),
            cubic.anchor1(),
        ] {
            left = left.min(p.x);
            top = top.min(p.y);
            right = right.max(p.x);
            bottom = bottom.max(p.y);
        }
    }
    let shape_center = Point::new((left + right) / 2.0, (top + bottom) / 2.0);
    let center = bounds.center();
    let rotation = rotate(frame.rotation);
    let place = |p: Point| {
        let p = rotation(Point::new(
            (p.x - shape_center.x) * scale,
            (p.y - shape_center.y) * scale,
        ));
        center + gpui::point(px(p.x), px(p.y))
    };
    let first = cubics.first()?;
    let mut builder = PathBuilder::fill();
    builder.move_to(place(first.anchor0()));
    for cubic in &cubics {
        builder.cubic_bezier_to(
            place(cubic.anchor1()),
            place(cubic.control0()),
            place(cubic.control1()),
        );
    }
    builder.close();
    builder.build().ok()
}

#[cfg(test)]
mod tests {
    use gpui::{TestAppContext, size};
    use md3_theme::md3_color::ColorRole;

    use super::*;

    fn ms(value: u64) -> Duration {
        Duration::from_millis(value)
    }

    fn assert_close(actual: f32, expected: f32, tolerance: f32, message: &str) {
        assert!(
            (actual - expected).abs() <= tolerance,
            "{message}: {actual} と {expected} の差が {tolerance} を超える"
        );
    }

    #[test]
    fn バネは_297ms_で止まったとみなす() {
        // 減衰比 0.6、剛性 200、閾値 0.1 を estimateUnderDamped の式で独立に評価した値
        let k = 200.0f64;
        let c = 2.0 * 0.6 * k.sqrt();
        let r = -c / 2.0;
        let w = (4.0 * k - c * c).sqrt() / 2.0;
        let expected = ((1.0 / (100.0 + (r * 10.0 / w).powi(2)).sqrt()).ln() / r * 1000.0) as u64;
        assert_eq!(expected, 297);
        assert_eq!(morph_settle_time(), ms(expected));
    }

    #[test]
    fn 不確定の形は_650ms_ごとに次の形へ変わり_90_度ずつ回る() {
        let start = indeterminate_frame(ms(0), ms(0));
        assert_eq!((start.morph_index, start.morph_progress), (0, 0.0));
        assert_close(start.rotation, 90.0, 1e-4, "始めの回転");

        // バネが止まった後は次の形の始まりで待つ
        let settled = indeterminate_frame(ms(0), ms(400));
        assert_eq!((settled.morph_index, settled.morph_progress), (1, 0.0));
        assert_close(settled.rotation, 180.0, 1e-4, "止まった後の回転");

        let second = indeterminate_frame(ms(0), ms(650 + 100));
        assert_eq!(second.morph_index, 1);
        assert!(second.morph_progress > 0.0 && second.morph_progress < 1.2);
        assert_close(
            second.rotation,
            second.morph_progress * 90.0 + 180.0,
            1e-4,
            "2 つ目の変化の回転",
        );

        // 最後の形から最初の形へ戻る変化がある
        let last = indeterminate_frame(ms(0), ms(650 * 6 + 100));
        assert_eq!(last.morph_index, 6);
        assert_eq!(indeterminate_sequence().morphs.len(), 7);
    }

    #[test]
    fn 形の列が一周しても回転の目標は続きから進む() {
        let cycle = indeterminate_morph_cycle();
        assert_eq!(cycle, ms(650 * 7 * 4));
        // 形の列が 1 周する 4550ms の前後でも、回転の目標は続きから進む
        let before = indeterminate_frame(ms(0), ms(4550 - 1));
        let after = indeterminate_frame(ms(0), ms(4550));
        assert_eq!((before.morph_index, after.morph_index), (0, 0));
        assert_close(after.rotation, before.rotation, 1e-3, "4550ms の前後の回転");
        // 周期の終わりから始めに戻るときも回転が続く
        let end = indeterminate_frame(ms(0), cycle - ms(1));
        let start = indeterminate_frame(ms(0), ms(0));
        assert_eq!(end.morph_index, start.morph_index);
        assert_close(end.rotation, start.rotation, 1e-3, "周期の境の回転");
    }

    #[test]
    fn 全体の回転は_4666ms_で_1_周する() {
        let quarter = indeterminate_frame(ms(4666 / 4), ms(400));
        assert_close(quarter.rotation, 180.0 + 90.0, 0.1, "4 分の 1 周の回転");
    }

    #[test]
    fn 決定的な形は進むにつれて変わり反時計回りに半回転する() {
        let start = determinate_frame(0.0);
        assert_eq!(
            (start.morph_index, start.morph_progress, start.rotation),
            (0, 0.0, 0.0)
        );
        let half = determinate_frame(0.5);
        assert_eq!(
            (half.morph_index, half.morph_progress, half.rotation),
            (0, 0.5, -90.0)
        );
        let done = determinate_frame(1.0);
        assert_eq!(
            (done.morph_index, done.morph_progress, done.rotation),
            (0, 1.0, -180.0)
        );
    }

    #[test]
    fn 形は回しても_38dp_の範囲に収まる() {
        let bounds = Bounds::new(gpui::point(px(0.0), px(0.0)), size(px(48.0), px(48.0)));
        let frames = (0..4666)
            .step_by(37)
            .map(|t| indeterminate_frame(ms(t as u64), ms(t as u64 * 3)))
            .chain((0..=10).map(|i| determinate_frame(i as f32 / 10.0)));
        for frame in frames {
            let sequence = frame.sequence;
            let morph = &sequence.morphs[frame.morph_index];
            let scale = 48.0 * sequence.scale_factor;
            // 形の点は中心から 38dp の円 (に行き過ぎの分を加えた範囲) を出ない
            for cubic in morph.as_cubics(frame.morph_progress.clamp(0.0, 1.0)) {
                for p in [cubic.anchor0(), cubic.anchor1()] {
                    let distance = ((p.x - 0.5).powi(2) + (p.y - 0.5).powi(2)).sqrt() * scale;
                    assert!(
                        distance <= 19.0 + 1.0,
                        "{frame:?} の点が中心から {distance} 離れる"
                    );
                }
            }
            assert!(shape_path(bounds, frame).is_some());
        }
    }

    /// 種類と、指定する大きさ (論理ピクセル)
    struct ElementView(LoadingIndicatorVariant, Option<f32>);

    impl gpui::Render for ElementView {
        fn render(
            &mut self,
            _window: &mut Window,
            _cx: &mut gpui::Context<Self>,
        ) -> impl IntoElement {
            div().p(px(8.0)).child(
                LoadingIndicator::new("indicator", Progress::Determinate(0.5))
                    .variant(self.0)
                    .aria_label("Loading")
                    .when_some(self.1, |this, side| this.size(px(side))),
            )
        }
    }

    /// 矩形の論理ピクセルの x, y, 幅, 高さと塗り、左上の角の半径
    type Quad = ((f32, f32, f32, f32), gpui::Background, f32);

    /// `variant` を大きさ `side` で描いたときの矩形と、容器の色。`side` が `None` なら既定の大きさ
    fn painted_quads(
        cx: &mut TestAppContext,
        variant: LoadingIndicatorVariant,
        side: Option<f32>,
    ) -> (Vec<Quad>, Hsla) {
        let (_, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            ElementView(variant, side)
        });
        cx.run_until_parked();
        cx.update(|window, cx| {
            let scale = window.scale_factor();
            let quads = window
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
            (quads, cx.md3_theme().color(ColorRole::PrimaryContainer))
        })
    }

    #[gpui::test]
    fn 容器のある種類は_48dp_の円形の容器を塗る(cx: &mut TestAppContext) {
        let (quads, container) = painted_quads(cx, LoadingIndicatorVariant::Contained, None);
        assert_eq!(
            quads,
            vec![((8.0, 8.0, 48.0, 48.0), container.into(), 24.0)],
            "容器だけを矩形で塗る"
        );
    }

    #[gpui::test]
    fn 大きさを変えても容器は円形を保つ(cx: &mut TestAppContext) {
        let (quads, container) = painted_quads(cx, LoadingIndicatorVariant::Contained, Some(80.0));
        assert_eq!(
            quads,
            vec![((8.0, 8.0, 80.0, 80.0), container.into(), 40.0)],
            "角の半径は描画時の短辺の半分"
        );
    }

    #[gpui::test]
    fn 容器のない種類は矩形を塗らない(cx: &mut TestAppContext) {
        let (quads, _) = painted_quads(cx, LoadingIndicatorVariant::Uncontained, None);
        assert!(
            quads.is_empty(),
            "形はパスで描き、矩形は塗らない: {quads:?}"
        );
    }
}
