// 角丸の多角形と、2 つの形の間の変形は androidx graphics-shapes から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: graphics/graphics-shapes/src/commonMain/kotlin/androidx/graphics/shapes/
//         Point.kt、Utils.kt、Cubic.kt (`Cubic`)、CornerRounding.kt (`CornerRounding`)、
//         Features.kt (`Feature`)、RoundedPolygon.kt (`RoundedPolygon`、`RoundedCorner`)、
//         Shapes.kt (`circle`、`star`)、Morph.kt (`Morph`)、PolygonMeasure.kt (`MeasuredPolygon`、
//         `LengthMeasurer`)、FeatureMapping.kt (`featureMapper`、`doMapping`)、
//         FloatMapping.kt (`DoubleMapper`、`linearMap`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2022, 2023 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
//
// 移すのはローディングインジケーターの形を作って変形させる部分だけ。
// 入力はこのモジュールの中で決めた形に限るため、移植元の入力検査 (`require`) は移していない。
// 検査で例外になる箇所は、パニックせずに移植元の正常な場合と同じ値を返す

use std::ops::{Add, Div, Mul, Sub};

const DISTANCE_EPSILON: f32 = 1e-4;
const ANGLE_EPSILON: f32 = 1e-6;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Point {
    pub(super) x: f32,
    pub(super) y: f32,
}

impl Point {
    pub(super) const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn distance(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    fn distance_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    fn dot(self, other: Point) -> f32 {
        self.x * other.x + self.y * other.y
    }

    fn clockwise(self, other: Point) -> bool {
        self.x * other.y - self.y * other.x > 0.0
    }

    /// 長さ 1 の同じ向きのベクトル。長さが 0 なら原点
    fn direction(self) -> Point {
        let d = self.distance();
        // 移植元は長さ 0 を例外にする。呼び出し元はどれも長さが 0 にならない場合だけ呼ぶ
        if d > 0.0 { self / d } else { Point::default() }
    }

    fn rotate90(self) -> Point {
        Point::new(-self.y, self.x)
    }
}

impl Add for Point {
    type Output = Point;
    fn add(self, other: Point) -> Point {
        Point::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Point {
    type Output = Point;
    fn sub(self, other: Point) -> Point {
        Point::new(self.x - other.x, self.y - other.y)
    }
}

impl Mul<f32> for Point {
    type Output = Point;
    fn mul(self, operand: f32) -> Point {
        Point::new(self.x * operand, self.y * operand)
    }
}

impl Div<f32> for Point {
    type Output = Point;
    fn div(self, operand: f32) -> Point {
        Point::new(self.x / operand, self.y / operand)
    }
}

fn interpolate(start: f32, stop: f32, fraction: f32) -> f32 {
    (1.0 - fraction) * start + fraction * stop
}

fn interpolate_point(start: Point, stop: Point, fraction: f32) -> Point {
    Point::new(
        interpolate(start.x, stop.x, fraction),
        interpolate(start.y, stop.y, fraction),
    )
}

fn radial_to_cartesian(radius: f32, angle_radians: f32) -> Point {
    Point::new(angle_radians.cos(), angle_radians.sin()) * radius
}

fn positive_modulo(num: f32, modulus: f32) -> f32 {
    (num % modulus + modulus) % modulus
}

fn convex(previous: Point, current: Point, next: Point) -> bool {
    (current - previous).clockwise(next - current)
}

/// 3 次ベジェ曲線。始点・制御点 2 つ・終点の順に x と y を並べる
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Cubic {
    points: [f32; 8],
}

impl Cubic {
    fn from_points(anchor0: Point, control0: Point, control1: Point, anchor1: Point) -> Self {
        Self {
            points: [
                anchor0.x, anchor0.y, control0.x, control0.y, control1.x, control1.y, anchor1.x,
                anchor1.y,
            ],
        }
    }

    pub(super) fn anchor0(&self) -> Point {
        Point::new(self.points[0], self.points[1])
    }

    pub(super) fn control0(&self) -> Point {
        Point::new(self.points[2], self.points[3])
    }

    pub(super) fn control1(&self) -> Point {
        Point::new(self.points[4], self.points[5])
    }

    pub(super) fn anchor1(&self) -> Point {
        Point::new(self.points[6], self.points[7])
    }

    fn straight_line(start: Point, end: Point) -> Self {
        Self::from_points(
            start,
            interpolate_point(start, end, 1.0 / 3.0),
            interpolate_point(start, end, 2.0 / 3.0),
            end,
        )
    }

    fn circular_arc(center: Point, p0: Point, p1: Point) -> Self {
        let p0d = (p0 - center).direction();
        let p1d = (p1 - center).direction();
        let rotated_p0 = p0d.rotate90();
        let rotated_p1 = p1d.rotate90();
        let clockwise = rotated_p0.dot(p1 - center) >= 0.0;
        let cosa = p0d.dot(p1d);
        if cosa > 0.999 {
            return Self::straight_line(p0, p1);
        }
        let k = (p0 - center).distance() * 4.0 / 3.0
            * ((2.0 * (1.0 - cosa)).sqrt() - (1.0 - cosa * cosa).sqrt())
            / (1.0 - cosa)
            * if clockwise { 1.0 } else { -1.0 };
        Self::from_points(p0, p0 + rotated_p0 * k, p1 - rotated_p1 * k, p1)
    }

    fn point_on_curve(&self, t: f32) -> Point {
        let u = 1.0 - t;
        let [a0x, a0y, c0x, c0y, c1x, c1y, a1x, a1y] = self.points;
        Point::new(
            a0x * (u * u * u)
                + c0x * (3.0 * t * u * u)
                + c1x * (3.0 * t * t * u)
                + a1x * (t * t * t),
            a0y * (u * u * u)
                + c0y * (3.0 * t * u * u)
                + c1y * (3.0 * t * t * u)
                + a1y * (t * t * t),
        )
    }

    fn zero_length(&self) -> bool {
        (self.points[0] - self.points[6]).abs() < DISTANCE_EPSILON
            && (self.points[1] - self.points[7]).abs() < DISTANCE_EPSILON
    }

    /// 始点・制御点・終点を囲む矩形 (移植元の `approximate = true`)。`[左, 上, 右, 下]` の順
    fn approximate_bounds(&self) -> [f32; 4] {
        let [a0x, a0y, c0x, c0y, c1x, c1y, a1x, a1y] = self.points;
        if self.zero_length() {
            return [a0x, a0y, a0x, a0y];
        }
        [
            a0x.min(a1x).min(c0x.min(c1x)),
            a0y.min(a1y).min(c0y.min(c1y)),
            a0x.max(a1x).max(c0x.max(c1x)),
            a0y.max(a1y).max(c0y.max(c1y)),
        ]
    }

    fn split(&self, t: f32) -> (Cubic, Cubic) {
        let u = 1.0 - t;
        let point = self.point_on_curve(t);
        let [a0x, a0y, c0x, c0y, c1x, c1y, a1x, a1y] = self.points;
        (
            Cubic {
                points: [
                    a0x,
                    a0y,
                    a0x * u + c0x * t,
                    a0y * u + c0y * t,
                    a0x * (u * u) + c0x * (2.0 * u * t) + c1x * (t * t),
                    a0y * (u * u) + c0y * (2.0 * u * t) + c1y * (t * t),
                    point.x,
                    point.y,
                ],
            },
            Cubic {
                points: [
                    point.x,
                    point.y,
                    c0x * (u * u) + c1x * (2.0 * u * t) + a1x * (t * t),
                    c0y * (u * u) + c1y * (2.0 * u * t) + a1y * (t * t),
                    c1x * u + a1x * t,
                    c1y * u + a1y * t,
                    a1x,
                    a1y,
                ],
            },
        )
    }

    fn reverse(&self) -> Cubic {
        Self::from_points(
            self.anchor1(),
            self.control1(),
            self.control0(),
            self.anchor0(),
        )
    }

    fn transformed(&self, f: &impl Fn(Point) -> Point) -> Cubic {
        Self::from_points(
            f(self.anchor0()),
            f(self.control0()),
            f(self.control1()),
            f(self.anchor1()),
        )
    }

    fn interpolate(start: &Cubic, end: &Cubic, progress: f32) -> Cubic {
        Cubic {
            points: std::array::from_fn(|i| interpolate(start.points[i], end.points[i], progress)),
        }
    }
}

/// 角の丸め。`radius` は丸めの半径、`smoothing` は 0〜1 で、角の前後の辺へ丸めを広げる割合
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct CornerRounding {
    pub(super) radius: f32,
    pub(super) smoothing: f32,
}

impl CornerRounding {
    pub(super) const UNROUNDED: Self = Self::new(0.0, 0.0);

    pub(super) const fn new(radius: f32, smoothing: f32) -> Self {
        Self { radius, smoothing }
    }
}

#[derive(Clone, Debug)]
enum Feature {
    Edge(Vec<Cubic>),
    Corner { cubics: Vec<Cubic>, convex: bool },
}

impl Feature {
    fn cubics(&self) -> &[Cubic] {
        match self {
            Feature::Edge(cubics) | Feature::Corner { cubics, .. } => cubics,
        }
    }

    fn transformed(&self, f: &impl Fn(Point) -> Point) -> Feature {
        let map = |cubics: &[Cubic]| cubics.iter().map(|c| c.transformed(f)).collect();
        match self {
            Feature::Edge(cubics) => Feature::Edge(map(cubics)),
            Feature::Corner { cubics, convex } => Feature::Corner {
                cubics: map(cubics),
                convex: *convex,
            },
        }
    }
}

/// 角を丸めた多角形
#[derive(Clone, Debug)]
pub(super) struct RoundedPolygon {
    features: Vec<Feature>,
    center: Point,
    cubics: Vec<Cubic>,
}

impl RoundedPolygon {
    fn from_features(features: Vec<Feature>, center: Point) -> Self {
        // 最後の曲線の終点を最初の曲線の始点にそろえる。わずかなずれでも描画に跡が残るため
        let mut cubics = Vec::new();
        let mut first_cubic: Option<Cubic> = None;
        let mut last_cubic: Option<Cubic> = None;
        let mut first_feature_split: Option<([Cubic; 2], [Cubic; 2])> = None;
        if let Some(first) = features.first()
            && let [c0, c1, c2] = first.cubics()
        {
            let (start, end) = c1.split(0.5);
            first_feature_split = Some(([*c0, start], [end, *c2]));
        }
        for i in 0..=features.len() {
            let feature_cubics: &[Cubic] = match (i, &first_feature_split) {
                (0, Some((_, split_end))) => split_end,
                (i, Some((split_start, _))) if i == features.len() => split_start,
                (i, None) if i == features.len() => break,
                (i, _) => features[i].cubics(),
            };
            for cubic in feature_cubics {
                if !cubic.zero_length() {
                    if let Some(last) = last_cubic {
                        cubics.push(last);
                    }
                    last_cubic = Some(*cubic);
                    if first_cubic.is_none() {
                        first_cubic = Some(*cubic);
                    }
                } else if let Some(last) = last_cubic.as_mut() {
                    // 長さ 0 の曲線を続けて捨てるとずれが積み重なるため、直前の曲線の終点を常に最新の点にする
                    last.points[6] = cubic.points[6];
                    last.points[7] = cubic.points[7];
                }
            }
        }
        match (last_cubic, first_cubic) {
            (Some(last), Some(first)) => cubics.push(Cubic::from_points(
                last.anchor0(),
                last.control0(),
                last.control1(),
                first.anchor0(),
            )),
            _ => cubics.push(Cubic::from_points(center, center, center, center)),
        }
        Self {
            features,
            center,
            cubics,
        }
    }

    /// 頂点とその丸めを順に結んだ多角形。`center` は形の中心
    pub(super) fn from_vertices(vertices: &[(Point, CornerRounding)], center: Point) -> Self {
        let n = vertices.len();
        if n == 0 {
            return Self::from_features(Vec::new(), center);
        }
        let corners: Vec<RoundedCorner> = (0..n)
            .map(|i| {
                RoundedCorner::new(
                    vertices[(i + n - 1) % n].0,
                    vertices[i].0,
                    vertices[(i + 1) % n].0,
                    vertices[i].1,
                )
            })
            .collect();

        // 辺ごとに、両端の角の丸めに使える割合 (丸めの分、なめらかさの分) を決める。
        // 辺 i は角 i から角 i + 1 まで
        let cut_adjusts: Vec<(f32, f32)> = (0..n)
            .map(|ix| {
                let next = (ix + 1) % n;
                let expected_round_cut =
                    corners[ix].expected_round_cut + corners[next].expected_round_cut;
                let expected_cut = corners[ix].expected_cut() + corners[next].expected_cut();
                let side_size = (vertices[ix].0 - vertices[next].0).distance();
                if expected_round_cut > side_size {
                    (side_size / expected_round_cut, 0.0)
                } else if expected_cut > side_size {
                    (
                        1.0,
                        (side_size - expected_round_cut) / (expected_cut - expected_round_cut),
                    )
                } else {
                    (1.0, 1.0)
                }
            })
            .collect();

        let corner_cubics: Vec<Vec<Cubic>> = (0..n)
            .map(|i| {
                let allowed = |side: usize| {
                    let (round_cut_ratio, cut_ratio) = cut_adjusts[side];
                    corners[i].expected_round_cut * round_cut_ratio
                        + (corners[i].expected_cut() - corners[i].expected_round_cut) * cut_ratio
                };
                corners[i].cubics(allowed((i + n - 1) % n), allowed(i))
            })
            .collect();

        let mut features = Vec::with_capacity(n * 2);
        for i in 0..n {
            let next = (i + 1) % n;
            let is_convex = convex(vertices[(i + n - 1) % n].0, vertices[i].0, vertices[next].0);
            let (Some(last), Some(next_first)) =
                (corner_cubics[i].last(), corner_cubics[next].first())
            else {
                continue;
            };
            let edge = Cubic::straight_line(last.anchor1(), next_first.anchor0());
            features.push(Feature::Corner {
                cubics: corner_cubics[i].clone(),
                convex: is_convex,
            });
            features.push(Feature::Edge(vec![edge]));
        }
        Self::from_features(features, center)
    }

    /// 半径 1 の円に近い形。頂点を `num_vertices` 個持つ正多角形の角を丸めて作る
    pub(super) fn circle(num_vertices: usize) -> Self {
        let theta = std::f32::consts::PI / num_vertices as f32;
        let polygon_radius = 1.0 / theta.cos();
        let rounding = CornerRounding::new(1.0, 0.0);
        let vertices: Vec<(Point, CornerRounding)> = (0..num_vertices)
            .map(|i| {
                let angle = std::f32::consts::PI / num_vertices as f32 * 2.0 * i as f32;
                (radial_to_cartesian(polygon_radius, angle), rounding)
            })
            .collect();
        Self::from_vertices(&vertices, Point::default())
    }

    /// 外側の半径 1、内側の半径 `inner_radius` の星形。すべての頂点を `rounding` で丸める
    pub(super) fn star(
        num_vertices_per_radius: usize,
        inner_radius: f32,
        rounding: CornerRounding,
    ) -> Self {
        let n = num_vertices_per_radius as f32;
        let pi = std::f32::consts::PI;
        let vertices: Vec<(Point, CornerRounding)> = (0..num_vertices_per_radius)
            .flat_map(|i| {
                let i = i as f32;
                [
                    (radial_to_cartesian(1.0, pi / n * 2.0 * i), rounding),
                    (
                        radial_to_cartesian(inner_radius, pi / n * (2.0 * i + 1.0)),
                        rounding,
                    ),
                ]
            })
            .collect();
        Self::from_vertices(&vertices, Point::default())
    }

    #[cfg(test)]
    fn cubics(&self) -> &[Cubic] {
        &self.cubics
    }

    /// すべての点を `f` で移した形
    pub(super) fn transformed(&self, f: impl Fn(Point) -> Point) -> Self {
        let features = self.features.iter().map(|ft| ft.transformed(&f)).collect();
        Self::from_features(features, f(self.center))
    }

    /// 縦横の比を保ったまま、長い方の辺が 0〜1 に収まるように移し、短い方を中央に寄せた形
    pub(super) fn normalized(&self) -> Self {
        let [left, top, right, bottom] = self.calculate_bounds();
        let width = right - left;
        let height = bottom - top;
        let side = width.max(height);
        let offset_x = (side - width) / 2.0 - left;
        let offset_y = (side - height) / 2.0 - top;
        self.transformed(|p| Point::new((p.x + offset_x) / side, (p.y + offset_y) / side))
    }

    /// 曲線の始点・制御点・終点をすべて囲む矩形。`[左, 上, 右, 下]` の順
    pub(super) fn calculate_bounds(&self) -> [f32; 4] {
        // 移植元の初期値 (Kotlin の Float.MAX_VALUE と Float.MIN_VALUE) に合わせる
        let mut bounds = [f32::MAX, f32::MAX, f32::from_bits(1), f32::from_bits(1)];
        for cubic in &self.cubics {
            let [l, t, r, b] = cubic.approximate_bounds();
            bounds = [
                bounds[0].min(l),
                bounds[1].min(t),
                bounds[2].max(r),
                bounds[3].max(b),
            ];
        }
        bounds
    }

    /// 中心のまわりにどの角度で回しても収まる正方形。`[左, 上, 右, 下]` の順
    pub(super) fn calculate_max_bounds(&self) -> [f32; 4] {
        let max_dist_squared = self.cubics.iter().fold(0.0f32, |acc, cubic| {
            let anchor = (cubic.anchor0() - self.center).distance_squared();
            let middle = (cubic.point_on_curve(0.5) - self.center).distance_squared();
            acc.max(anchor.max(middle))
        });
        let distance = max_dist_squared.sqrt();
        [
            self.center.x - distance,
            self.center.y - distance,
            self.center.x + distance,
            self.center.y + distance,
        ]
    }
}

struct RoundedCorner {
    p0: Point,
    p1: Point,
    p2: Point,
    d1: Point,
    d2: Point,
    corner_radius: f32,
    smoothing: f32,
    expected_round_cut: f32,
}

impl RoundedCorner {
    fn new(p0: Point, p1: Point, p2: Point, rounding: CornerRounding) -> Self {
        let v01 = p0 - p1;
        let v21 = p2 - p1;
        let d01 = v01.distance();
        let d21 = v21.distance();
        if d01 > 0.0 && d21 > 0.0 {
            let d1 = v01 / d01;
            let d2 = v21 / d21;
            let cos_angle = d1.dot(d2);
            let sin_angle = (1.0 - cos_angle * cos_angle).sqrt();
            // tan(A/2) = sinA / (1 + cosA) = radius / cut から、半径を得るために辺を切る長さを求める
            let expected_round_cut = if sin_angle > 1e-3 {
                rounding.radius * (cos_angle + 1.0) / sin_angle
            } else {
                0.0
            };
            Self {
                p0,
                p1,
                p2,
                d1,
                d2,
                corner_radius: rounding.radius,
                smoothing: rounding.smoothing,
                expected_round_cut,
            }
        } else {
            Self {
                p0,
                p1,
                p2,
                d1: Point::default(),
                d2: Point::default(),
                corner_radius: 0.0,
                smoothing: 0.0,
                expected_round_cut: 0.0,
            }
        }
    }

    fn expected_cut(&self) -> f32 {
        (1.0 + self.smoothing) * self.expected_round_cut
    }

    fn cubics(&self, allowed_cut0: f32, allowed_cut1: f32) -> Vec<Cubic> {
        let allowed_cut = allowed_cut0.min(allowed_cut1);
        if self.expected_round_cut < DISTANCE_EPSILON
            || allowed_cut < DISTANCE_EPSILON
            || self.corner_radius < DISTANCE_EPSILON
        {
            return vec![Cubic::straight_line(self.p1, self.p1)];
        }
        let actual_round_cut = allowed_cut.min(self.expected_round_cut);
        let actual_smoothing0 = self.actual_smoothing(allowed_cut0);
        let actual_smoothing1 = self.actual_smoothing(allowed_cut1);
        let actual_r = self.corner_radius * actual_round_cut / self.expected_round_cut;
        let center_distance = (actual_r * actual_r + actual_round_cut * actual_round_cut).sqrt();
        let center = self.p1 + ((self.d1 + self.d2) / 2.0).direction() * center_distance;
        let circle_intersection0 = self.p1 + self.d1 * actual_round_cut;
        let circle_intersection2 = self.p1 + self.d2 * actual_round_cut;
        let flanking0 = flanking_curve(
            actual_round_cut,
            actual_smoothing0,
            self.p1,
            self.p0,
            circle_intersection0,
            circle_intersection2,
            center,
            actual_r,
        );
        let flanking2 = flanking_curve(
            actual_round_cut,
            actual_smoothing1,
            self.p1,
            self.p2,
            circle_intersection2,
            circle_intersection0,
            center,
            actual_r,
        )
        .reverse();
        vec![
            flanking0,
            Cubic::circular_arc(center, flanking0.anchor1(), flanking2.anchor0()),
            flanking2,
        ]
    }

    fn actual_smoothing(&self, allowed_cut: f32) -> f32 {
        if allowed_cut > self.expected_cut() {
            self.smoothing
        } else if allowed_cut > self.expected_round_cut {
            self.smoothing * (allowed_cut - self.expected_round_cut)
                / (self.expected_cut() - self.expected_round_cut)
        } else {
            0.0
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "移植元の computeFlankingCurve と同じ引数で対応を追えるようにする"
)]
fn flanking_curve(
    actual_round_cut: f32,
    actual_smoothing: f32,
    corner: Point,
    side_start: Point,
    circle_segment_intersection: Point,
    other_circle_segment_intersection: Point,
    circle_center: Point,
    actual_r: f32,
) -> Cubic {
    let side_direction = (side_start - corner).direction();
    let curve_start = corner + side_direction * actual_round_cut * (1.0 + actual_smoothing);
    // なめらかさが 0 なら円弧の区間を全部使い、1 なら使わない。その間は割合に応じて近似で削る
    let p = interpolate_point(
        circle_segment_intersection,
        (circle_segment_intersection + other_circle_segment_intersection) / 2.0,
        actual_smoothing,
    );
    let curve_end = circle_center + (p - circle_center).direction() * actual_r;
    // 円側の制御点は、円と脇の曲線の境での円の接線と、辺の直線の交点に置く
    let circle_tangent = (curve_end - circle_center).rotate90();
    let anchor_end = line_intersection(side_start, side_direction, curve_end, circle_tangent)
        .unwrap_or(circle_segment_intersection);
    let anchor_start = (curve_start + anchor_end * 2.0) / 3.0;
    Cubic::from_points(curve_start, anchor_start, anchor_end, curve_end)
}

fn line_intersection(p0: Point, d0: Point, p1: Point, d1: Point) -> Option<Point> {
    let rotated_d1 = d1.rotate90();
    let den = d0.dot(rotated_d1);
    if den.abs() < DISTANCE_EPSILON {
        return None;
    }
    let num = (p1 - p0).dot(rotated_d1);
    // den / num の絶対値が DISTANCE_EPSILON より小さいかを、割り算をせずに確かめる
    if den.abs() < DISTANCE_EPSILON * num.abs() {
        return None;
    }
    Some(p0 + d0 * (num / den))
}

/// 2 つの形の間を補間する変形
#[derive(Clone, Debug)]
pub(super) struct Morph {
    matched: Vec<(Cubic, Cubic)>,
}

impl Morph {
    /// `start` から `end` への変形を作る。両方の形の曲線を、輪郭の上の対応する位置で組にする
    pub(super) fn new(start: &RoundedPolygon, end: &RoundedPolygon) -> Self {
        let measured1 = MeasuredPolygon::measure(start);
        let measured2 = MeasuredPolygon::measure(end);
        let mapper = feature_mapper(&measured1.features, &measured2.features);
        // 1 つ目の形の進み 0 に対応する、2 つ目の形の位置で切って回す
        let cut_point = mapper.map(0.0);
        let bs1 = measured1;
        let bs2 = measured2.cut_and_shift(cut_point);

        let mut matched = Vec::new();
        let mut i1 = 0;
        let mut i2 = 0;
        let mut b1 = bs1.cubics.get(i1).copied();
        i1 += 1;
        let mut b2 = bs2.cubics.get(i2).copied();
        i2 += 1;
        while let (Some(c1), Some(c2)) = (b1, b2) {
            // 進みはどちらも 1 つ目の形から見た値
            let b1a = if i1 == bs1.cubics.len() { 1.0 } else { c1.end };
            let b2a = if i2 == bs2.cubics.len() {
                1.0
            } else {
                mapper.map_back(positive_modulo(c2.end + cut_point, 1.0))
            };
            let minb = b1a.min(b2a);
            let (seg1, new_b1) = if b1a > minb + ANGLE_EPSILON {
                let (a, b) = c1.cut_at_progress(minb);
                (a, Some(b))
            } else {
                let next = bs1.cubics.get(i1).copied();
                i1 += 1;
                (c1, next)
            };
            let (seg2, new_b2) = if b2a > minb + ANGLE_EPSILON {
                let (a, b) = c2.cut_at_progress(positive_modulo(mapper.map(minb) - cut_point, 1.0));
                (a, Some(b))
            } else {
                let next = bs2.cubics.get(i2).copied();
                i2 += 1;
                (c2, next)
            };
            matched.push((seg1.cubic, seg2.cubic));
            b1 = new_b1;
            b2 = new_b2;
        }
        Self { matched }
    }

    /// 進み `progress` (0 で始めの形、1 で終わりの形) の形を表す閉じた曲線の列
    ///
    /// 0〜1 の外の値は、その外側へ補間を延ばした形になる
    pub(super) fn as_cubics(&self, progress: f32) -> Vec<Cubic> {
        let mut cubics: Vec<Cubic> = self
            .matched
            .iter()
            .map(|(start, end)| Cubic::interpolate(start, end, progress))
            .collect();
        // 最後の曲線の終点を最初の曲線の始点にそろえる
        if let Some(first) = cubics.first().map(Cubic::anchor0)
            && let Some(last) = cubics.last_mut()
        {
            last.points[6] = first.x;
            last.points[7] = first.y;
        }
        cubics
    }
}

#[derive(Clone, Copy, Debug)]
struct ProgressableFeature {
    progress: f32,
    convex: bool,
    representative_point: Point,
}

#[derive(Clone, Copy, Debug)]
struct MeasuredCubic {
    cubic: Cubic,
    start: f32,
    end: f32,
    measured_size: f32,
}

impl MeasuredCubic {
    fn new(cubic: Cubic, start: f32, end: f32) -> Self {
        Self {
            cubic,
            start,
            end,
            measured_size: measure_cubic(&cubic),
        }
    }

    fn cut_at_progress(&self, cut_progress: f32) -> (MeasuredCubic, MeasuredCubic) {
        // 浮動小数点の誤差で範囲をわずかに外れることがあるため、この曲線の範囲に収める
        let bounded = cut_progress.clamp(self.start, self.end);
        let relative = (bounded - self.start) / (self.end - self.start);
        let t = find_cubic_cut_point(&self.cubic, relative * self.measured_size);
        let (c1, c2) = self.cubic.split(t);
        (
            MeasuredCubic::new(c1, self.start, bounded),
            MeasuredCubic::new(c2, bounded, self.end),
        )
    }
}

#[derive(Clone, Debug)]
struct MeasuredPolygon {
    cubics: Vec<MeasuredCubic>,
    features: Vec<ProgressableFeature>,
}

impl MeasuredPolygon {
    fn new(features: Vec<ProgressableFeature>, cubics: &[Cubic], outline_progress: &[f32]) -> Self {
        let mut measured: Vec<MeasuredCubic> = Vec::new();
        let mut start = 0.0;
        for (index, cubic) in cubics.iter().enumerate() {
            let end = outline_progress[index + 1];
            // 長さが 0 に近い曲線を除く
            if end - outline_progress[index] > DISTANCE_EPSILON {
                measured.push(MeasuredCubic::new(*cubic, start, end));
                start = end;
            }
        }
        // 末尾の曲線を除いた場合も、最後の曲線が 1 で終わるようにする
        if let Some(last) = measured.last_mut() {
            last.end = 1.0;
        }
        Self {
            cubics: measured,
            features,
        }
    }

    fn measure(polygon: &RoundedPolygon) -> Self {
        let mut cubics = Vec::new();
        let mut feature_to_cubic = Vec::new();
        for feature in &polygon.features {
            let feature_cubics = feature.cubics();
            for (index, cubic) in feature_cubics.iter().enumerate() {
                if let Feature::Corner { convex, .. } = feature
                    && index == feature_cubics.len() / 2
                {
                    feature_to_cubic.push((
                        *convex,
                        feature_representative_point(feature),
                        cubics.len(),
                    ));
                }
                cubics.push(*cubic);
            }
        }
        let mut measures = Vec::with_capacity(cubics.len() + 1);
        let mut total = 0.0;
        measures.push(total);
        for cubic in &cubics {
            total += measure_cubic(cubic);
            measures.push(total);
        }
        let outline_progress: Vec<f32> = measures.iter().map(|m| m / total).collect();
        let features = feature_to_cubic
            .into_iter()
            .map(|(convex, representative_point, ix)| ProgressableFeature {
                progress: positive_modulo(
                    (outline_progress[ix] + outline_progress[ix + 1]) / 2.0,
                    1.0,
                ),
                convex,
                representative_point,
            })
            .collect();
        Self::new(features, &cubics, &outline_progress)
    }

    /// 輪郭の進み `cutting_point` の位置から始まるように、曲線を切って並べ替えた同じ形
    fn cut_and_shift(&self, cutting_point: f32) -> Self {
        if cutting_point < DISTANCE_EPSILON {
            return self.clone();
        }
        let n = self.cubics.len();
        let Some(target_index) = self
            .cubics
            .iter()
            .position(|c| (c.start..=c.end).contains(&cutting_point))
        else {
            return self.clone();
        };
        let (b1, b2) = self.cubics[target_index].cut_at_progress(cutting_point);
        let mut cubics = Vec::with_capacity(n + 1);
        cubics.push(b2.cubic);
        for i in 1..n {
            cubics.push(self.cubics[(i + target_index) % n].cubic);
        }
        cubics.push(b1.cubic);

        let outline_progress: Vec<f32> = (0..n + 2)
            .map(|index| match index {
                0 => 0.0,
                i if i == n + 1 => 1.0,
                i => positive_modulo(
                    self.cubics[(target_index + i - 1) % n].end - cutting_point,
                    1.0,
                ),
            })
            .collect();
        let features = self
            .features
            .iter()
            .map(|f| ProgressableFeature {
                progress: positive_modulo(f.progress - cutting_point, 1.0),
                ..*f
            })
            .collect();
        Self::new(features, &cubics, &outline_progress)
    }
}

/// 曲線を 3 つの線分で近似した長さ
fn measure_cubic(cubic: &Cubic) -> f32 {
    closest_progress_to(cubic, f32::INFINITY).1
}

/// 始点からの長さが `measure` になる曲線のパラメーター t
fn find_cubic_cut_point(cubic: &Cubic, measure: f32) -> f32 {
    closest_progress_to(cubic, measure).0
}

fn closest_progress_to(cubic: &Cubic, threshold: f32) -> (f32, f32) {
    // 円弧で 98.5% の精度になる最小の分割数
    const SEGMENTS: usize = 3;
    let mut total = 0.0;
    let mut remainder = threshold;
    let mut prev = cubic.anchor0();
    for i in 1..=SEGMENTS {
        let progress = i as f32 / SEGMENTS as f32;
        let point = cubic.point_on_curve(progress);
        let segment = (point - prev).distance();
        if segment >= remainder {
            return (
                progress - (1.0 - remainder / segment) / SEGMENTS as f32,
                threshold,
            );
        }
        remainder -= segment;
        total += segment;
        prev = point;
    }
    (1.0, total)
}

fn feature_representative_point(feature: &Feature) -> Point {
    let cubics = feature.cubics();
    match (cubics.first(), cubics.last()) {
        (Some(first), Some(last)) => (first.anchor0() + last.anchor1()) / 2.0,
        _ => Point::default(),
    }
}

/// 2 つの形の角を、近いものから順に、交差しないように対応させる
fn feature_mapper(
    features1: &[ProgressableFeature],
    features2: &[ProgressableFeature],
) -> DoubleMapper {
    let mut distances = Vec::new();
    for (i1, f1) in features1.iter().enumerate() {
        for (i2, f2) in features2.iter().enumerate() {
            // 凸の角と凹の角は対応させない
            if f1.convex != f2.convex {
                continue;
            }
            let d = (f1.representative_point - f2.representative_point).distance_squared();
            distances.push((d, i1, i2));
        }
    }
    distances.sort_by(|a, b| a.0.total_cmp(&b.0));

    let mapping: Vec<(f32, f32)> = match distances.as_slice() {
        [] => vec![(0.0, 0.0), (0.5, 0.5)],
        [(_, i1, i2)] => {
            let p1 = features1[*i1].progress;
            let p2 = features2[*i2].progress;
            vec![(p1, p2), ((p1 + 0.5) % 1.0, (p2 + 0.5) % 1.0)]
        }
        _ => {
            let mut mapping: Vec<(f32, f32)> = Vec::new();
            let mut used1 = vec![false; features1.len()];
            let mut used2 = vec![false; features2.len()];
            for &(_, i1, i2) in &distances {
                if used1[i1] || used2[i2] {
                    continue;
                }
                let p1 = features1[i1].progress;
                let p2 = features2[i2].progress;
                // 同じ進みの角が 2 つあるときは移植元では例外になる。後の方を使わない
                let Err(insertion) = mapping.binary_search_by(|m| m.0.total_cmp(&p1)) else {
                    continue;
                };
                let n = mapping.len();
                if n >= 1 {
                    let (before1, before2) = mapping[(insertion + n - 1) % n];
                    let (after1, after2) = mapping[insertion % n];
                    // 近すぎる角どうしは対応を不安定にするため使わない
                    if progress_distance(p1, before1) < DISTANCE_EPSILON
                        || progress_distance(p1, after1) < DISTANCE_EPSILON
                        || progress_distance(p2, before2) < DISTANCE_EPSILON
                        || progress_distance(p2, after2) < DISTANCE_EPSILON
                    {
                        continue;
                    }
                    // 2 つ以上あるときは、対応が交差しないものだけを加える
                    if n > 1 && !progress_in_range(p2, before2, after2) {
                        continue;
                    }
                }
                mapping.insert(insertion, (p1, p2));
                used1[i1] = true;
                used2[i2] = true;
            }
            mapping
        }
    };
    DoubleMapper {
        source: mapping.iter().map(|m| m.0).collect(),
        target: mapping.iter().map(|m| m.1).collect(),
    }
}

/// 0〜1 を一周とする進みの間の対応。与えた対応点の間は線形に補間し、1 を超えると 0 に戻る
#[derive(Debug)]
struct DoubleMapper {
    source: Vec<f32>,
    target: Vec<f32>,
}

impl DoubleMapper {
    fn map(&self, x: f32) -> f32 {
        linear_map(&self.source, &self.target, x)
    }

    fn map_back(&self, x: f32) -> f32 {
        linear_map(&self.target, &self.source, x)
    }
}

fn linear_map(x_values: &[f32], y_values: &[f32], x: f32) -> f32 {
    let n = x_values.len();
    let start = (0..n)
        .find(|&i| progress_in_range(x, x_values[i], x_values[(i + 1) % n]))
        .unwrap_or(0);
    let end = (start + 1) % n;
    let segment_size_x = positive_modulo(x_values[end] - x_values[start], 1.0);
    let segment_size_y = positive_modulo(y_values[end] - y_values[start], 1.0);
    let position = if segment_size_x < 0.001 {
        0.5
    } else {
        positive_modulo(x - x_values[start], 1.0) / segment_size_x
    };
    positive_modulo(y_values[start] + segment_size_y * position, 1.0)
}

/// `progress` が `from` から `to` までに入るか。進みは一周するため、`to < from` なら 1 をまたぐ範囲になる
fn progress_in_range(progress: f32, from: f32, to: f32) -> bool {
    if to >= from {
        (from..=to).contains(&progress)
    } else {
        progress >= from || progress <= to
    }
}

/// 一周する進みの間の短い方の距離
fn progress_distance(p1: f32, p2: f32) -> f32 {
    let d = (p1 - p2).abs();
    d.min(1.0 - d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f32, expected: f32, tolerance: f32, message: &str) {
        assert!(
            (actual - expected).abs() <= tolerance,
            "{message}: {actual} と {expected} の差が {tolerance} を超える"
        );
    }

    fn regular(num_vertices: usize, rounding: CornerRounding) -> RoundedPolygon {
        let vertices: Vec<(Point, CornerRounding)> = (0..num_vertices)
            .map(|i| {
                let angle = std::f32::consts::PI / num_vertices as f32 * 2.0 * i as f32;
                (radial_to_cartesian(1.0, angle), rounding)
            })
            .collect();
        RoundedPolygon::from_vertices(&vertices, Point::default())
    }

    // PolygonMeasureTest.regularPolygonMeasure
    #[test]
    fn 角のない正多角形の辺は輪郭を等分する() {
        for sides in [3, 5, 8, 12, 20] {
            let measured = MeasuredPolygon::measure(&regular(sides, CornerRounding::UNROUNDED));
            assert_eq!(measured.cubics.len(), sides, "{sides} 角形の辺の数");
            for (index, cubic) in measured.cubics.iter().enumerate() {
                assert_close(
                    cubic.start,
                    index as f32 / sides as f32,
                    1e-4,
                    "辺の始まりの進み",
                );
            }
        }
    }

    // PolygonMeasureTest.measureCircle
    #[test]
    fn 円の長さを_1_5_パーセントの誤差で測る() {
        for vertices in [3, 4, 5, 6, 10, 20] {
            let circle = RoundedPolygon::circle(vertices);
            let length: f32 = circle.cubics().iter().map(measure_cubic).sum();
            let expected = 2.0 * std::f32::consts::PI;
            assert_close(length, expected, 0.015 * expected, "円周");
        }
    }

    // FloatMappingTest.simpleMappingTest / multiplePointTest
    #[test]
    fn 対応点の間を線形に補間し_逆向きにも戻せる() {
        let mapper = DoubleMapper {
            source: vec![0.0, 0.5],
            target: vec![0.0, 0.25],
        };
        let multiple = DoubleMapper {
            source: vec![0.4, 0.5, 0.0],
            target: vec![0.2, 0.22, 0.8],
        };
        for i in 0..1000 {
            let x = i as f32 / 1000.0;
            let y = if x < 0.5 {
                x / 2.0
            } else {
                (3.0 * x - 1.0) / 2.0
            };
            assert_close(mapper.map(x), y, 1e-4, "対応先");
            assert_close(mapper.map_back(y), x, 1e-4, "対応元");

            let y = if x < 0.4 {
                (0.8 + x) % 1.0
            } else if x < 0.5 {
                0.2 + (x - 0.4) / 5.0
            } else {
                0.22 + (x - 0.5) * 1.16
            };
            assert_close(
                progress_distance(multiple.map(x), y),
                0.0,
                1e-4,
                "複数点の対応先",
            );
        }
    }

    // MorphTest.cubicsTest
    #[test]
    fn 同じ形どうしの変形は元の形の曲線を使う() {
        let triangle =
            regular(3, CornerRounding::UNROUNDED).transformed(|p| Point::new(p.x + 0.5, p.y + 0.5));
        let cubics = Morph::new(&triangle, &triangle).as_cubics(0.0);
        assert!(!cubics.is_empty());
        for cubic in &cubics {
            let matched = triangle.cubics().iter().any(|c| {
                c.points
                    .iter()
                    .zip(cubic.points)
                    .all(|(a, b)| (a - b).abs() < 1e-4)
            });
            assert!(matched, "{cubic:?} は元の形の曲線にない");
        }
    }

    #[test]
    fn 変形の両端は始めと終わりの形の輪郭になる() {
        let square = regular(4, CornerRounding::new(0.2, 0.0)).normalized();
        let circle = RoundedPolygon::circle(10).normalized();
        let morph = Morph::new(&square, &circle);
        for (progress, shape) in [(0.0, &square), (1.0, &circle)] {
            let cubics = morph.as_cubics(progress);
            // 長さが 0 に近い曲線を除いた分のずれだけを残してつながる
            for pair in cubics.windows(2) {
                let gap = (pair[0].anchor1() - pair[1].anchor0()).distance();
                assert_close(gap, 0.0, 1e-3, "曲線の間");
            }
            let length: f32 = cubics.iter().map(measure_cubic).sum();
            let expected: f32 = shape.cubics().iter().map(measure_cubic).sum();
            assert_close(length, expected, 0.01 * expected, "輪郭の長さ");
            assert_eq!(
                cubics.first().map(Cubic::anchor0),
                cubics.last().map(Cubic::anchor1)
            );
        }
    }

    #[test]
    fn 正規化した形は_0_から_1_に収まり短い辺を中央に寄せる() {
        let wide =
            regular(4, CornerRounding::UNROUNDED).transformed(|p| Point::new(p.x * 2.0, p.y));
        let [left, top, right, bottom] = wide.normalized().calculate_bounds();
        assert_close(left, 0.0, 1e-5, "左");
        assert_close(right, 1.0, 1e-5, "右");
        assert_close(top, 0.25, 1e-5, "上");
        assert_close(bottom, 0.75, 1e-5, "下");
    }
}
