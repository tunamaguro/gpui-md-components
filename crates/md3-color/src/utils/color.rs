// Ported from material-color-utilities
// (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/utils/color_utils.ts @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2021 Google LLC. Licensed under the Apache License, Version 2.0.

//! HCT / CAM16 以外の色空間の変換
//!
//! ARGB は `0xAARRGGBB` の `u32`、線形 RGB と XYZ の成分は 0〜100 の尺度で扱う

use super::math;

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

pub(crate) const WHITE_POINT_D65: [f64; 3] = [95.047, 100.0, 108.883];

/// RGB の各成分 (下位 8 bit を使う) から不透明な ARGB を作る
pub(crate) fn argb_from_rgb(red: u32, green: u32, blue: u32) -> u32 {
    (255 << 24) | ((red & 255) << 16) | ((green & 255) << 8) | (blue & 255)
}

/// 線形 RGB から ARGB を作る
pub(crate) fn argb_from_linrgb(linrgb: [f64; 3]) -> u32 {
    let r = delinearized(linrgb[0]);
    let g = delinearized(linrgb[1]);
    let b = delinearized(linrgb[2]);
    argb_from_rgb(r, g, b)
}

pub(crate) fn red_from_argb(argb: u32) -> u32 {
    (argb >> 16) & 255
}

pub(crate) fn green_from_argb(argb: u32) -> u32 {
    (argb >> 8) & 255
}

pub(crate) fn blue_from_argb(argb: u32) -> u32 {
    argb & 255
}

/// XYZ から ARGB を作る
pub(crate) fn argb_from_xyz(x: f64, y: f64, z: f64) -> u32 {
    let matrix = XYZ_TO_SRGB;
    let linear_r = matrix[0][0] * x + matrix[0][1] * y + matrix[0][2] * z;
    let linear_g = matrix[1][0] * x + matrix[1][1] * y + matrix[1][2] * z;
    let linear_b = matrix[2][0] * x + matrix[2][1] * y + matrix[2][2] * z;
    let r = delinearized(linear_r);
    let g = delinearized(linear_g);
    let b = delinearized(linear_b);
    argb_from_rgb(r, g, b)
}

/// ARGB から XYZ を求める
pub(crate) fn xyz_from_argb(argb: u32) -> [f64; 3] {
    let r = linearized(red_from_argb(argb));
    let g = linearized(green_from_argb(argb));
    let b = linearized(blue_from_argb(argb));
    math::matrix_multiply([r, g, b], &SRGB_TO_XYZ)
}

/// ARGB から L*a*b* を求める
pub(crate) fn lab_from_argb(argb: u32) -> [f64; 3] {
    let linear_r = linearized(red_from_argb(argb));
    let linear_g = linearized(green_from_argb(argb));
    let linear_b = linearized(blue_from_argb(argb));
    let matrix = SRGB_TO_XYZ;
    let x = matrix[0][0] * linear_r + matrix[0][1] * linear_g + matrix[0][2] * linear_b;
    let y = matrix[1][0] * linear_r + matrix[1][1] * linear_g + matrix[1][2] * linear_b;
    let z = matrix[2][0] * linear_r + matrix[2][1] * linear_g + matrix[2][2] * linear_b;
    let white_point = WHITE_POINT_D65;
    let x_normalized = x / white_point[0];
    let y_normalized = y / white_point[1];
    let z_normalized = z / white_point[2];
    let fx = lab_f(x_normalized);
    let fy = lab_f(y_normalized);
    let fz = lab_f(z_normalized);
    let l = 116.0 * fy - 16.0;
    let a = 500.0 * (fx - fy);
    let b = 200.0 * (fy - fz);
    [l, a, b]
}

/// L* と同じ明度の無彩色の ARGB
pub(crate) fn argb_from_lstar(lstar: f64) -> u32 {
    let y = y_from_lstar(lstar);
    let component = delinearized(y);
    argb_from_rgb(component, component, component)
}

/// ARGB の L*
pub(crate) fn lstar_from_argb(argb: u32) -> f64 {
    let y = xyz_from_argb(argb)[1];
    116.0 * lab_f(y / 100.0) - 16.0
}

/// L* を XYZ の Y に変換する
pub(crate) fn y_from_lstar(lstar: f64) -> f64 {
    100.0 * lab_invf((lstar + 16.0) / 116.0)
}

/// XYZ の Y を L* に変換する
pub(crate) fn lstar_from_y(y: f64) -> f64 {
    lab_f(y / 100.0) * 116.0 - 16.0
}

/// RGB の成分 (0〜255) を線形 RGB (0〜100) に変換する
pub(crate) fn linearized(rgb_component: u32) -> f64 {
    let normalized = f64::from(rgb_component) / 255.0;
    if normalized <= 0.040449936 {
        normalized / 12.92 * 100.0
    } else {
        ((normalized + 0.055) / 1.055).powf(2.4) * 100.0
    }
}

/// 線形 RGB の成分 (0〜100) を RGB の成分 (0〜255) に変換する。範囲外は 0〜255 に丸める
pub(crate) fn delinearized(rgb_component: f64) -> u32 {
    let normalized = rgb_component / 100.0;
    let delinearized = if normalized <= 0.0031308 {
        normalized * 12.92
    } else {
        1.055 * normalized.powf(1.0 / 2.4) - 0.055
    };
    // NaN は `as` で 0 になる。移植元の `clampInt` に NaN が渡ったときも、
    // `argbFromRgb` のビット演算で 0 として扱われる
    math::clamp_int(0, 255, math::round_js(delinearized * 255.0) as i64) as u32
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

// Source: typescript/utils/utils_test.ts @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_から_argb() {
        assert_eq!(argb_from_rgb(255, 255, 255), 0xffffffff);
        assert_eq!(argb_from_rgb(0, 0, 0), 0xff000000);
        assert_eq!(argb_from_rgb(50, 150, 250), 0xff3296fa);
    }

    #[test]
    fn y_と_lstar_の対応() {
        // [L*, Y]
        let cases = [
            (0.0, 0.0),
            (0.9032962, 0.1),
            (4.5164814, 0.5),
            (8.0, 0.8856451),
            (8.9914424, 1.0),
            (37.8424304, 10.0),
            (50.0, 18.4186518),
            (76.0692610, 50.0),
            (90.0, 76.3033539),
            (99.0, 97.4360239),
            (100.0, 100.0),
        ];
        for (lstar, y) in cases {
            assert!((y_from_lstar(lstar) - y).abs() < 0.5e-5, "L* {lstar}");
            assert!((lstar_from_y(y) - lstar).abs() < 0.5e-5, "Y {y}");
        }
        for i in 0..=1000 {
            let v = i as f64 * 0.1;
            assert!((y_from_lstar(lstar_from_y(v)) - v).abs() < 0.5e-8);
            assert!((lstar_from_y(y_from_lstar(v)) - v).abs() < 0.5e-8);
        }
    }
}
