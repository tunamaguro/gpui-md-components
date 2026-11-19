// 形の定義は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/MaterialShapes.kt
//         (`MaterialShapes` の `circle`、`softBurst`、`cookie9`、`pentagon`、`pill`、`sunny`、`cookie4`、
//         `oval`、`customPolygon`、`doRepeat`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2024 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
//
// 移すのはローディングインジケーターが使う形だけ

use super::shapes::{CornerRounding, Point, RoundedPolygon};

const CENTER: Point = Point::new(0.5, 0.5);

/// ローディングインジケーターが使う MD3 の形。どれも 0〜1 の正方形に収めてある
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MaterialShape {
    Circle,
    SoftBurst,
    Cookie9Sided,
    Pentagon,
    Pill,
    Sunny,
    Cookie4Sided,
    Oval,
}

impl MaterialShape {
    pub(super) fn polygon(self) -> RoundedPolygon {
        let polygon = match self {
            MaterialShape::Circle => RoundedPolygon::circle(10),
            MaterialShape::SoftBurst => custom_polygon(
                &[
                    (Point::new(0.193, 0.277), CornerRounding::new(0.053, 0.0)),
                    (Point::new(0.176, 0.055), CornerRounding::new(0.053, 0.0)),
                ],
                10,
                false,
            ),
            MaterialShape::Cookie9Sided => {
                RoundedPolygon::star(9, 0.8, CornerRounding::new(0.5, 0.0))
                    .transformed(rotate(-90.0))
            }
            MaterialShape::Pentagon => custom_polygon(
                &[
                    (Point::new(0.500, -0.009), CornerRounding::new(0.172, 0.0)),
                    (Point::new(1.030, 0.365), CornerRounding::new(0.164, 0.0)),
                    (Point::new(0.828, 0.970), CornerRounding::new(0.169, 0.0)),
                ],
                1,
                true,
            ),
            MaterialShape::Pill => custom_polygon(
                &[
                    (Point::new(0.961, 0.039), CornerRounding::new(0.426, 0.0)),
                    (Point::new(1.001, 0.428), CornerRounding::UNROUNDED),
                    (Point::new(1.000, 0.609), CornerRounding::new(1.0, 0.0)),
                ],
                2,
                true,
            ),
            MaterialShape::Sunny => RoundedPolygon::star(8, 0.8, CornerRounding::new(0.15, 0.0)),
            MaterialShape::Cookie4Sided => custom_polygon(
                &[
                    (Point::new(1.237, 1.236), CornerRounding::new(0.258, 0.0)),
                    (Point::new(0.500, 0.918), CornerRounding::new(0.233, 0.0)),
                ],
                4,
                false,
            ),
            MaterialShape::Oval => RoundedPolygon::circle(8)
                .transformed(|p| Point::new(p.x, p.y * 0.64))
                .transformed(rotate(-45.0)),
        };
        polygon.normalized()
    }
}

/// 原点のまわりに `degrees` 度回す変換 (Compose の `Matrix.rotateZ`)
pub(super) fn rotate(degrees: f32) -> impl Fn(Point) -> Point {
    let (sin, cos) = degrees.to_radians().sin_cos();
    move |p| Point::new(p.x * cos - p.y * sin, p.x * sin + p.y * cos)
}

/// `points` を中心 (0.5, 0.5) のまわりに `reps` 回繰り返した多角形。
/// `mirroring` では、繰り返しの区間ごとに点の並びを折り返す
fn custom_polygon(
    points: &[(Point, CornerRounding)],
    reps: usize,
    mirroring: bool,
) -> RoundedPolygon {
    let mut vertices = Vec::new();
    if mirroring {
        let angle = |p: Point| (p.y - CENTER.y).atan2(p.x - CENTER.x).to_degrees();
        let first_angle = points.first().map_or(0.0, |(p, _)| angle(*p));
        let actual_reps = reps * 2;
        let section_angle = 360.0 / actual_reps as f32;
        for it in 0..actual_reps {
            for index in 0..points.len() {
                let i = if it % 2 == 0 {
                    index
                } else {
                    points.len() - 1 - index
                };
                if i > 0 || it % 2 == 0 {
                    let (point, rounding) = points[i];
                    let a = section_angle * it as f32
                        + if it % 2 == 0 {
                            angle(point)
                        } else {
                            section_angle - angle(point) + 2.0 * first_angle
                        };
                    let distance =
                        ((point.x - CENTER.x).powi(2) + (point.y - CENTER.y).powi(2)).sqrt();
                    let (sin, cos) = a.to_radians().sin_cos();
                    vertices.push((
                        Point::new(cos * distance + CENTER.x, sin * distance + CENTER.y),
                        rounding,
                    ));
                }
            }
        }
    } else {
        let np = points.len();
        for it in 0..np * reps {
            let (point, rounding) = points[it % np];
            let r = rotate((it / np) as f32 * 360.0 / reps as f32);
            let rotated = r(Point::new(point.x - CENTER.x, point.y - CENTER.y));
            vertices.push((
                Point::new(rotated.x + CENTER.x, rotated.y + CENTER.y),
                rounding,
            ));
        }
    }
    RoundedPolygon::from_vertices(&vertices, CENTER)
}
