// Ported from material-color-utilities (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/utils/math_utils.ts @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2021 Google LLC. Licensed under the Apache License, Version 2.0.

//! 数値ユーティリティ。

/// 符号（-1, 0, 1）。
pub(crate) fn signum(num: f64) -> f64 {
    if num < 0.0 {
        -1.0
    } else if num == 0.0 {
        0.0
    } else {
        1.0
    }
}

/// 線形補間。
pub(crate) fn lerp(start: f64, stop: f64, amount: f64) -> f64 {
    (1.0 - amount) * start + amount * stop
}

/// `input` を `[min, max]` に収める。
pub(crate) fn clamp_int(min: i64, max: i64, input: i64) -> i64 {
    if input < min {
        min
    } else if input > max {
        max
    } else {
        input
    }
}

/// `input` を `[min, max]` に収める（NaN はそのまま返す）。
pub(crate) fn clamp_double(min: f64, max: f64, input: f64) -> f64 {
    if input < min {
        min
    } else if input > max {
        max
    } else {
        input
    }
}

/// 角度（整数）を `[0, 360)` に正規化する。
pub(crate) fn sanitize_degrees_int(degrees: i64) -> i64 {
    let degrees = degrees % 360;
    if degrees < 0 { degrees + 360 } else { degrees }
}

/// 角度を `[0, 360)` に正規化する。
pub(crate) fn sanitize_degrees_double(degrees: f64) -> f64 {
    let degrees = degrees % 360.0;
    if degrees < 0.0 {
        degrees + 360.0
    } else {
        degrees
    }
}

/// 行ベクトルと 3x3 行列の積。
pub(crate) fn matrix_multiply(row: [f64; 3], matrix: &[[f64; 3]; 3]) -> [f64; 3] {
    let a = row[0] * matrix[0][0] + row[1] * matrix[0][1] + row[2] * matrix[0][2];
    let b = row[0] * matrix[1][0] + row[1] * matrix[1][1] + row[2] * matrix[1][2];
    let c = row[0] * matrix[2][0] + row[1] * matrix[2][1] + row[2] * matrix[2][2];
    [a, b, c]
}

/// JavaScript の `Math.round` と同じ丸め（0.5 は正の無限大方向）。
///
/// 移植元の期待値と ARGB を完全一致させるため、`f64::round`（0 から離れる方向）ではなくこれを使う。
pub(crate) fn js_round(x: f64) -> f64 {
    (x + 0.5).floor()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn js_round_rounds_half_towards_positive_infinity() {
        assert_eq!(js_round(0.5), 1.0);
        assert_eq!(js_round(-0.5), 0.0);
        assert_eq!(js_round(-1.5), -1.0);
        assert_eq!(js_round(2.4), 2.0);
    }
}
