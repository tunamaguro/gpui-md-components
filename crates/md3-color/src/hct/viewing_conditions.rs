// Ported from material-color-utilities (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/hct/viewing_conditions.ts @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2021 Google LLC. Licensed under the Apache License, Version 2.0.

use std::f64::consts::PI;
use std::sync::OnceLock;

use crate::utils::{color, math};

/// CAM16 の観察条件。色の見え方に影響する環境の情報。
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

impl ViewingConditions {
    /// sRGB 相当の既定の観察条件（中程度の灰色の背景、平均的な周囲）。
    pub fn standard() -> &'static Self {
        static DEFAULT: OnceLock<ViewingConditions> = OnceLock::new();
        DEFAULT.get_or_init(|| {
            Self::make(
                color::WHITE_POINT_D65,
                (200.0 / PI) * color::y_from_lstar(50.0) / 100.0,
                50.0,
                2.0,
                false,
            )
        })
    }

    /// 観察条件を作る。
    ///
    /// - `white_point`: 白色点の XYZ
    /// - `adapting_luminance`: 順応輝度（lux）
    /// - `background_lstar`: 背景の L*
    /// - `surround`: 周囲の明るさ（0 = 暗い、1 = 薄暗い、2 = 平均）
    /// - `discounting_illuminant`: 照明の色を差し引くか
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
        }
        .clamp(0.0, 1.0);
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

#[cfg(test)]
mod tests {
    // Ported from typescript/hct/hct_test.ts ("viewing conditions")

    use super::*;

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 0.0005, "{actual} != {expected}");
    }

    #[test]
    fn default() {
        let vc = ViewingConditions::standard();
        assert_close(vc.n, 0.184);
        assert_close(vc.aw, 29.981);
        assert_close(vc.nbb, 1.017);
        assert_close(vc.ncb, 1.017);
        assert_close(vc.c, 0.69);
        assert_close(vc.nc, 1.0);
        assert_close(vc.rgb_d[0], 1.021);
        assert_close(vc.rgb_d[1], 0.986);
        assert_close(vc.rgb_d[2], 0.934);
        assert_close(vc.fl, 0.388);
        assert_close(vc.fl_root, 0.789);
        assert_close(vc.z, 1.909);
    }
}
