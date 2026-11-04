// Ported from material-color-utilities
// (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/utils/math_utils.ts @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2021 Google LLC. Licensed under the Apache License, Version 2.0.

//! 数値計算の補助

/// 符号。0 は 0、負は -1、それ以外 (NaN を含む) は 1
pub(crate) fn signum(num: f64) -> f64 {
    if num < 0.0 {
        -1.0
    } else if num == 0.0 {
        0.0
    } else {
        1.0
    }
}

/// `amount` が 0 なら `start`、1 なら `stop`
pub(crate) fn lerp(start: f64, stop: f64, amount: f64) -> f64 {
    (1.0 - amount) * start + amount * stop
}

/// `[min, max]` に収める
pub(crate) fn clamp_int(min: i64, max: i64, input: i64) -> i64 {
    if input < min {
        min
    } else if input > max {
        max
    } else {
        input
    }
}

/// `[min, max]` に収める。NaN はそのまま返し、`min > max` でも panic しない
pub(crate) fn clamp_double(min: f64, max: f64, input: f64) -> f64 {
    if input < min {
        min
    } else if input > max {
        max
    } else {
        input
    }
}

/// 角度を `[0, 360)` に回し込む
pub(crate) fn sanitize_degrees_int(degrees: i64) -> i64 {
    let degrees = degrees % 360;
    if degrees < 0 { degrees + 360 } else { degrees }
}

/// 角度を `[0, 360)` に回し込む
pub(crate) fn sanitize_degrees_double(degrees: f64) -> f64 {
    let degrees = degrees % 360.0;
    if degrees < 0.0 {
        degrees + 360.0
    } else {
        degrees
    }
}

/// `from` から `to` へ最短で回る向き。増える向きなら 1、減る向きなら -1。180 度離れているときは 1
pub(crate) fn rotation_direction(from: f64, to: f64) -> f64 {
    let increasing_difference = sanitize_degrees_double(to - from);
    if increasing_difference <= 180.0 {
        1.0
    } else {
        -1.0
    }
}

/// 円周上の 2 つの角度の差 (度)
pub(crate) fn difference_degrees(a: f64, b: f64) -> f64 {
    180.0 - ((a - b).abs() - 180.0).abs()
}

/// 1x3 の行ベクトルと 3x3 行列の積
pub(crate) fn matrix_multiply(row: [f64; 3], matrix: &[[f64; 3]; 3]) -> [f64; 3] {
    let a = row[0] * matrix[0][0] + row[1] * matrix[0][1] + row[2] * matrix[0][2];
    let b = row[0] * matrix[1][0] + row[1] * matrix[1][1] + row[2] * matrix[1][2];
    let c = row[0] * matrix[2][0] + row[1] * matrix[2][1] + row[2] * matrix[2][2];
    [a, b, c]
}

/// JavaScript の `Math.round` と同じ丸め。0.5 ちょうどは正の無限大の向きに丸める
///
/// 移植元にない補助で、`f64::round` (0 から遠い向き) との差を吸収する
pub(crate) fn round_js(x: f64) -> f64 {
    let floor = x.floor();
    if x - floor >= 0.5 { floor + 1.0 } else { floor }
}

/// JavaScript の `Math.max` と同じ比較。どちらかが NaN なら NaN を返す
///
/// 移植元にない補助で、NaN を無視する `f64::max` との差を吸収する
pub(crate) fn max_js(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else {
        a.max(b)
    }
}

/// JavaScript の `Math.min` と同じ比較。どちらかが NaN なら NaN を返す
///
/// 移植元にない補助で、NaN を無視する `f64::min` との差を吸収する
pub(crate) fn min_js(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else {
        a.min(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Source: typescript/utils/utils_test.ts @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
    #[test]
    fn rotation_direction_は最短の向き() {
        for from in (0..24).map(|i| i as f64 * 15.0) {
            for to in (0..24).map(|i| 7.5 + i as f64 * 15.0) {
                let candidates = [to - from, to - from + 360.0, to - from - 360.0];
                let nearest = candidates
                    .into_iter()
                    .fold(f64::INFINITY, |a, b| if b.abs() < a.abs() { b } else { a });
                let expected = if nearest >= 0.0 { 1.0 } else { -1.0 };
                assert_eq!(rotation_direction(from, to), expected, "{from} -> {to}");
            }
        }
    }

    #[test]
    fn max_js_と_min_js_は_nan_を伝える() {
        assert!(max_js(0.0, f64::NAN).is_nan());
        assert!(min_js(f64::NAN, 1.0).is_nan());
        assert_eq!(max_js(-0.5, 2.0), 2.0);
        assert_eq!(min_js(-0.5, 2.0), -0.5);
    }

    #[test]
    fn round_js_は_0_5_を正の向きに丸める() {
        assert_eq!(round_js(0.5), 1.0);
        assert_eq!(round_js(-0.5), 0.0);
        assert_eq!(round_js(-1.5), -1.0);
        assert_eq!(round_js(2.4), 2.0);
    }
}
