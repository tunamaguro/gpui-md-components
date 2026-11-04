//! モーション
//!
//! 時間とイージングの固定値に加え、コンポーネントのトークンが [`MotionSchemeKey`] で参照するバネを持つ。
//! キーは [`MotionScheme`] (Standard / Expressive) ごとに異なるバネに解決される
//!
//! [`MotionScheme::spring`] の対応は Jetpack Compose Material3 の `MotionScheme`
//! (`compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/MotionScheme.kt`
//! @ 2a941dd11aabd9bb7d2016ed159764b220117c20) に従う

use crate::generated::{expressive_motion, standard_motion};

pub use crate::generated::motion::*;
pub use crate::generated::motion_scheme_key::MotionSchemeKey;

/// 3 次ベジェ曲線のイージング。始点 (0, 0) と終点 (1, 1) の間の 2 つの制御点を持つ
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubicBezier {
    /// 1 つ目の制御点の x
    pub x1: f32,
    /// 1 つ目の制御点の y
    pub y1: f32,
    /// 2 つ目の制御点の x
    pub x2: f32,
    /// 2 つ目の制御点の y
    pub y2: f32,
}

impl CubicBezier {
    /// 2 つの制御点からイージングを作る
    pub const fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }
}

/// バネのアニメーションの値
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    /// 減衰比。1 で臨界減衰、1 未満で行き過ぎて戻る
    pub damping_ratio: f32,
    /// 剛性
    pub stiffness: f32,
}

/// モーションの方式
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MotionScheme {
    /// 標準
    #[default]
    Standard,
    /// Expressive
    Expressive,
}

impl MotionScheme {
    /// この方式で `key` に対応するバネ
    pub const fn spring(self, key: MotionSchemeKey) -> Spring {
        let (damping_ratio, stiffness) = match self {
            Self::Standard => match key {
                MotionSchemeKey::DefaultSpatial => (
                    standard_motion::SPRING_DEFAULT_SPATIAL_DAMPING,
                    standard_motion::SPRING_DEFAULT_SPATIAL_STIFFNESS,
                ),
                MotionSchemeKey::FastSpatial => (
                    standard_motion::SPRING_FAST_SPATIAL_DAMPING,
                    standard_motion::SPRING_FAST_SPATIAL_STIFFNESS,
                ),
                MotionSchemeKey::SlowSpatial => (
                    standard_motion::SPRING_SLOW_SPATIAL_DAMPING,
                    standard_motion::SPRING_SLOW_SPATIAL_STIFFNESS,
                ),
                MotionSchemeKey::DefaultEffects => (
                    standard_motion::SPRING_DEFAULT_EFFECTS_DAMPING,
                    standard_motion::SPRING_DEFAULT_EFFECTS_STIFFNESS,
                ),
                MotionSchemeKey::FastEffects => (
                    standard_motion::SPRING_FAST_EFFECTS_DAMPING,
                    standard_motion::SPRING_FAST_EFFECTS_STIFFNESS,
                ),
                MotionSchemeKey::SlowEffects => (
                    standard_motion::SPRING_SLOW_EFFECTS_DAMPING,
                    standard_motion::SPRING_SLOW_EFFECTS_STIFFNESS,
                ),
            },
            Self::Expressive => match key {
                MotionSchemeKey::DefaultSpatial => (
                    expressive_motion::SPRING_DEFAULT_SPATIAL_DAMPING,
                    expressive_motion::SPRING_DEFAULT_SPATIAL_STIFFNESS,
                ),
                MotionSchemeKey::FastSpatial => (
                    expressive_motion::SPRING_FAST_SPATIAL_DAMPING,
                    expressive_motion::SPRING_FAST_SPATIAL_STIFFNESS,
                ),
                MotionSchemeKey::SlowSpatial => (
                    expressive_motion::SPRING_SLOW_SPATIAL_DAMPING,
                    expressive_motion::SPRING_SLOW_SPATIAL_STIFFNESS,
                ),
                MotionSchemeKey::DefaultEffects => (
                    expressive_motion::SPRING_DEFAULT_EFFECTS_DAMPING,
                    expressive_motion::SPRING_DEFAULT_EFFECTS_STIFFNESS,
                ),
                MotionSchemeKey::FastEffects => (
                    expressive_motion::SPRING_FAST_EFFECTS_DAMPING,
                    expressive_motion::SPRING_FAST_EFFECTS_STIFFNESS,
                ),
                MotionSchemeKey::SlowEffects => (
                    expressive_motion::SPRING_SLOW_EFFECTS_DAMPING,
                    expressive_motion::SPRING_SLOW_EFFECTS_STIFFNESS,
                ),
            },
        };
        Spring {
            damping_ratio,
            stiffness,
        }
    }
}
