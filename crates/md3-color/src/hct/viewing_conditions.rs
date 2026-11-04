// Ported from material-color-utilities
// (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/hct/viewing_conditions.ts @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2021 Google LLC. Licensed under the Apache License, Version 2.0.

use std::f64::consts::PI;
use std::sync::LazyLock;

use crate::utils::{color, math};

/// CAM16 の観測条件
///
/// 色の見え方は観測環境に左右されるため、CAM16 は色そのものに加えて観測条件を入力に取る。
/// この型は観測条件だけで決まる CAM16 の中間値を保持する
#[derive(Clone, Debug, PartialEq)]
pub struct ViewingConditions {
    pub(crate) n: f64,
    pub(crate) aw: f64,
    pub(crate) nbb: f64,
    pub(crate) ncb: f64,
    pub(crate) c: f64,
    pub(crate) nc: f64,
    pub(crate) rgb_d: [f64; 3],
    pub(crate) fl: f64,
    pub(crate) fl_root: f64,
    pub(crate) z: f64,
}

static STANDARD: LazyLock<ViewingConditions> = LazyLock::new(ViewingConditions::standard);

impl ViewingConditions {
    /// [`ViewingConditions::standard`] の共有インスタンス
    pub(crate) fn standard_ref() -> &'static Self {
        &STANDARD
    }

    /// sRGB に近い既定の観測条件
    ///
    /// D65 の白色点、順応輝度 `(200 / π) * Y(L* = 50) / 100`、背景の L* 50、
    /// 周囲 2.0、照明の補正なし
    pub fn standard() -> Self {
        Self::make(
            color::WHITE_POINT_D65,
            (200.0 / PI) * color::y_from_lstar(50.0) / 100.0,
            50.0,
            2.0,
            false,
        )
    }

    /// 物理的な観測条件から作る
    ///
    /// - `white_point`: XYZ 色空間で測った白色点
    /// - `adapting_luminance`: 順応視野の輝度。lux に 0.0586 を掛けた値
    /// - `background_lstar`: 色の周囲の明るさ (L*)
    /// - `surround`: 周囲の照明。0 は真っ暗、1.0 は薄暗い部屋、2.0 は色と周囲の照明が同じ
    /// - `discounting_illuminant`: 目が照明の色を補正するか。ディスプレイのような自発光体では `false`
    pub fn make(
        white_point: [f64; 3],
        adapting_luminance: f64,
        background_lstar: f64,
        surround: f64,
        discounting_illuminant: bool,
    ) -> Self {
        let xyz = white_point;
        let r_w = xyz[0] * 0.401288 + xyz[1] * 0.650173 + xyz[2] * -0.051461;
        let g_w = xyz[0] * -0.250268 + xyz[1] * 1.204414 + xyz[2] * 0.045854;
        let b_w = xyz[0] * -0.002079 + xyz[1] * 0.048952 + xyz[2] * 0.953127;
        let f = 0.8 + surround / 10.0;
        let c = if f >= 0.9 {
            math::lerp(0.59, 0.69, (f - 0.9) * 10.0)
        } else {
            math::lerp(0.525, 0.59, (f - 0.8) * 10.0)
        };
        let d = if discounting_illuminant {
            1.0
        } else {
            f * (1.0 - (1.0 / 3.6) * ((-adapting_luminance - 42.0) / 92.0).exp())
        };
        let d = d.clamp(0.0, 1.0);
        let nc = f;
        let rgb_d = [
            d * (100.0 / r_w) + 1.0 - d,
            d * (100.0 / g_w) + 1.0 - d,
            d * (100.0 / b_w) + 1.0 - d,
        ];
        let k = 1.0 / (5.0 * adapting_luminance + 1.0);
        let k4 = k * k * k * k;
        let k4_f = 1.0 - k4;
        let fl = k4 * adapting_luminance + 0.1 * k4_f * k4_f * (5.0 * adapting_luminance).cbrt();
        let n = color::y_from_lstar(background_lstar) / white_point[1];
        let z = 1.48 + n.sqrt();
        let nbb = 0.725 / n.powf(0.2);
        let ncb = nbb;
        let rgb_a_factors = [
            ((fl * rgb_d[0] * r_w) / 100.0).powf(0.42),
            ((fl * rgb_d[1] * g_w) / 100.0).powf(0.42),
            ((fl * rgb_d[2] * b_w) / 100.0).powf(0.42),
        ];
        let rgb_a = [
            (400.0 * rgb_a_factors[0]) / (rgb_a_factors[0] + 27.13),
            (400.0 * rgb_a_factors[1]) / (rgb_a_factors[1] + 27.13),
            (400.0 * rgb_a_factors[2]) / (rgb_a_factors[2] + 27.13),
        ];
        let aw = (2.0 * rgb_a[0] + rgb_a[1] + 0.05 * rgb_a[2]) * nbb;
        Self {
            n,
            aw,
            nbb,
            ncb,
            c,
            nc,
            rgb_d,
            fl,
            fl_root: fl.powf(0.25),
            z,
        }
    }
}

impl Default for ViewingConditions {
    fn default() -> Self {
        Self::standard()
    }
}

// Source: typescript/hct/hct_test.ts @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 既定の観測条件() {
        let vc = ViewingConditions::standard();
        let cases = [
            (vc.n, 0.184),
            (vc.aw, 29.981),
            (vc.nbb, 1.017),
            (vc.ncb, 1.017),
            (vc.c, 0.69),
            (vc.nc, 1.0),
            (vc.rgb_d[0], 1.021),
            (vc.rgb_d[1], 0.986),
            (vc.rgb_d[2], 0.934),
            (vc.fl, 0.388),
            (vc.fl_root, 0.789),
            (vc.z, 1.909),
        ];
        for (actual, expected) in cases {
            assert!((actual - expected).abs() < 0.5e-3, "{actual} != {expected}");
        }
    }
}
