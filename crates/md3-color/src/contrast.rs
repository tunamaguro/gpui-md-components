// Ported from material-color-utilities
// (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/contrast/contrast.ts @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2022 Google LLC. Licensed under the Apache License, Version 2.0.

//! トーンのコントラスト比 (WCAG 2.1 の相対輝度に基づく)
//!
//! コントラスト比は 1 から 21 の値を取る。トーンの差が 40 以上あれば 3.0、50 以上あれば 4.5 以上になる

use crate::units::Tone;
use crate::utils::{color, math};

/// 2 つのトーンのコントラスト比。順序によらない
pub fn ratio_of_tones(tone_a: Tone, tone_b: Tone) -> f64 {
    ratio_of_tones_raw(tone_a.value(), tone_b.value())
}

/// `tone` 以上に明るく、`ratio` 以上のコントラスト比になるトーン
///
/// 目標の比を満たせない (誤差 0.04 を超えて下回る) とき、または結果が `[0, 100]` を外れるときは `None`。
/// 結果は端数の誤差を見込んで 0.4 だけ明るい側に寄せる
pub fn lighter(tone: Tone, ratio: f64) -> Option<Tone> {
    let lighter = lighter_raw(tone.value(), ratio);
    (lighter >= 0.0).then(|| Tone::new(lighter))
}

/// `tone` 以下に暗く、`ratio` 以上のコントラスト比になるトーン
///
/// 目標の比を満たせない (誤差 0.04 を超えて下回る) とき、または結果が `[0, 100]` を外れるときは `None`。
/// 結果は端数の誤差を見込んで 0.4 だけ暗い側に寄せる
pub fn darker(tone: Tone, ratio: f64) -> Option<Tone> {
    let darker = darker_raw(tone.value(), ratio);
    (darker >= 0.0).then(|| Tone::new(darker))
}

/// [`lighter`] と同じだが、目標を満たせないときは 100 を返す
pub fn lighter_unsafe(tone: Tone, ratio: f64) -> Tone {
    Tone::new(lighter_unsafe_raw(tone.value(), ratio))
}

/// [`darker`] と同じだが、目標を満たせないときは 0 を返す
pub fn darker_unsafe(tone: Tone, ratio: f64) -> Tone {
    Tone::new(darker_unsafe_raw(tone.value(), ratio))
}

pub(crate) fn ratio_of_tones_raw(tone_a: f64, tone_b: f64) -> f64 {
    let tone_a = math::clamp_double(0.0, 100.0, tone_a);
    let tone_b = math::clamp_double(0.0, 100.0, tone_b);
    ratio_of_ys(color::y_from_lstar(tone_a), color::y_from_lstar(tone_b))
}

fn ratio_of_ys(y1: f64, y2: f64) -> f64 {
    let lighter = if y1 > y2 { y1 } else { y2 };
    let darker = if lighter == y2 { y1 } else { y2 };
    (lighter + 5.0) / (darker + 5.0)
}

/// 移植元と同じく、条件を満たせないときは -1 を返す
pub(crate) fn lighter_raw(tone: f64, ratio: f64) -> f64 {
    if !(0.0..=100.0).contains(&tone) {
        return -1.0;
    }
    // 2 つの色の Y と比の関係は、比 = (明るい Y + 5) / (暗い Y + 5)
    let dark_y = color::y_from_lstar(tone);
    let light_y = ratio * (dark_y + 5.0) - 5.0;
    let real_contrast = ratio_of_ys(light_y, dark_y);
    let delta = (real_contrast - ratio).abs();
    if real_contrast < ratio && delta > 0.04 {
        return -1.0;
    }
    let return_value = color::lstar_from_y(light_y) + 0.4;
    if !(0.0..=100.0).contains(&return_value) {
        return -1.0;
    }
    return_value
}

/// 移植元と同じく、条件を満たせないときは -1 を返す
pub(crate) fn darker_raw(tone: f64, ratio: f64) -> f64 {
    if !(0.0..=100.0).contains(&tone) {
        return -1.0;
    }
    let light_y = color::y_from_lstar(tone);
    let dark_y = ((light_y + 5.0) / ratio) - 5.0;
    let real_contrast = ratio_of_ys(light_y, dark_y);
    let delta = (real_contrast - ratio).abs();
    if real_contrast < ratio && delta > 0.04 {
        return -1.0;
    }
    let return_value = color::lstar_from_y(dark_y) - 0.4;
    if !(0.0..=100.0).contains(&return_value) {
        return -1.0;
    }
    return_value
}

pub(crate) fn lighter_unsafe_raw(tone: f64, ratio: f64) -> f64 {
    let lighter_safe = lighter_raw(tone, ratio);
    if lighter_safe < 0.0 {
        100.0
    } else {
        lighter_safe
    }
}

pub(crate) fn darker_unsafe_raw(tone: f64, ratio: f64) -> f64 {
    let darker_safe = darker_raw(tone, ratio);
    if darker_safe < 0.0 { 0.0 } else { darker_safe }
}
