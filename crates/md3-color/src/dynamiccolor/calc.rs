// Ported from material-color-utilities
// (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/dynamiccolor/dynamic_color.ts
//         (DynamicColor, ColorCalculationDelegateImpl2021, ColorCalculationDelegateImpl2025)
//         @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2022 Google LLC. Licensed under the Apache License, Version 2.0.

//! 色の役割の定義から、配色での HCT とトーンを求める
//!
//! 移植元は 2021 仕様の配色に 2021 仕様の計算を、それ以外の配色に 2025 仕様の計算を使う

use super::spec::{ColorSpec, spec};
use super::tone_delta_pair::{DeltaConstraint, TonePolarity};
use super::{ColorRole, DynamicScheme, Platform, SpecVersion};
use crate::contrast;
use crate::hct::Hct;
use crate::palettes::TonalPalette;
use crate::utils::math::{self, round_js};

/// 配色での `role` の色
pub(crate) fn get_hct(scheme: &DynamicScheme, role: ColorRole) -> Hct {
    let color = spec(role, scheme.spec_version);
    let palette = scheme.palette((color.palette)(scheme));
    let tone = get_tone(scheme, role);
    if scheme.spec_version == SpecVersion::Spec2021 {
        return palette.hct_raw(tone);
    }

    let multiplier = (color.chroma_multiplier)(scheme);
    if multiplier == 1.0 {
        return palette.hct_raw(tone);
    }
    let chroma = palette.chroma_raw() * multiplier;
    if tone == 99.0 && Hct::is_yellow(palette.hue_raw()) {
        return TonalPalette::from_hue_and_chroma_raw(palette.hue_raw(), chroma).hct_raw(tone);
    }
    Hct::from_hct_raw(palette.hue_raw(), chroma, tone)
}

/// 配色での `role` のトーン。背景とのコントラストやトーンの差の制約を反映した値
pub(crate) fn get_tone(scheme: &DynamicScheme, role: ColorRole) -> f64 {
    if scheme.spec_version == SpecVersion::Spec2021 {
        get_tone_2021(scheme, role)
    } else {
        get_tone_2025(scheme, role)
    }
}

/// 定義のトーン関数の値 (移植元の `DynamicColor.tone(scheme)`)。制約は反映しない
pub(crate) fn initial_tone(scheme: &DynamicScheme, role: ColorRole) -> f64 {
    let color = spec(role, scheme.spec_version);
    initial_tone_of(scheme, &color)
}

fn initial_tone_of(scheme: &DynamicScheme, color: &ColorSpec) -> f64 {
    match color.tone {
        Some(tone) => tone(scheme),
        None => tone_from_background(scheme, (color.background)(scheme)),
    }
}

/// 移植元の `DynamicColor.getInitialToneFromBackground`。背景がなければ 50
pub(crate) fn tone_from_background(scheme: &DynamicScheme, background: Option<ColorRole>) -> f64 {
    match background {
        Some(background) => get_tone(scheme, background),
        None => 50.0,
    }
}

/// 背景のトーン `bg_tone` に対して、コントラスト比 `ratio` 以上になる文字のトーン
///
/// 比を満たせないときは、比が大きい側のトーンを選ぶ
pub(crate) fn foreground_tone(bg_tone: f64, ratio: f64) -> f64 {
    let lighter_tone = contrast::lighter_unsafe_raw(bg_tone, ratio);
    let darker_tone = contrast::darker_unsafe_raw(bg_tone, ratio);
    let lighter_ratio = contrast::ratio_of_tones_raw(lighter_tone, bg_tone);
    let darker_ratio = contrast::ratio_of_tones_raw(darker_tone, bg_tone);
    let prefer_lighter = tone_prefers_light_foreground(bg_tone);

    if prefer_lighter {
        // 明るい側と暗い側の比の差が 0.1 未満で、どちらも目標を満たさないときは明るい側を選ぶ
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

/// 四捨五入したトーンが 60 未満なら、明るい文字が向く
///
/// トーン 50〜59 は明るい文字でも暗い文字でも十分なコントラストを得られないため、
/// 境界を 50 ではなく 60 に置き、明るい文字を優先する
fn tone_prefers_light_foreground(tone: f64) -> bool {
    round_js(tone) < 60.0
}

/// 背景と 2 つ目の背景の両方に対してコントラスト比を満たすトーン
///
/// 2021 仕様と 2025 仕様の計算で共通の処理
fn tone_for_two_backgrounds(
    scheme: &DynamicScheme,
    answer: f64,
    background: ColorRole,
    second_background: ColorRole,
    desired_ratio: f64,
) -> f64 {
    let bg_tone1 = get_tone(scheme, background);
    let bg_tone2 = get_tone(scheme, second_background);
    let upper = math::max_js(bg_tone1, bg_tone2);
    let lower = math::min_js(bg_tone1, bg_tone2);

    if contrast::ratio_of_tones_raw(upper, answer) >= desired_ratio
        && contrast::ratio_of_tones_raw(lower, answer) >= desired_ratio
    {
        return answer;
    }

    // 背景より明るい候補と暗い候補のうち、比を満たせるもの
    let light_option = contrast::lighter_raw(upper, desired_ratio);
    let dark_option = contrast::darker_raw(lower, desired_ratio);
    let availables = [light_option, dark_option]
        .into_iter()
        .filter(|option| *option != -1.0)
        .count();

    let prefers_light =
        tone_prefers_light_foreground(bg_tone1) || tone_prefers_light_foreground(bg_tone2);
    if prefers_light {
        return if light_option < 0.0 {
            100.0
        } else {
            light_option
        };
    }
    if availables == 1 {
        return if light_option != -1.0 {
            light_option
        } else {
            dark_option
        };
    }
    if dark_option < 0.0 { 0.0 } else { dark_option }
}

fn get_tone_2021(scheme: &DynamicScheme, role: ColorRole) -> f64 {
    let color = spec(role, scheme.spec_version);
    let decreasing_contrast = scheme.contrast_level < 0.0;

    if let Some(tone_delta_pair) = (color.tone_delta_pair)(scheme) {
        let role_a = tone_delta_pair.role_a;
        let role_b = tone_delta_pair.role_b;
        let delta = tone_delta_pair.delta;
        let polarity = tone_delta_pair.polarity;
        let stay_together = tone_delta_pair.stay_together;

        let a_is_nearer = polarity == TonePolarity::Nearer
            || (polarity == TonePolarity::Lighter && !scheme.is_dark)
            || (polarity == TonePolarity::Darker && scheme.is_dark);
        let (nearer, farther) = if a_is_nearer {
            (role_a, role_b)
        } else {
            (role_b, role_a)
        };
        let am_nearer = role == nearer;
        let expansion_dir = if scheme.is_dark { 1.0 } else { -1.0 };
        let nearer_spec = spec(nearer, scheme.spec_version);
        let farther_spec = spec(farther, scheme.spec_version);
        let mut n_tone = initial_tone_of(scheme, &nearer_spec);
        let mut f_tone = initial_tone_of(scheme, &farther_spec);

        // 1 回目: 背景とのコントラスト比を満たすように動かす
        if let (Some(bg), Some(n_contrast_curve), Some(f_contrast_curve)) = (
            (color.background)(scheme),
            (nearer_spec.contrast_curve)(scheme),
            (farther_spec.contrast_curve)(scheme),
        ) {
            let bg_tone = get_tone(scheme, bg);
            let n_contrast = n_contrast_curve.get(scheme.contrast_level);
            let f_contrast = f_contrast_curve.get(scheme.contrast_level);
            if contrast::ratio_of_tones_raw(bg_tone, n_tone) < n_contrast {
                n_tone = foreground_tone(bg_tone, n_contrast);
            }
            if contrast::ratio_of_tones_raw(bg_tone, f_tone) < f_contrast {
                f_tone = foreground_tone(bg_tone, f_contrast);
            }
            if decreasing_contrast {
                // コントラストを下げるときは、目標の比を満たす範囲で背景に近づける
                n_tone = foreground_tone(bg_tone, n_contrast);
                f_tone = foreground_tone(bg_tone, f_contrast);
            }
        }

        // 2 回目: 2 つの役割のトーンの差を保つ
        if (f_tone - n_tone) * expansion_dir < delta {
            // 差が足りなければ、まず遠い方を広げる
            f_tone = math::clamp_double(0.0, 100.0, n_tone + delta * expansion_dir);
            // それでも足りなければ、近い方を戻す
            if (f_tone - n_tone) * expansion_dir < delta {
                n_tone = math::clamp_double(0.0, 100.0, f_tone - delta * expansion_dir);
            }
        }

        // 3 回目: トーン 50〜59 を避ける。明るい色も暗い色も十分なコントラストを得られない
        if (50.0..60.0).contains(&n_tone) {
            // 近い方を先に動かし、遠い方も差を保つように動かす
            if expansion_dir > 0.0 {
                n_tone = 60.0;
                f_tone = math::max_js(f_tone, n_tone + delta * expansion_dir);
            } else {
                n_tone = 49.0;
                f_tone = math::min_js(f_tone, n_tone + delta * expansion_dir);
            }
        } else if (50.0..60.0).contains(&f_tone) {
            if stay_together {
                // 2 つを同じ側へ動かす
                if expansion_dir > 0.0 {
                    n_tone = 60.0;
                    f_tone = math::max_js(f_tone, n_tone + delta * expansion_dir);
                } else {
                    n_tone = 49.0;
                    f_tone = math::min_js(f_tone, n_tone + delta * expansion_dir);
                }
            } else if expansion_dir > 0.0 {
                // 遠い方だけを動かす
                f_tone = 60.0;
            } else {
                f_tone = 49.0;
            }
        }

        return if am_nearer { n_tone } else { f_tone };
    }

    // 単独の役割
    let mut answer = initial_tone_of(scheme, &color);

    let (Some(background), Some(contrast_curve)) =
        ((color.background)(scheme), (color.contrast_curve)(scheme))
    else {
        // 背景がない役割は調整しない
        return answer;
    };

    let bg_tone = get_tone(scheme, background);
    let desired_ratio = contrast_curve.get(scheme.contrast_level);

    if contrast::ratio_of_tones_raw(bg_tone, answer) < desired_ratio {
        // 比が足りなければ、背景から離す
        answer = foreground_tone(bg_tone, desired_ratio);
    }

    if decreasing_contrast {
        answer = foreground_tone(bg_tone, desired_ratio);
    }

    if color.is_background && (50.0..60.0).contains(&answer) {
        // 背景になる役割はトーン 50〜59 を避ける
        answer = if contrast::ratio_of_tones_raw(49.0, bg_tone) >= desired_ratio {
            49.0
        } else {
            60.0
        };
    }

    let Some(second_background) = (color.second_background)(scheme) else {
        return answer;
    };
    tone_for_two_backgrounds(scheme, answer, background, second_background, desired_ratio)
}

fn get_tone_2025(scheme: &DynamicScheme, role: ColorRole) -> f64 {
    let color = spec(role, scheme.spec_version);

    if let Some(tone_delta_pair) = (color.tone_delta_pair)(scheme) {
        let role_a = tone_delta_pair.role_a;
        let role_b = tone_delta_pair.role_b;
        let polarity = tone_delta_pair.polarity;
        let constraint = tone_delta_pair.constraint;
        let absolute_delta = if polarity == TonePolarity::Darker
            || (polarity == TonePolarity::RelativeLighter && scheme.is_dark)
            || (polarity == TonePolarity::RelativeDarker && !scheme.is_dark)
        {
            -tone_delta_pair.delta
        } else {
            tone_delta_pair.delta
        };

        let am_role_a = role == role_a;
        let (self_role, ref_role) = if am_role_a {
            (role_a, role_b)
        } else {
            (role_b, role_a)
        };
        let mut self_tone = initial_tone(scheme, self_role);
        let ref_tone = get_tone(scheme, ref_role);
        let relative_delta = absolute_delta * if am_role_a { 1.0 } else { -1.0 };

        match constraint {
            DeltaConstraint::Exact => {
                self_tone = math::clamp_double(0.0, 100.0, ref_tone + relative_delta);
            }
            DeltaConstraint::Farther => {
                self_tone = if relative_delta > 0.0 {
                    math::clamp_double(ref_tone + relative_delta, 100.0, self_tone)
                } else {
                    math::clamp_double(0.0, ref_tone + relative_delta, self_tone)
                };
            }
        }

        if let (Some(background), Some(contrast_curve)) =
            ((color.background)(scheme), (color.contrast_curve)(scheme))
        {
            let bg_tone = get_tone(scheme, background);
            let self_contrast = contrast_curve.get(scheme.contrast_level);
            if !(contrast::ratio_of_tones_raw(bg_tone, self_tone) >= self_contrast
                && scheme.contrast_level >= 0.0)
            {
                self_tone = foreground_tone(bg_tone, self_contrast);
            }
        }

        // 色の役割は名前ではなく ColorRole で表すため、`_fixed_dim` で終わる名前の判定を役割で行う
        if scheme.platform == Platform::Phone && color.is_background && !is_fixed_dim(role) {
            self_tone = if self_tone >= 57.0 {
                math::clamp_double(65.0, 100.0, self_tone)
            } else {
                math::clamp_double(0.0, 49.0, self_tone)
            };
        }

        return self_tone;
    }

    // 単独の役割
    let mut answer = initial_tone_of(scheme, &color);

    let (Some(background), Some(contrast_curve)) =
        ((color.background)(scheme), (color.contrast_curve)(scheme))
    else {
        // 背景がない役割は調整しない
        return answer;
    };

    let bg_tone = get_tone(scheme, background);
    let desired_ratio = contrast_curve.get(scheme.contrast_level);

    // 比が足りないとき、またはコントラストを下げるときは、目標の比になるトーンにする
    if !(contrast::ratio_of_tones_raw(bg_tone, answer) >= desired_ratio
        && scheme.contrast_level >= 0.0)
    {
        answer = foreground_tone(bg_tone, desired_ratio);
    }

    if scheme.platform == Platform::Phone && color.is_background && !is_fixed_dim(role) {
        // 背景になる役割はトーン 50〜64 を避ける
        answer = if answer >= 57.0 {
            math::clamp_double(65.0, 100.0, answer)
        } else {
            math::clamp_double(0.0, 49.0, answer)
        };
    } else if scheme.platform == Platform::Watch
        && color.is_background
        && (50.0..60.0).contains(&answer)
    {
        answer = if contrast::ratio_of_tones_raw(49.0, bg_tone) >= desired_ratio {
            49.0
        } else {
            60.0
        };
    }

    let Some(second_background) = (color.second_background)(scheme) else {
        return answer;
    };
    tone_for_two_backgrounds(scheme, answer, background, second_background, desired_ratio)
}

/// 移植元の `color.name.endsWith('_fixed_dim')`
fn is_fixed_dim(role: ColorRole) -> bool {
    matches!(
        role,
        ColorRole::PrimaryFixedDim | ColorRole::SecondaryFixedDim | ColorRole::TertiaryFixedDim
    )
}
