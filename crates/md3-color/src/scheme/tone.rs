// Ported from material-color-utilities (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/dynamiccolor/dynamic_color.ts (DynamicColor, ColorCalculationDelegateImpl2021)
//         @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2021 Google LLC. Licensed under the Apache License, Version 2.0.

//! コントラスト制約を適用して各役割のトーンを決める（2021 仕様）。

use super::DynamicScheme;
use super::role::{ColorRole, TonePolarity};
use crate::contrast;
use crate::hct::Hct;
use crate::utils::math;

/// 背景トーン `bg_tone` に対してコントラスト比 `ratio` を満たす前景のトーン。
pub(crate) fn foreground_tone(bg_tone: f64, ratio: f64) -> f64 {
    let lighter_tone = contrast::lighter_unsafe_raw(bg_tone, ratio);
    let darker_tone = contrast::darker_unsafe_raw(bg_tone, ratio);
    let lighter_ratio = contrast::ratio_of_tones_raw(lighter_tone, bg_tone);
    let darker_ratio = contrast::ratio_of_tones_raw(darker_tone, bg_tone);
    let prefer_lighter = tone_prefers_light_foreground(bg_tone);

    if prefer_lighter {
        // "Negligible difference" handles an edge case where the initial contrast
        // ratio is high (ex. 13.0), and the ratio passed to the function is that
        // high ratio, and both the lighter and darker ratio fails to pass that
        // ratio.
        let negligible_difference = (lighter_ratio - darker_ratio).abs() < 0.1
            && lighter_ratio < ratio
            && darker_ratio < ratio;
        if lighter_ratio >= ratio || lighter_ratio >= darker_ratio || negligible_difference {
            lighter_tone
        } else {
            darker_tone
        }
    } else if darker_ratio >= ratio || darker_ratio >= lighter_ratio {
        darker_tone
    } else {
        lighter_tone
    }
}

/// 明るい前景色が好ましいトーンか（丸めて 60 未満）。
pub(crate) fn tone_prefers_light_foreground(tone: f64) -> bool {
    math::js_round(tone) < 60.0
}

/// 役割のトーンを求める。移植元の `ColorCalculationDelegateImpl2021.getTone`。
pub(crate) fn get_tone(s: &DynamicScheme, color: ColorRole) -> f64 {
    let decreasing_contrast = s.contrast_level < 0.0;

    if let Some(tone_delta_pair) = color.tone_delta_pair() {
        let role_a = tone_delta_pair.role_a;
        let role_b = tone_delta_pair.role_b;
        let delta = tone_delta_pair.delta;
        let polarity = tone_delta_pair.polarity;
        let stay_together = tone_delta_pair.stay_together;

        let a_is_nearer = polarity == TonePolarity::Nearer
            || (polarity == TonePolarity::Lighter && !s.is_dark)
            || (polarity == TonePolarity::Darker && s.is_dark);
        let (nearer, farther) = if a_is_nearer {
            (role_a, role_b)
        } else {
            (role_b, role_a)
        };
        let am_nearer = color == nearer;
        let expansion_dir = if s.is_dark { 1.0 } else { -1.0 };

        let mut n_tone = nearer.tone(s);
        let mut f_tone = farther.tone(s);

        // 1st round: solve to min, each
        if let (Some(bg), Some(n_contrast_curve), Some(f_contrast_curve)) = (
            color.background(s),
            nearer.contrast_curve(),
            farther.contrast_curve(),
        ) {
            let bg_tone = get_tone(s, bg);
            let n_contrast = n_contrast_curve.get(s.contrast_level);
            let f_contrast = f_contrast_curve.get(s.contrast_level);

            if contrast::ratio_of_tones_raw(bg_tone, n_tone) < n_contrast {
                n_tone = foreground_tone(bg_tone, n_contrast);
            }
            if contrast::ratio_of_tones_raw(bg_tone, f_tone) < f_contrast {
                f_tone = foreground_tone(bg_tone, f_contrast);
            }
            if decreasing_contrast {
                // If decreasing contrast, adjust color to the "bare minimum"
                // that satisfies contrast.
                n_tone = foreground_tone(bg_tone, n_contrast);
                f_tone = foreground_tone(bg_tone, f_contrast);
            }
        }

        // If constraint is not satisfied, try another round.
        if (f_tone - n_tone) * expansion_dir < delta {
            // 2nd round: expand farther to match delta.
            f_tone = math::clamp_double(0.0, 100.0, n_tone + delta * expansion_dir);
            // If constraint is not satisfied, try another round.
            if (f_tone - n_tone) * expansion_dir < delta {
                // 3rd round: contract nearer to match delta.
                n_tone = math::clamp_double(0.0, 100.0, f_tone - delta * expansion_dir);
            }
        }

        // Avoids the 50-59 awkward zone.
        if (50.0..60.0).contains(&n_tone) {
            // If `nearer` is in the awkward zone, move it away, together with
            // `farther`.
            if expansion_dir > 0.0 {
                n_tone = 60.0;
                f_tone = f64::max(f_tone, n_tone + delta * expansion_dir);
            } else {
                n_tone = 49.0;
                f_tone = f64::min(f_tone, n_tone + delta * expansion_dir);
            }
        } else if (50.0..60.0).contains(&f_tone) {
            if stay_together {
                // Fixes both, to avoid two colors on opposite sides of the "awkward
                // zone".
                if expansion_dir > 0.0 {
                    n_tone = 60.0;
                    f_tone = f64::max(f_tone, n_tone + delta * expansion_dir);
                } else {
                    n_tone = 49.0;
                    f_tone = f64::min(f_tone, n_tone + delta * expansion_dir);
                }
            } else {
                // Not required to stay together; fixes just one.
                f_tone = if expansion_dir > 0.0 { 60.0 } else { 49.0 };
            }
        }

        // Returns `nTone` if this color is `nearer`, otherwise `fTone`.
        return if am_nearer { n_tone } else { f_tone };
    }

    // Case 2: No contrast pair; just solve for itself.
    let mut answer = color.tone(s);

    let (Some(bg), Some(contrast_curve)) = (color.background(s), color.contrast_curve()) else {
        return answer; // No adjustment for colors with no background.
    };

    let bg_tone = get_tone(s, bg);
    let desired_ratio = contrast_curve.get(s.contrast_level);

    if contrast::ratio_of_tones_raw(bg_tone, answer) < desired_ratio {
        // Rough improvement.
        answer = foreground_tone(bg_tone, desired_ratio);
    }

    if decreasing_contrast {
        answer = foreground_tone(bg_tone, desired_ratio);
    }

    if color.is_background() && (50.0..60.0).contains(&answer) {
        // Must adjust
        answer = if contrast::ratio_of_tones_raw(49.0, bg_tone) >= desired_ratio {
            49.0
        } else {
            60.0
        };
    }

    let Some(second_bg) = color.second_background() else {
        return answer;
    };

    // Case 3: Adjust for dual backgrounds.
    let bg_tone1 = get_tone(s, bg);
    let bg_tone2 = get_tone(s, second_bg);
    let upper = f64::max(bg_tone1, bg_tone2);
    let lower = f64::min(bg_tone1, bg_tone2);

    if contrast::ratio_of_tones_raw(upper, answer) >= desired_ratio
        && contrast::ratio_of_tones_raw(lower, answer) >= desired_ratio
    {
        return answer;
    }

    // The darkest light tone that satisfies the desired ratio,
    // or -1 if such ratio cannot be reached.
    let light_option = contrast::lighter_raw(upper, desired_ratio);

    // The lightest dark tone that satisfies the desired ratio,
    // or -1 if such ratio cannot be reached.
    let dark_option = contrast::darker_raw(lower, desired_ratio);

    // Tones suitable for the foreground.
    let availables: Vec<f64> = [light_option, dark_option]
        .into_iter()
        .filter(|&t| t != -1.0)
        .collect();

    let prefers_light =
        tone_prefers_light_foreground(bg_tone1) || tone_prefers_light_foreground(bg_tone2);
    if prefers_light {
        return if light_option < 0.0 {
            100.0
        } else {
            light_option
        };
    }
    if let [only] = availables.as_slice() {
        return *only;
    }
    if dark_option < 0.0 { 0.0 } else { dark_option }
}

/// 役割の色。移植元の `ColorCalculationDelegateImpl2021.getHct`。
pub(crate) fn get_hct(s: &DynamicScheme, color: ColorRole) -> Hct {
    let tone = get_tone(s, color);
    color.palette(s).hct_raw(tone)
}
