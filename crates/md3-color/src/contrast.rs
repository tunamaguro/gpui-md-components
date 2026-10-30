// Ported from material-color-utilities: typescript/contrast/contrast.ts @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3. See NOTICE.

//! WCAG のコントラスト比の計算。
//!
//! トーン（L*）どうしのコントラスト比を求めたり、目標のコントラスト比を満たす
//! トーンを求めたりする。

use crate::units::Tone;
use crate::utils::{color, math};

/// 2 つのトーンのコントラスト比（1〜21）。
pub fn ratio_of_tones(a: Tone, b: Tone) -> f64 {
    ratio_of_tones_raw(a.value(), b.value())
}

/// `tone` より明るく、コントラスト比が `ratio` 以上になるトーン。
///
/// 満たすトーンがなければ `None`。
pub fn lighter(tone: Tone, ratio: f64) -> Option<Tone> {
    let lighter = lighter_raw(tone.value(), ratio);
    (lighter >= 0.0).then_some(Tone::raw(lighter))
}

/// `tone` より暗く、コントラスト比が `ratio` 以上になるトーン。
///
/// 満たすトーンがなければ `None`。
pub fn darker(tone: Tone, ratio: f64) -> Option<Tone> {
    let darker = darker_raw(tone.value(), ratio);
    (darker >= 0.0).then_some(Tone::raw(darker))
}

/// [`lighter`] と同じだが、満たすトーンがなければ 100 を返す。
pub fn lighter_unsafe(tone: Tone, ratio: f64) -> Tone {
    Tone::raw(lighter_unsafe_raw(tone.value(), ratio))
}

/// [`darker`] と同じだが、満たすトーンがなければ 0 を返す。
pub fn darker_unsafe(tone: Tone, ratio: f64) -> Tone {
    Tone::raw(darker_unsafe_raw(tone.value(), ratio))
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

/// 移植元の `Contrast.lighter`。見つからなければ -1 を返す。
pub(crate) fn lighter_raw(tone: f64, ratio: f64) -> f64 {
    if !(0.0..=100.0).contains(&tone) {
        return -1.0;
    }
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

/// 移植元の `Contrast.darker`。見つからなければ -1 を返す。
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

#[cfg(test)]
mod tests {
    // Ported from typescript/contrast/contrast_test.ts

    use super::*;

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 0.5, "{actual} != {expected}");
    }

    #[test]
    fn ratio_of_tones_out_of_bounds_input() {
        assert_close(ratio_of_tones_raw(-10.0, 110.0), 21.0);
    }

    #[test]
    fn lighter_impossible_ratio_errors() {
        assert_close(lighter_raw(90.0, 10.0), -1.0);
        assert_eq!(lighter(Tone::new(90.0), 10.0), None);
    }

    #[test]
    fn lighter_out_of_bounds_input_above_errors() {
        assert_close(lighter_raw(110.0, 2.0), -1.0);
    }

    #[test]
    fn lighter_out_of_bounds_input_below_errors() {
        assert_close(lighter_raw(-10.0, 2.0), -1.0);
    }

    #[test]
    fn lighter_unsafe_returns_max_tone() {
        assert_close(lighter_unsafe(Tone::new(100.0), 2.0).value(), 100.0);
    }

    #[test]
    fn darker_impossible_ratio_errors() {
        assert_close(darker_raw(10.0, 20.0), -1.0);
        assert_eq!(darker(Tone::new(10.0), 20.0), None);
    }

    #[test]
    fn darker_out_of_bounds_input_above_errors() {
        assert_close(darker_raw(110.0, 2.0), -1.0);
    }

    #[test]
    fn darker_out_of_bounds_input_below_errors() {
        assert_close(darker_raw(-10.0, 2.0), -1.0);
    }

    #[test]
    fn darker_unsafe_returns_min_tone() {
        assert_close(darker_unsafe(Tone::new(0.0), 2.0).value(), 0.0);
    }
}
