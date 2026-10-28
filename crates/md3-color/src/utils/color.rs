// Ported from material-color-utilities (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/utils/color_utils.ts @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2021 Google LLC. Licensed under the Apache License, Version 2.0.

//! 色空間の変換（sRGB / 線形 RGB / XYZ / L*a*b* / L*）。ARGB は `u32` で扱う。

use super::math::{clamp_int, js_round, matrix_multiply};

const SRGB_TO_XYZ: [[f64; 3]; 3] = [
    [0.41233895, 0.35762064, 0.18051042],
    [0.2126, 0.7152, 0.0722],
    [0.01932141, 0.11916382, 0.95034478],
];

const XYZ_TO_SRGB: [[f64; 3]; 3] = [
    [
        3.2413774792388685,
        -1.5376652402851851,
        -0.49885366846268053,
    ],
    [-0.9691452513005321, 1.8758853451067872, 0.04156585616912061],
    [
        0.05562093689691305,
        -0.20395524564742123,
        1.0571799111220335,
    ],
];

/// D65 の白色点（XYZ）。
pub(crate) const WHITE_POINT_D65: [f64; 3] = [95.047, 100.0, 108.883];

/// 8bit の RGB から不透明な ARGB を作る。
pub(crate) fn argb_from_rgb(red: u8, green: u8, blue: u8) -> u32 {
    0xff00_0000 | (u32::from(red) << 16) | (u32::from(green) << 8) | u32::from(blue)
}

/// 線形 RGB（0〜100）から ARGB を作る。
pub(crate) fn argb_from_linrgb(linrgb: [f64; 3]) -> u32 {
    argb_from_rgb(
        delinearized(linrgb[0]),
        delinearized(linrgb[1]),
        delinearized(linrgb[2]),
    )
}

pub(crate) fn red_from_argb(argb: u32) -> u8 {
    ((argb >> 16) & 0xff) as u8
}

pub(crate) fn green_from_argb(argb: u32) -> u8 {
    ((argb >> 8) & 0xff) as u8
}

pub(crate) fn blue_from_argb(argb: u32) -> u8 {
    (argb & 0xff) as u8
}

/// XYZ から ARGB を作る。
pub(crate) fn argb_from_xyz(x: f64, y: f64, z: f64) -> u32 {
    let m = &XYZ_TO_SRGB;
    let linear_r = m[0][0] * x + m[0][1] * y + m[0][2] * z;
    let linear_g = m[1][0] * x + m[1][1] * y + m[1][2] * z;
    let linear_b = m[2][0] * x + m[2][1] * y + m[2][2] * z;
    argb_from_rgb(
        delinearized(linear_r),
        delinearized(linear_g),
        delinearized(linear_b),
    )
}

/// ARGB を XYZ にする。
pub(crate) fn xyz_from_argb(argb: u32) -> [f64; 3] {
    let r = linearized(red_from_argb(argb));
    let g = linearized(green_from_argb(argb));
    let b = linearized(blue_from_argb(argb));
    matrix_multiply([r, g, b], &SRGB_TO_XYZ)
}

/// ARGB を L*a*b* にする。
pub(crate) fn lab_from_argb(argb: u32) -> [f64; 3] {
    let linear_r = linearized(red_from_argb(argb));
    let linear_g = linearized(green_from_argb(argb));
    let linear_b = linearized(blue_from_argb(argb));
    let m = &SRGB_TO_XYZ;
    let x = m[0][0] * linear_r + m[0][1] * linear_g + m[0][2] * linear_b;
    let y = m[1][0] * linear_r + m[1][1] * linear_g + m[1][2] * linear_b;
    let z = m[2][0] * linear_r + m[2][1] * linear_g + m[2][2] * linear_b;
    let fx = lab_f(x / WHITE_POINT_D65[0]);
    let fy = lab_f(y / WHITE_POINT_D65[1]);
    let fz = lab_f(z / WHITE_POINT_D65[2]);
    let l = 116.0 * fy - 16.0;
    let a = 500.0 * (fx - fy);
    let b = 200.0 * (fy - fz);
    [l, a, b]
}

/// L* から無彩色の ARGB を作る。
pub(crate) fn argb_from_lstar(lstar: f64) -> u32 {
    let y = y_from_lstar(lstar);
    let component = delinearized(y);
    argb_from_rgb(component, component, component)
}

/// ARGB の L*。
pub(crate) fn lstar_from_argb(argb: u32) -> f64 {
    let y = xyz_from_argb(argb)[1];
    116.0 * lab_f(y / 100.0) - 16.0
}

/// L* から相対輝度 Y（0〜100）。
pub(crate) fn y_from_lstar(lstar: f64) -> f64 {
    100.0 * lab_invf((lstar + 16.0) / 116.0)
}

/// 相対輝度 Y（0〜100）から L*。
pub(crate) fn lstar_from_y(y: f64) -> f64 {
    lab_f(y / 100.0) * 116.0 - 16.0
}

/// sRGB の成分（0〜255）を線形 RGB（0〜100）にする。
pub(crate) fn linearized(rgb_component: u8) -> f64 {
    let normalized = f64::from(rgb_component) / 255.0;
    if normalized <= 0.040449936 {
        normalized / 12.92 * 100.0
    } else {
        ((normalized + 0.055) / 1.055).powf(2.4) * 100.0
    }
}

/// 線形 RGB（0〜100）を sRGB の成分（0〜255）にする。範囲外は丸める。
pub(crate) fn delinearized(rgb_component: f64) -> u8 {
    let normalized = rgb_component / 100.0;
    let delinearized = if normalized <= 0.0031308 {
        normalized * 12.92
    } else {
        1.055 * normalized.powf(1.0 / 2.4) - 0.055
    };
    let rounded = js_round(delinearized * 255.0);
    // NaN は 0 として扱う（`as` は NaN を 0 にし、範囲外は飽和させる）。
    clamp_int(0, 255, rounded as i64) as u8
}

fn lab_f(t: f64) -> f64 {
    let e = 216.0 / 24389.0;
    let kappa = 24389.0 / 27.0;
    if t > e {
        t.powf(1.0 / 3.0)
    } else {
        (kappa * t + 16.0) / 116.0
    }
}

fn lab_invf(ft: f64) -> f64 {
    let e = 216.0 / 24389.0;
    let kappa = 24389.0 / 27.0;
    let ft3 = ft * ft * ft;
    if ft3 > e {
        ft3
    } else {
        (116.0 * ft - 16.0) / kappa
    }
}

#[cfg(test)]
mod tests {
    // Ported from typescript/utils/utils_test.ts

    use super::*;

    fn assert_close(actual: f64, expected: f64, digits: i32) {
        // jasmine の toBeCloseTo(expected, digits) と同じ判定。
        let tolerance = 10f64.powi(-digits) / 2.0;
        assert!(
            (actual - expected).abs() < tolerance,
            "{actual} is not close to {expected} ({digits} digits)"
        );
    }

    #[test]
    fn argb_from_rgb_returns_correct_values() {
        assert_eq!(argb_from_rgb(255, 255, 255), 0xffff_ffff);
        assert_eq!(argb_from_rgb(0, 0, 0), 0xff00_0000);
        assert_eq!(argb_from_rgb(50, 150, 250), 0xff32_96fa);
    }

    #[test]
    fn y_from_lstar_satisfies_given_values() {
        let cases = [
            (0.0, 0.0),
            (0.1, 0.0110705),
            (0.2, 0.0221411),
            (0.3, 0.0332116),
            (0.4, 0.0442822),
            (0.5, 0.0553528),
            (1.0, 0.1107056),
            (2.0, 0.2214112),
            (3.0, 0.3321169),
            (4.0, 0.4428225),
            (5.0, 0.5535282),
            (8.0, 0.8856451),
            (10.0, 1.1260199),
            (15.0, 1.9085832),
            (20.0, 2.9890524),
            (25.0, 4.4154767),
            (30.0, 6.2359055),
            (40.0, 11.2509737),
            (50.0, 18.4186518),
            (60.0, 28.1233342),
            (70.0, 40.7494157),
            (80.0, 56.6812907),
            (90.0, 76.3033539),
            (95.0, 87.6183294),
            (99.0, 97.4360239),
            (100.0, 100.0),
        ];
        for (lstar, y) in cases {
            assert_close(y_from_lstar(lstar), y, 5);
        }
    }

    #[test]
    fn y_from_lstar_is_inverse_of_lstar_from_y() {
        let mut y = 0.0;
        while y <= 100.0 {
            assert_close(y_from_lstar(lstar_from_y(y)), y, 8);
            y += 0.1;
        }
    }

    #[test]
    fn lstar_from_y_satisfies_given_values() {
        let cases = [
            (0.0, 0.0),
            (0.1, 0.9032962),
            (0.2, 1.8065925),
            (0.3, 2.7098888),
            (0.4, 3.6131851),
            (0.5, 4.5164814),
            (0.8856451, 8.0),
            (1.0, 8.9914424),
            (2.0, 15.4872443),
            (3.0, 20.0438970),
            (4.0, 23.6714419),
            (5.0, 26.7347653),
            (10.0, 37.8424304),
            (15.0, 45.6341970),
            (20.0, 51.8372115),
            (25.0, 57.0754208),
            (30.0, 61.6542222),
            (40.0, 69.4695307),
            (50.0, 76.0692610),
            (60.0, 81.8381891),
            (70.0, 86.9968642),
            (80.0, 91.6848609),
            (90.0, 95.9967686),
            (95.0, 98.0335184),
            (99.0, 99.6120372),
            (100.0, 100.0),
        ];
        for (y, lstar) in cases {
            assert_close(lstar_from_y(y), lstar, 5);
        }
    }

    #[test]
    fn lstar_from_y_is_inverse_of_y_from_lstar() {
        let mut lstar = 0.0;
        while lstar <= 100.0 {
            assert_close(lstar_from_y(y_from_lstar(lstar)), lstar, 8);
            lstar += 0.1;
        }
    }
}
