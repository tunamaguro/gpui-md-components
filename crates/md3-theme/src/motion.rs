//! モーションスキーム（バネの指定の組）と、gpui のバネへの変換。

use std::time::Duration;

use gpui::{SpringConfig, SpringState};
use md3_tokens::motion::{self as tokens, SpringSpec};

/// モーションスキーム。部品のバネの速さと弾み方の組を決める。
///
/// Compose の `MotionScheme.standard()` / `MotionScheme.expressive()` に当たる。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MotionScheme {
    /// 落ち着いた動き（行き過ぎがほとんどない）。
    #[default]
    Standard,
    /// 弾む動き（M3 Expressive）。位置・大きさ・形の変化が目標を少し行き過ぎて戻る。
    Expressive,
}

/// モーションスキームの中のバネの種類。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MotionToken {
    /// 位置・大きさ・形の変化（標準の速さ）。
    DefaultSpatial,
    /// 位置・大きさ・形の変化（小さな部品の速い変化。押下時の形の変化など）。
    FastSpatial,
    /// 位置・大きさ・形の変化（画面全体のような大きな変化）。
    SlowSpatial,
    /// 色・不透明度の変化（標準の速さ）。
    DefaultEffects,
    /// 色・不透明度の変化（速い）。
    FastEffects,
    /// 色・不透明度の変化（遅い）。
    SlowEffects,
}

impl MotionScheme {
    /// すべてのスキーム。
    pub const ALL: [MotionScheme; 2] = [MotionScheme::Standard, MotionScheme::Expressive];

    /// バネの指定（減衰比と剛性）。
    pub const fn spring_spec(self, token: MotionToken) -> SpringSpec {
        match (self, token) {
            (MotionScheme::Standard, MotionToken::DefaultSpatial) => {
                tokens::STANDARD_SPRING_DEFAULT_SPATIAL
            }
            (MotionScheme::Standard, MotionToken::FastSpatial) => {
                tokens::STANDARD_SPRING_FAST_SPATIAL
            }
            (MotionScheme::Standard, MotionToken::SlowSpatial) => {
                tokens::STANDARD_SPRING_SLOW_SPATIAL
            }
            (MotionScheme::Standard, MotionToken::DefaultEffects) => {
                tokens::STANDARD_SPRING_DEFAULT_EFFECTS
            }
            (MotionScheme::Standard, MotionToken::FastEffects) => {
                tokens::STANDARD_SPRING_FAST_EFFECTS
            }
            (MotionScheme::Standard, MotionToken::SlowEffects) => {
                tokens::STANDARD_SPRING_SLOW_EFFECTS
            }
            (MotionScheme::Expressive, MotionToken::DefaultSpatial) => {
                tokens::EXPRESSIVE_SPRING_DEFAULT_SPATIAL
            }
            (MotionScheme::Expressive, MotionToken::FastSpatial) => {
                tokens::EXPRESSIVE_SPRING_FAST_SPATIAL
            }
            (MotionScheme::Expressive, MotionToken::SlowSpatial) => {
                tokens::EXPRESSIVE_SPRING_SLOW_SPATIAL
            }
            (MotionScheme::Expressive, MotionToken::DefaultEffects) => {
                tokens::EXPRESSIVE_SPRING_DEFAULT_EFFECTS
            }
            (MotionScheme::Expressive, MotionToken::FastEffects) => {
                tokens::EXPRESSIVE_SPRING_FAST_EFFECTS
            }
            (MotionScheme::Expressive, MotionToken::SlowEffects) => {
                tokens::EXPRESSIVE_SPRING_SLOW_EFFECTS
            }
        }
    }

    /// gpui のバネ（`with_spring` に渡す [`SpringConfig`]）。
    pub fn spring(self, token: MotionToken) -> SpringConfig {
        to_spring_config(self.spring_spec(token))
    }
}

/// 減衰比と剛性の指定を gpui のバネ（質量 1、減衰係数 = 2 × 減衰比 × √剛性）に変える。
pub fn to_spring_config(spec: SpringSpec) -> SpringConfig {
    SpringConfig::new(
        spec.stiffness,
        2.0 * spec.damping_ratio * spec.stiffness.sqrt(),
        1.0,
    )
}

/// 静止した 0 から目標 1 に向かうバネが、`fraction` に初めて届くまでの時間。
///
/// 押下の形の変化を、短いクリックでも見える所まで進めてから戻すのに使う
/// （Compose の ButtonGroup が押下を離した後、値が 0.75 を超えるまで待つのと同じ）。
/// 届かない（減衰が強すぎる、または `fraction` が 1 を超える）なら、落ち着くまでの時間を返す。
pub fn time_to_reach(config: SpringConfig, fraction: f32) -> Duration {
    const STEP: f32 = 1.0 / 240.0;
    let limit = config.settle_time(
        SpringState {
            position: 0.0,
            velocity: 0.0,
        },
        1.0,
        0.001,
    );
    let mut state = SpringState {
        position: 0.0,
        velocity: 0.0,
    };
    let mut elapsed = 0.0_f32;
    while state.position < fraction && Duration::from_secs_f32(elapsed) < limit {
        state = config.step(state, 1.0, STEP);
        elapsed += STEP;
    }
    Duration::from_secs_f32(elapsed).min(limit)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Compose の SpringSimulation.updateValues（減衰比 < 1 / = 1 の解析解）を、
    // gpui のバネとの突き合わせのために写したもの。
    // Ported from Jetpack Compose (https://android.googlesource.com/platform/frameworks/support)
    // Source: compose/animation/animation-core/src/commonMain/kotlin/androidx/compose/animation/core/SpringSimulation.kt
    //         @ b8256a52
    // Copyright 2019 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
    fn compose_spring(spec: SpringSpec, displacement: f64, seconds: f64) -> f64 {
        let natural_freq = f64::from(spec.stiffness).sqrt();
        let damping_ratio = f64::from(spec.damping_ratio);
        let r = -damping_ratio * natural_freq;
        if damping_ratio < 1.0 {
            let damped_freq = natural_freq * (1.0 - damping_ratio * damping_ratio).sqrt();
            let cos_coeff = displacement;
            let sin_coeff = (1.0 / damped_freq) * (-r * displacement);
            let dfdt = damped_freq * seconds;
            (r * seconds).exp() * (cos_coeff * dfdt.cos() + sin_coeff * dfdt.sin())
        } else {
            let coeff_a = displacement;
            let coeff_b = natural_freq * displacement;
            (coeff_a + coeff_b * seconds) * (-natural_freq * seconds).exp()
        }
    }

    const TOKENS: [MotionToken; 6] = [
        MotionToken::DefaultSpatial,
        MotionToken::FastSpatial,
        MotionToken::SlowSpatial,
        MotionToken::DefaultEffects,
        MotionToken::FastEffects,
        MotionToken::SlowEffects,
    ];

    #[test]
    fn gpui_spring_matches_compose_spring() {
        for scheme in MotionScheme::ALL {
            for token in TOKENS {
                let spec = scheme.spring_spec(token);
                let config = scheme.spring(token);
                let (_, ratio) = config.canonical();
                assert!((ratio - spec.damping_ratio).abs() < 1e-4);
                for millis in [8_u32, 16, 50, 100, 200, 400] {
                    let seconds = millis as f32 / 1000.0;
                    let state = config.step(
                        SpringState {
                            position: 0.0,
                            velocity: 0.0,
                        },
                        1.0,
                        seconds,
                    );
                    let expected = 1.0 - compose_spring(spec, 1.0, f64::from(seconds));
                    assert!(
                        (f64::from(state.position) - expected).abs() < 1e-3,
                        "{scheme:?} {token:?} {millis}ms: {} != {expected}",
                        state.position
                    );
                }
            }
        }
    }

    #[test]
    fn expressive_spatial_springs_overshoot_and_effects_do_not() {
        let peak = |config: SpringConfig| {
            let mut state = SpringState {
                position: 0.0,
                velocity: 0.0,
            };
            let mut peak = 0.0_f32;
            for _ in 0..1000 {
                state = config.step(state, 1.0, 1.0 / 240.0);
                peak = peak.max(state.position);
            }
            peak
        };
        let expressive = MotionScheme::Expressive;
        assert!(peak(expressive.spring(MotionToken::FastSpatial)) > 1.05);
        assert!(peak(expressive.spring(MotionToken::FastEffects)) <= 1.0 + 1e-3);
    }

    #[test]
    fn time_to_reach_is_short_and_ordered() {
        let fast = time_to_reach(
            MotionScheme::Expressive.spring(MotionToken::FastSpatial),
            0.75,
        );
        let slow = time_to_reach(
            MotionScheme::Expressive.spring(MotionToken::SlowSpatial),
            0.75,
        );
        assert!(
            fast > Duration::ZERO && fast < Duration::from_millis(100),
            "{fast:?}"
        );
        assert!(slow > fast);
    }
}
