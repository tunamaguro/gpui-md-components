// Ported from Material Web (https://github.com/material-components/material-web)
// Source: elevation/internal/_elevation.scss (styles), tokens/_md-comp-elevation.scss,
//         tokens/_md-sys-elevation.scss
//         @ de5470f7ed6a653457ca8f5cc0e7f7602c7a8391
// Copyright 2022 Google LLC. Licensed under the Apache License, Version 2.0.

//! 高さの段階に応じた影
//!
//! Compose は影の描画をプラットフォームに任せて値を持たないため、Material Web の影を転記している。
//! 影は位置の決まった key shadow と、広がる ambient shadow の 2 つを重ねる。
//! 色はどちらも [`SHADOW_COLOR`] で、それぞれの不透明度を掛けて描く。
//! Material Web の CSS px は [`Dp`] として扱う

use md3_color::ColorRole;

use crate::Dp;
use crate::elevation::Elevation;

/// 影の色。`md.comp.elevation.shadow-color` (`md.sys.color.shadow`)
pub const SHADOW_COLOR: ColorRole = ColorRole::Shadow;

/// key shadow の不透明度
pub const KEY_OPACITY: f32 = 0.3;

/// ambient shadow の不透明度
pub const AMBIENT_OPACITY: f32 = 0.15;

/// 1 つの影。CSS の `box-shadow: 0 y blur spread` に当たり、横方向のずれはない
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    /// 下方向へのずれ
    pub y: Dp,
    /// ぼかしの半径
    pub blur: Dp,
    /// 広がり
    pub spread: Dp,
    /// 不透明度
    pub opacity: f32,
}

/// 高さの段階に応じた 2 つの影
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ElevationShadow {
    /// key shadow
    pub key: Shadow,
    /// ambient shadow
    pub ambient: Shadow,
}

/// 段階 `elevation` の影を求める
pub fn shadow(elevation: Elevation) -> ElevationShadow {
    let level = f32::from(elevation.level());
    // CSS の clamp(0, level - offset, max)
    let step = |offset: f32, max: f32| (level - offset).clamp(0.0, max);
    ElevationShadow {
        key: Shadow {
            y: Dp(step(0.0, 1.0) + step(3.0, 1.0) + 2.0 * step(4.0, 1.0)),
            blur: Dp(2.0 * step(0.0, 1.0) + step(2.0, 1.0) + step(4.0, 1.0)),
            spread: Dp(0.0),
            opacity: KEY_OPACITY,
        },
        ambient: Shadow {
            y: Dp(step(0.0, 1.0) + step(1.0, 1.0) + 2.0 * step(2.0, 3.0)),
            blur: Dp(3.0 * step(0.0, 2.0) + 2.0 * step(2.0, 3.0)),
            spread: Dp(step(0.0, 4.0) + 2.0 * step(4.0, 1.0)),
            opacity: AMBIENT_OPACITY,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 各段階の影は移植元のコメントの値と一致する() {
        // _elevation.scss のコメントにある段階ごとの box-shadow (y, blur, spread)
        let key = [(0., 0.), (1., 2.), (1., 2.), (1., 3.), (2., 3.), (4., 4.)];
        let ambient = [
            (0., 0., 0.),
            (1., 3., 1.),
            (2., 6., 2.),
            (4., 8., 3.),
            (6., 10., 4.),
            (8., 12., 6.),
        ];
        for (level, elevation) in Elevation::ALL.into_iter().enumerate() {
            let s = shadow(elevation);
            assert_eq!(
                (s.key.y.0, s.key.blur.0, s.key.spread.0),
                (key[level].0, key[level].1, 0.0)
            );
            assert_eq!(
                (s.ambient.y.0, s.ambient.blur.0, s.ambient.spread.0),
                ambient[level]
            );
        }
    }
}
