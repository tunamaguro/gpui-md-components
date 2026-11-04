// Ported from material-color-utilities
// (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/dynamiccolor/color_spec_2025.ts (ColorSpecDelegateImpl2025)
//         @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2025 Google LLC. Licensed under the Apache License, Version 2.0.

//! 2025 仕様の色の役割の定義

use super::calc::{foreground_tone, get_tone, tone_from_background};
use super::contrast_curve::ContrastCurve;
use super::scheme::Palette;
use super::spec::{ColorSpec, spec as spec_of};
use super::spec_2021::highest_surface;
use super::tone_delta_pair::{DeltaConstraint, ToneDeltaPair, TonePolarity};
use super::{ColorRole as R, DynamicScheme, Platform, SpecVersion, Variant};
use crate::hct::Hct;
use crate::palettes::TonalPalette;
use crate::utils::math;

/// 彩度が最も高くなるトーンを、明るい側から探して `[lower_bound, upper_bound]` に収める
pub(crate) fn t_max_c(
    palette: &TonalPalette,
    lower_bound: f64,
    upper_bound: f64,
    chroma_multiplier: f64,
) -> f64 {
    let answer = find_best_tone_for_chroma(
        palette.hue_raw(),
        palette.chroma_raw() * chroma_multiplier,
        100.0,
        true,
    );
    math::clamp_double(lower_bound, upper_bound, answer)
}

/// 彩度が最も高くなるトーンを、暗い側から探して `[lower_bound, upper_bound]` に収める
pub(crate) fn t_min_c(palette: &TonalPalette, lower_bound: f64, upper_bound: f64) -> f64 {
    let answer = find_best_tone_for_chroma(palette.hue_raw(), palette.chroma_raw(), 0.0, false);
    math::clamp_double(lower_bound, upper_bound, answer)
}

/// `tone` から 1 ずつ動かし、指定の彩度に届くまでの間で最も彩度が高いトーンを探す
fn find_best_tone_for_chroma(hue: f64, chroma: f64, tone: f64, by_decreasing_tone: bool) -> f64 {
    let mut tone = tone;
    let mut answer = tone;
    let mut best_candidate = Hct::from_hct_raw(hue, chroma, answer);
    while best_candidate.chroma_raw() < chroma {
        if !(0.0..=100.0).contains(&tone) {
            break;
        }
        tone += if by_decreasing_tone { -1.0 } else { 1.0 };
        let new_candidate = Hct::from_hct_raw(hue, chroma, tone);
        if best_candidate.chroma_raw() < new_candidate.chroma_raw() {
            best_candidate = new_candidate;
            answer = tone;
        }
    }
    answer
}

fn is_close_to_chroma(hct: &Hct, chroma: f64) -> bool {
    (hct.chroma_raw() - chroma).abs() < 0.4
}

/// `starting_tone` から `ending_tone` へ 1 ずつ動かし、指定の彩度に最も近いトーンを探す
fn find_desired_chroma_by_tone(hue: f64, chroma: f64, starting_tone: f64, ending_tone: f64) -> f64 {
    if ending_tone == starting_tone {
        return starting_tone;
    }
    let mut answer = starting_tone;
    let mut potential_answer = answer;
    let increment = if ending_tone < starting_tone {
        -1.0
    } else {
        1.0
    };
    let mut closest_to_chroma = Hct::from_hct_raw(hue, chroma, starting_tone);
    while closest_to_chroma.chroma_raw() < chroma && !is_close_to_chroma(&closest_to_chroma, chroma)
    {
        potential_answer += increment;
        if (potential_answer < ending_tone && ending_tone < starting_tone)
            || (potential_answer > ending_tone && ending_tone > starting_tone)
        {
            return answer;
        }
        let potential_solution = Hct::from_hct_raw(hue, chroma, potential_answer);
        if is_close_to_chroma(&potential_solution, chroma) {
            return potential_answer;
        }
        let potential_delta = (potential_solution.chroma_raw() - chroma).abs();
        let current_delta = (closest_to_chroma.chroma_raw() - chroma).abs();
        if potential_delta < current_delta {
            closest_to_chroma = potential_solution;
            answer = potential_answer;
        }
    }
    answer
}

fn main_tone(palette: &TonalPalette, max_tone: f64) -> f64 {
    let ending_tone = foreground_tone(25.0, 7.0);
    find_desired_chroma_by_tone(
        palette.hue_raw(),
        palette.chroma_raw(),
        max_tone,
        ending_tone,
    )
}

fn dim_tone(palette: &TonalPalette, max_tone: f64) -> f64 {
    let initial_tone = math::min_js(main_tone(palette, max_tone) - 5.0, max_tone);
    let ending_tone = foreground_tone(25.0, 4.5);
    find_desired_chroma_by_tone(
        palette.hue_raw(),
        palette.chroma_raw(),
        initial_tone,
        ending_tone,
    )
}

fn container_tone(palette: &TonalPalette, max_tone: f64) -> f64 {
    let initial_tone = math::min_js(dim_tone(palette, max_tone) - 5.0, max_tone);
    let ending_tone = foreground_tone(0.0, 7.0);
    find_desired_chroma_by_tone(
        palette.hue_raw(),
        palette.chroma_raw(),
        initial_tone,
        ending_tone,
    )
}

/// 既定のコントラスト比から、コントラストレベルに応じた目標を作る
pub(crate) fn get_curve(default_contrast: f64) -> Option<ContrastCurve> {
    Some(if default_contrast == 1.5 {
        ContrastCurve::new(1.5, 1.5, 3.0, 5.5)
    } else if default_contrast == 3.0 {
        ContrastCurve::new(3.0, 3.0, 4.5, 7.0)
    } else if default_contrast == 4.5 {
        ContrastCurve::new(4.5, 4.5, 7.0, 11.0)
    } else if default_contrast == 6.0 {
        ContrastCurve::new(6.0, 6.0, 7.0, 11.0)
    } else if default_contrast == 7.0 {
        ContrastCurve::new(7.0, 7.0, 11.0, 21.0)
    } else if default_contrast == 9.0 {
        ContrastCurve::new(9.0, 9.0, 11.0, 21.0)
    } else if default_contrast == 11.0 {
        ContrastCurve::new(11.0, 11.0, 21.0, 21.0)
    } else if default_contrast == 21.0 {
        ContrastCurve::new(21.0, 21.0, 21.0, 21.0)
    } else {
        // 想定外の既定値
        ContrastCurve::new(default_contrast, default_contrast, 7.0, 21.0)
    })
}

const fn curve(low: f64, normal: f64, medium: f64, high: f64) -> Option<ContrastCurve> {
    Some(ContrastCurve::new(low, normal, medium, high))
}

fn phone(s: &DynamicScheme) -> bool {
    s.platform == Platform::Phone
}

fn watch(s: &DynamicScheme) -> bool {
    s.platform == Platform::Watch
}

fn yellow_neutral(s: &DynamicScheme) -> bool {
    Hct::is_yellow(s.neutral_palette.hue_raw())
}

/// スマートフォンでは最も高いサーフェス、腕時計では高いサーフェスコンテナ
fn phone_highest_surface(s: &DynamicScheme) -> Option<R> {
    Some(if phone(s) {
        highest_surface(s)
    } else {
        R::SurfaceContainerHigh
    })
}

/// サーフェス上の文字・輪郭線に共通する彩度の倍率
fn on_surface_chroma_multiplier(s: &DynamicScheme) -> f64 {
    if phone(s) {
        match s.variant {
            Variant::Neutral => return 2.2,
            Variant::TonalSpot => return 1.7,
            Variant::Expressive => {
                return if yellow_neutral(s) {
                    if s.is_dark { 3.0 } else { 2.3 }
                } else {
                    1.6
                };
            }
            _ => {}
        }
    }
    1.0
}

/// コントラストを上げたときだけ、背景との比 1.5 を求める
fn container_curve(s: &DynamicScheme) -> Option<ContrastCurve> {
    if s.contrast_level > 0.0 {
        get_curve(1.5)
    } else {
        None
    }
}

fn delta_pair(
    role_a: R,
    role_b: R,
    delta: f64,
    polarity: TonePolarity,
    constraint: DeltaConstraint,
) -> Option<ToneDeltaPair> {
    Some(ToneDeltaPair::new(role_a, role_b, delta, polarity, true).with_constraint(constraint))
}

/// 2025 仕様での `role` の定義。2025 仕様で定義が変わらない役割は `None`
///
/// 移植元が別の役割の複製で定義する役割 (`background` など) は、`version` での複製元の定義を返す
pub(crate) fn spec(role: R, version: SpecVersion) -> Option<ColorSpec> {
    let spec = match role {
        R::Surface => ColorSpec {
            tone: Some(|s| {
                if phone(s) {
                    if s.is_dark {
                        4.0
                    } else if yellow_neutral(s) {
                        99.0
                    } else if s.variant == Variant::Vibrant {
                        97.0
                    } else {
                        98.0
                    }
                } else {
                    0.0
                }
            }),
            is_background: true,
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::SurfaceDim => ColorSpec {
            tone: Some(|s| {
                if s.is_dark || watch(s) {
                    4.0
                } else if yellow_neutral(s) {
                    90.0
                } else if s.variant == Variant::Vibrant {
                    85.0
                } else {
                    87.0
                }
            }),
            is_background: true,
            chroma_multiplier: |s| {
                if !s.is_dark && phone(s) {
                    match s.variant {
                        Variant::Neutral => return 2.5,
                        Variant::TonalSpot => return 1.7,
                        Variant::Expressive => return if yellow_neutral(s) { 2.7 } else { 1.75 },
                        Variant::Vibrant => return 1.36,
                        _ => {}
                    }
                }
                1.0
            },
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::SurfaceBright => ColorSpec {
            tone: Some(|s| {
                if s.is_dark || watch(s) {
                    18.0
                } else if yellow_neutral(s) {
                    99.0
                } else if s.variant == Variant::Vibrant {
                    97.0
                } else {
                    98.0
                }
            }),
            is_background: true,
            chroma_multiplier: |s| {
                if s.is_dark || watch(s) {
                    match s.variant {
                        Variant::Neutral => return 2.5,
                        Variant::TonalSpot => return 1.7,
                        Variant::Expressive => return if yellow_neutral(s) { 2.7 } else { 1.75 },
                        Variant::Vibrant => return 1.36,
                        _ => {}
                    }
                }
                1.0
            },
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::SurfaceContainerLowest => ColorSpec {
            tone: Some(|s| if s.is_dark || watch(s) { 0.0 } else { 100.0 }),
            is_background: true,
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::SurfaceContainerLow => ColorSpec {
            tone: Some(|s| {
                if phone(s) {
                    if s.is_dark {
                        6.0
                    } else if yellow_neutral(s) {
                        98.0
                    } else if s.variant == Variant::Vibrant {
                        95.0
                    } else {
                        96.0
                    }
                } else {
                    15.0
                }
            }),
            is_background: true,
            chroma_multiplier: |s| {
                if phone(s) {
                    match s.variant {
                        Variant::Neutral => return 1.3,
                        Variant::TonalSpot => return 1.25,
                        Variant::Expressive => return if yellow_neutral(s) { 1.3 } else { 1.15 },
                        Variant::Vibrant => return 1.08,
                        _ => {}
                    }
                }
                1.0
            },
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::SurfaceContainer => ColorSpec {
            tone: Some(|s| {
                if phone(s) {
                    if s.is_dark {
                        9.0
                    } else if yellow_neutral(s) {
                        96.0
                    } else if s.variant == Variant::Vibrant {
                        92.0
                    } else {
                        94.0
                    }
                } else {
                    20.0
                }
            }),
            is_background: true,
            chroma_multiplier: |s| {
                if phone(s) {
                    match s.variant {
                        Variant::Neutral => return 1.6,
                        Variant::TonalSpot => return 1.4,
                        Variant::Expressive => return if yellow_neutral(s) { 1.6 } else { 1.3 },
                        Variant::Vibrant => return 1.15,
                        _ => {}
                    }
                }
                1.0
            },
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::SurfaceContainerHigh => ColorSpec {
            tone: Some(|s| {
                if phone(s) {
                    if s.is_dark {
                        12.0
                    } else if yellow_neutral(s) {
                        94.0
                    } else if s.variant == Variant::Vibrant {
                        90.0
                    } else {
                        92.0
                    }
                } else {
                    25.0
                }
            }),
            is_background: true,
            chroma_multiplier: |s| {
                if phone(s) {
                    match s.variant {
                        Variant::Neutral => return 1.9,
                        Variant::TonalSpot => return 1.5,
                        Variant::Expressive => return if yellow_neutral(s) { 1.95 } else { 1.45 },
                        Variant::Vibrant => return 1.22,
                        _ => {}
                    }
                }
                1.0
            },
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::SurfaceContainerHighest => ColorSpec {
            tone: Some(|s| {
                if s.is_dark || watch(s) {
                    15.0
                } else if yellow_neutral(s) {
                    92.0
                } else if s.variant == Variant::Vibrant {
                    88.0
                } else {
                    90.0
                }
            }),
            is_background: true,
            chroma_multiplier: |s| match s.variant {
                Variant::Neutral => 2.2,
                Variant::TonalSpot => 1.7,
                Variant::Expressive => {
                    if yellow_neutral(s) {
                        2.3
                    } else {
                        1.6
                    }
                }
                Variant::Vibrant => 1.29,
                _ => 1.0,
            },
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::OnSurface => ColorSpec {
            tone: Some(|s| {
                if phone(s) && s.variant == Variant::Vibrant {
                    t_max_c(&s.neutral_palette, 0.0, 100.0, 1.1)
                } else {
                    tone_from_background(s, phone_highest_surface(s))
                }
            }),
            chroma_multiplier: on_surface_chroma_multiplier,
            background: phone_highest_surface,
            contrast_curve: |s| {
                if phone(s) {
                    if s.is_dark {
                        get_curve(11.0)
                    } else {
                        get_curve(9.0)
                    }
                } else {
                    curve(4.5, 9.0, 11.0, 21.0)
                }
            },
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::OnSurfaceVariant => ColorSpec {
            chroma_multiplier: on_surface_chroma_multiplier,
            background: phone_highest_surface,
            contrast_curve: |s| {
                if phone(s) {
                    if s.is_dark {
                        get_curve(6.0)
                    } else {
                        get_curve(4.5)
                    }
                } else {
                    curve(3.0, 7.0, 7.0, 11.0)
                }
            },
            ..ColorSpec::new(|s| {
                if watch(s) {
                    Palette::NeutralVariant
                } else {
                    Palette::Neutral
                }
            })
        },
        R::Outline => ColorSpec {
            chroma_multiplier: on_surface_chroma_multiplier,
            background: phone_highest_surface,
            contrast_curve: |s| {
                if phone(s) {
                    get_curve(3.0)
                } else {
                    curve(1.5, 4.5, 4.5, 7.0)
                }
            },
            ..ColorSpec::new(|s| {
                if watch(s) {
                    Palette::NeutralVariant
                } else {
                    Palette::Neutral
                }
            })
        },
        R::OutlineVariant => ColorSpec {
            chroma_multiplier: on_surface_chroma_multiplier,
            background: phone_highest_surface,
            contrast_curve: |s| {
                if phone(s) {
                    get_curve(1.5)
                } else {
                    curve(1.0, 3.0, 3.0, 4.5)
                }
            },
            ..ColorSpec::new(|s| {
                if watch(s) {
                    Palette::NeutralVariant
                } else {
                    Palette::Neutral
                }
            })
        },
        R::InverseSurface => ColorSpec {
            tone: Some(|s| if s.is_dark || watch(s) { 98.0 } else { 4.0 }),
            is_background: true,
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::InverseOnSurface => ColorSpec {
            background: |_| Some(R::InverseSurface),
            contrast_curve: |_| get_curve(7.0),
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::Primary => ColorSpec {
            tone: Some(|s| {
                let p = &s.primary_palette;
                if watch(s) {
                    if s.variant == Variant::Neutral {
                        90.0
                    } else {
                        main_tone(
                            p,
                            if s.variant == Variant::TonalSpot {
                                90.0
                            } else {
                                100.0
                            },
                        )
                    }
                } else if s.variant == Variant::Neutral {
                    if s.is_dark { 80.0 } else { 40.0 }
                } else if s.variant == Variant::TonalSpot {
                    if s.is_dark {
                        80.0
                    } else {
                        t_max_c(p, 0.0, 100.0, 1.0)
                    }
                } else if s.variant == Variant::Expressive {
                    let upper = if s.is_dark {
                        if Hct::is_cyan(p.hue_raw()) {
                            88.0
                        } else {
                            98.0
                        }
                    } else if Hct::is_yellow(p.hue_raw()) {
                        25.0
                    } else {
                        98.0
                    };
                    t_max_c(p, 0.0, upper, 1.0)
                } else {
                    // Vibrant
                    t_max_c(
                        p,
                        0.0,
                        if Hct::is_cyan(p.hue_raw()) {
                            88.0
                        } else {
                            98.0
                        },
                        1.0,
                    )
                }
            }),
            is_background: true,
            background: phone_highest_surface,
            contrast_curve: |s| {
                if phone(s) {
                    get_curve(4.5)
                } else {
                    get_curve(7.0)
                }
            },
            tone_delta_pair: |s| {
                if phone(s) {
                    delta_pair(
                        R::PrimaryContainer,
                        R::Primary,
                        5.0,
                        TonePolarity::RelativeLighter,
                        DeltaConstraint::Farther,
                    )
                } else {
                    None
                }
            },
            ..ColorSpec::new(|_| Palette::Primary)
        },
        R::PrimaryDim => primary_dim(),
        R::OnPrimary => ColorSpec {
            background: |s| Some(if phone(s) { R::Primary } else { R::PrimaryDim }),
            contrast_curve: |s| {
                if phone(s) {
                    get_curve(6.0)
                } else {
                    get_curve(7.0)
                }
            },
            ..ColorSpec::new(|_| Palette::Primary)
        },
        R::PrimaryContainer => ColorSpec {
            tone: Some(|s| {
                let p = &s.primary_palette;
                if watch(s) {
                    30.0
                } else if s.variant == Variant::Neutral {
                    if s.is_dark { 30.0 } else { 90.0 }
                } else if s.variant == Variant::TonalSpot {
                    if s.is_dark {
                        t_min_c(p, 35.0, 93.0)
                    } else {
                        t_max_c(p, 0.0, 90.0, 1.0)
                    }
                } else if s.variant == Variant::Expressive {
                    if s.is_dark {
                        t_min_c(p, 30.0, 93.0)
                    } else {
                        t_max_c(
                            p,
                            78.0,
                            if Hct::is_cyan(p.hue_raw()) {
                                88.0
                            } else {
                                90.0
                            },
                            1.0,
                        )
                    }
                } else if s.is_dark {
                    // Vibrant
                    t_min_c(p, 66.0, 93.0)
                } else {
                    t_max_c(
                        p,
                        66.0,
                        if Hct::is_cyan(p.hue_raw()) {
                            88.0
                        } else {
                            93.0
                        },
                        1.0,
                    )
                }
            }),
            is_background: true,
            background: phone_highest_surface,
            tone_delta_pair: |s| {
                if phone(s) {
                    None
                } else {
                    delta_pair(
                        R::PrimaryContainer,
                        R::PrimaryDim,
                        10.0,
                        TonePolarity::Darker,
                        DeltaConstraint::Farther,
                    )
                }
            },
            contrast_curve: |s| {
                if phone(s) {
                    container_curve(s)
                } else {
                    curve(1.0, 1.0, 3.0, 4.5)
                }
            },
            ..ColorSpec::new(|_| Palette::Primary)
        },
        R::OnPrimaryContainer => ColorSpec {
            background: |_| Some(R::PrimaryContainer),
            contrast_curve: |s| {
                if phone(s) {
                    get_curve(6.0)
                } else {
                    curve(3.0, 7.0, 7.0, 11.0)
                }
            },
            ..ColorSpec::new(|_| Palette::Primary)
        },
        R::PrimaryFixed => ColorSpec {
            tone: Some(|s| {
                if watch(s) {
                    return if s.variant == Variant::Neutral {
                        90.0
                    } else {
                        main_tone(
                            &s.primary_palette,
                            if s.variant == Variant::TonalSpot {
                                90.0
                            } else {
                                100.0
                            },
                        )
                    };
                }
                get_tone(&s.light_standard_contrast(), R::PrimaryContainer)
            }),
            is_background: true,
            background: phone_highest_surface,
            contrast_curve: |s| {
                if phone(s) {
                    container_curve(s)
                } else {
                    curve(7.0, 7.0, 7.0, 7.0)
                }
            },
            ..ColorSpec::new(|_| Palette::Primary)
        },
        R::PrimaryFixedDim => ColorSpec {
            tone: Some(|s| {
                if watch(s) {
                    if s.variant == Variant::Neutral {
                        85.0
                    } else {
                        dim_tone(
                            &s.primary_palette,
                            if s.variant == Variant::TonalSpot {
                                90.0
                            } else {
                                100.0
                            },
                        )
                    }
                } else {
                    get_tone(s, R::PrimaryFixed)
                }
            }),
            is_background: true,
            background: |s| {
                if watch(s) {
                    Some(R::SurfaceContainerHigh)
                } else {
                    None
                }
            },
            contrast_curve: |s| {
                if watch(s) {
                    curve(4.5, 4.5, 4.5, 4.5)
                } else {
                    None
                }
            },
            tone_delta_pair: |s| {
                if watch(s) {
                    None
                } else {
                    delta_pair(
                        R::PrimaryFixedDim,
                        R::PrimaryFixed,
                        5.0,
                        TonePolarity::Darker,
                        DeltaConstraint::Exact,
                    )
                }
            },
            ..ColorSpec::new(|_| Palette::Primary)
        },
        R::OnPrimaryFixed => ColorSpec {
            background: |_| Some(R::PrimaryFixedDim),
            second_background: |s| {
                if watch(s) {
                    Some(R::PrimaryFixed)
                } else {
                    None
                }
            },
            contrast_curve: |s| {
                if watch(s) {
                    curve(4.5, 7.0, 11.0, 21.0)
                } else {
                    get_curve(7.0)
                }
            },
            ..ColorSpec::new(|_| Palette::Primary)
        },
        R::OnPrimaryFixedVariant => ColorSpec {
            tone: Some(|s| {
                if watch(s) {
                    30.0
                } else {
                    tone_from_background(s, Some(R::PrimaryFixedDim))
                }
            }),
            background: |_| Some(R::PrimaryFixedDim),
            second_background: |s| {
                if watch(s) {
                    Some(R::PrimaryFixed)
                } else {
                    None
                }
            },
            contrast_curve: |s| {
                if watch(s) {
                    curve(3.0, 4.5, 7.0, 11.0)
                } else {
                    get_curve(4.5)
                }
            },
            ..ColorSpec::new(|_| Palette::Primary)
        },
        R::InversePrimary => ColorSpec {
            tone: Some(|s| t_max_c(&s.primary_palette, 0.0, 100.0, 1.0)),
            background: |_| Some(R::InverseSurface),
            contrast_curve: |s| {
                if phone(s) {
                    get_curve(6.0)
                } else {
                    get_curve(7.0)
                }
            },
            ..ColorSpec::new(|_| Palette::Primary)
        },
        R::Secondary => ColorSpec {
            tone: Some(|s| {
                let p = &s.secondary_palette;
                if watch(s) {
                    if s.variant == Variant::Neutral {
                        90.0
                    } else {
                        main_tone(p, 90.0)
                    }
                } else if s.variant == Variant::Neutral {
                    if s.is_dark {
                        t_min_c(p, 0.0, 98.0)
                    } else {
                        t_max_c(p, 0.0, 100.0, 1.0)
                    }
                } else if s.variant == Variant::Vibrant {
                    t_max_c(p, 0.0, if s.is_dark { 90.0 } else { 98.0 }, 1.0)
                } else if s.is_dark {
                    // Expressive と TonalSpot
                    80.0
                } else {
                    t_max_c(p, 0.0, 100.0, 1.0)
                }
            }),
            is_background: true,
            background: phone_highest_surface,
            contrast_curve: |s| {
                if phone(s) {
                    get_curve(4.5)
                } else {
                    get_curve(7.0)
                }
            },
            tone_delta_pair: |s| {
                if phone(s) {
                    delta_pair(
                        R::SecondaryContainer,
                        R::Secondary,
                        5.0,
                        TonePolarity::RelativeLighter,
                        DeltaConstraint::Farther,
                    )
                } else {
                    None
                }
            },
            ..ColorSpec::new(|_| Palette::Secondary)
        },
        R::SecondaryDim => secondary_dim(),
        R::OnSecondary => ColorSpec {
            background: |s| {
                Some(if phone(s) {
                    R::Secondary
                } else {
                    R::SecondaryDim
                })
            },
            contrast_curve: |s| {
                if phone(s) {
                    get_curve(6.0)
                } else {
                    get_curve(7.0)
                }
            },
            ..ColorSpec::new(|_| Palette::Secondary)
        },
        R::SecondaryContainer => ColorSpec {
            tone: Some(|s| {
                let p = &s.secondary_palette;
                if watch(s) {
                    30.0
                } else if s.variant == Variant::Vibrant {
                    if s.is_dark {
                        t_min_c(p, 30.0, 40.0)
                    } else {
                        t_max_c(p, 84.0, 90.0, 1.0)
                    }
                } else if s.variant == Variant::Expressive {
                    if s.is_dark {
                        15.0
                    } else {
                        t_max_c(p, 90.0, 95.0, 1.0)
                    }
                } else if s.is_dark {
                    25.0
                } else {
                    90.0
                }
            }),
            is_background: true,
            background: phone_highest_surface,
            tone_delta_pair: |s| {
                if watch(s) {
                    delta_pair(
                        R::SecondaryContainer,
                        R::SecondaryDim,
                        10.0,
                        TonePolarity::Darker,
                        DeltaConstraint::Farther,
                    )
                } else {
                    None
                }
            },
            contrast_curve: |s| {
                if phone(s) {
                    container_curve(s)
                } else {
                    curve(1.0, 1.0, 3.0, 4.5)
                }
            },
            ..ColorSpec::new(|_| Palette::Secondary)
        },
        R::OnSecondaryContainer => ColorSpec {
            background: |_| Some(R::SecondaryContainer),
            contrast_curve: |s| {
                if phone(s) {
                    get_curve(6.0)
                } else {
                    curve(3.0, 7.0, 7.0, 11.0)
                }
            },
            ..ColorSpec::new(|_| Palette::Secondary)
        },
        R::SecondaryFixed => ColorSpec {
            tone: Some(|s| {
                if watch(s) {
                    return if s.variant == Variant::Neutral {
                        90.0
                    } else {
                        main_tone(&s.secondary_palette, 90.0)
                    };
                }
                get_tone(&s.light_standard_contrast(), R::SecondaryContainer)
            }),
            is_background: true,
            background: phone_highest_surface,
            contrast_curve: |s| {
                if phone(s) {
                    container_curve(s)
                } else {
                    curve(7.0, 7.0, 7.0, 7.0)
                }
            },
            ..ColorSpec::new(|_| Palette::Secondary)
        },
        R::SecondaryFixedDim => ColorSpec {
            tone: Some(|s| {
                if watch(s) {
                    if s.variant == Variant::Neutral {
                        85.0
                    } else {
                        dim_tone(&s.secondary_palette, 90.0)
                    }
                } else {
                    get_tone(s, R::SecondaryFixed)
                }
            }),
            is_background: true,
            background: |s| {
                if watch(s) {
                    Some(R::SurfaceContainerHigh)
                } else {
                    None
                }
            },
            contrast_curve: |s| {
                if watch(s) {
                    curve(4.5, 4.5, 4.5, 4.5)
                } else {
                    None
                }
            },
            tone_delta_pair: |s| {
                if watch(s) {
                    None
                } else {
                    delta_pair(
                        R::SecondaryFixedDim,
                        R::SecondaryFixed,
                        5.0,
                        TonePolarity::Darker,
                        DeltaConstraint::Exact,
                    )
                }
            },
            ..ColorSpec::new(|_| Palette::Secondary)
        },
        R::OnSecondaryFixed => ColorSpec {
            background: |_| Some(R::SecondaryFixedDim),
            second_background: |s| {
                if watch(s) {
                    Some(R::SecondaryFixed)
                } else {
                    None
                }
            },
            contrast_curve: |s| {
                if watch(s) {
                    curve(4.5, 7.0, 11.0, 21.0)
                } else {
                    get_curve(7.0)
                }
            },
            ..ColorSpec::new(|_| Palette::Secondary)
        },
        R::OnSecondaryFixedVariant => ColorSpec {
            tone: Some(|s| {
                if watch(s) {
                    30.0
                } else {
                    tone_from_background(s, Some(R::SecondaryFixedDim))
                }
            }),
            background: |_| Some(R::SecondaryFixedDim),
            second_background: |s| {
                if watch(s) {
                    Some(R::SecondaryFixed)
                } else {
                    None
                }
            },
            contrast_curve: |s| {
                if watch(s) {
                    curve(3.0, 4.5, 7.0, 11.0)
                } else {
                    get_curve(4.5)
                }
            },
            ..ColorSpec::new(|_| Palette::Secondary)
        },
        R::Tertiary => ColorSpec {
            tone: Some(|s| {
                let p = &s.tertiary_palette;
                if watch(s) {
                    main_tone(
                        p,
                        if s.variant == Variant::TonalSpot {
                            90.0
                        } else {
                            100.0
                        },
                    )
                } else if s.variant == Variant::Expressive || s.variant == Variant::Vibrant {
                    let upper = if Hct::is_cyan(p.hue_raw()) {
                        88.0
                    } else if s.is_dark {
                        98.0
                    } else {
                        100.0
                    };
                    t_max_c(p, 0.0, upper, 1.0)
                } else if s.is_dark {
                    // Neutral と TonalSpot
                    t_max_c(p, 0.0, 98.0, 1.0)
                } else {
                    t_max_c(p, 0.0, 100.0, 1.0)
                }
            }),
            is_background: true,
            background: phone_highest_surface,
            contrast_curve: |s| {
                if phone(s) {
                    get_curve(4.5)
                } else {
                    get_curve(7.0)
                }
            },
            tone_delta_pair: |s| {
                if phone(s) {
                    delta_pair(
                        R::TertiaryContainer,
                        R::Tertiary,
                        5.0,
                        TonePolarity::RelativeLighter,
                        DeltaConstraint::Farther,
                    )
                } else {
                    None
                }
            },
            ..ColorSpec::new(|_| Palette::Tertiary)
        },
        R::TertiaryDim => tertiary_dim(),
        R::OnTertiary => ColorSpec {
            background: |s| {
                Some(if phone(s) {
                    R::Tertiary
                } else {
                    R::TertiaryDim
                })
            },
            contrast_curve: |s| {
                if phone(s) {
                    get_curve(6.0)
                } else {
                    get_curve(7.0)
                }
            },
            ..ColorSpec::new(|_| Palette::Tertiary)
        },
        R::TertiaryContainer => ColorSpec {
            tone: Some(|s| {
                let p = &s.tertiary_palette;
                if watch(s) {
                    container_tone(
                        p,
                        if s.variant == Variant::TonalSpot {
                            90.0
                        } else {
                            100.0
                        },
                    )
                } else if s.variant == Variant::Neutral {
                    if s.is_dark {
                        t_max_c(p, 0.0, 93.0, 1.0)
                    } else {
                        t_max_c(p, 0.0, 96.0, 1.0)
                    }
                } else if s.variant == Variant::TonalSpot {
                    t_max_c(p, 0.0, if s.is_dark { 93.0 } else { 100.0 }, 1.0)
                } else if s.variant == Variant::Expressive {
                    let upper = if Hct::is_cyan(p.hue_raw()) {
                        88.0
                    } else if s.is_dark {
                        93.0
                    } else {
                        100.0
                    };
                    t_max_c(p, 75.0, upper, 1.0)
                } else if s.is_dark {
                    // Vibrant
                    t_max_c(p, 0.0, 93.0, 1.0)
                } else {
                    t_max_c(p, 72.0, 100.0, 1.0)
                }
            }),
            is_background: true,
            background: phone_highest_surface,
            contrast_curve: |s| {
                if phone(s) {
                    container_curve(s)
                } else {
                    curve(1.0, 1.0, 3.0, 4.5)
                }
            },
            ..ColorSpec::new(|_| Palette::Tertiary)
        },
        R::OnTertiaryContainer => ColorSpec {
            background: |_| Some(R::TertiaryContainer),
            contrast_curve: |s| {
                if phone(s) {
                    get_curve(6.0)
                } else {
                    curve(3.0, 7.0, 7.0, 11.0)
                }
            },
            ..ColorSpec::new(|_| Palette::Tertiary)
        },
        R::TertiaryFixed => ColorSpec {
            tone: Some(|s| {
                if watch(s) {
                    return main_tone(
                        &s.tertiary_palette,
                        if s.variant == Variant::TonalSpot {
                            90.0
                        } else {
                            100.0
                        },
                    );
                }
                get_tone(&s.light_standard_contrast(), R::TertiaryContainer)
            }),
            is_background: true,
            background: phone_highest_surface,
            contrast_curve: |s| {
                if phone(s) {
                    container_curve(s)
                } else {
                    curve(7.0, 7.0, 7.0, 7.0)
                }
            },
            ..ColorSpec::new(|_| Palette::Tertiary)
        },
        R::TertiaryFixedDim => ColorSpec {
            tone: Some(|s| {
                if watch(s) {
                    dim_tone(
                        &s.tertiary_palette,
                        if s.variant == Variant::TonalSpot {
                            90.0
                        } else {
                            100.0
                        },
                    )
                } else {
                    get_tone(s, R::TertiaryFixed)
                }
            }),
            is_background: true,
            background: |s| {
                if watch(s) {
                    Some(R::SurfaceContainerHigh)
                } else {
                    None
                }
            },
            contrast_curve: |s| {
                if watch(s) {
                    curve(4.5, 4.5, 4.5, 4.5)
                } else {
                    None
                }
            },
            tone_delta_pair: |s| {
                if watch(s) {
                    None
                } else {
                    delta_pair(
                        R::TertiaryFixedDim,
                        R::TertiaryFixed,
                        5.0,
                        TonePolarity::Darker,
                        DeltaConstraint::Exact,
                    )
                }
            },
            ..ColorSpec::new(|_| Palette::Tertiary)
        },
        R::OnTertiaryFixed => ColorSpec {
            background: |_| Some(R::TertiaryFixedDim),
            second_background: |s| {
                if watch(s) {
                    Some(R::TertiaryFixed)
                } else {
                    None
                }
            },
            contrast_curve: |s| {
                if watch(s) {
                    curve(4.5, 7.0, 11.0, 21.0)
                } else {
                    get_curve(7.0)
                }
            },
            ..ColorSpec::new(|_| Palette::Tertiary)
        },
        R::OnTertiaryFixedVariant => ColorSpec {
            tone: Some(|s| {
                if watch(s) {
                    30.0
                } else {
                    tone_from_background(s, Some(R::TertiaryFixedDim))
                }
            }),
            background: |_| Some(R::TertiaryFixedDim),
            second_background: |s| {
                if watch(s) {
                    Some(R::TertiaryFixed)
                } else {
                    None
                }
            },
            contrast_curve: |s| {
                if watch(s) {
                    curve(3.0, 4.5, 7.0, 11.0)
                } else {
                    get_curve(4.5)
                }
            },
            ..ColorSpec::new(|_| Palette::Tertiary)
        },
        R::Error => ColorSpec {
            tone: Some(|s| {
                let p = &s.error_palette;
                if phone(s) {
                    if s.is_dark {
                        t_min_c(p, 0.0, 98.0)
                    } else {
                        t_max_c(p, 0.0, 100.0, 1.0)
                    }
                } else {
                    let starting_tone = foreground_tone(25.0, 7.0);
                    find_desired_chroma_by_tone(p.hue_raw(), p.chroma_raw(), starting_tone, 100.0)
                }
            }),
            is_background: true,
            background: phone_highest_surface,
            contrast_curve: |s| {
                if phone(s) {
                    get_curve(4.5)
                } else {
                    curve(3.0, 7.0, 7.0, 7.0)
                }
            },
            tone_delta_pair: |s| {
                if phone(s) {
                    delta_pair(
                        R::ErrorContainer,
                        R::Error,
                        5.0,
                        TonePolarity::RelativeLighter,
                        DeltaConstraint::Farther,
                    )
                } else {
                    None
                }
            },
            ..ColorSpec::new(|_| Palette::Error)
        },
        R::ErrorDim => error_dim(),
        R::OnError => ColorSpec {
            background: |s| Some(if phone(s) { R::Error } else { R::ErrorDim }),
            contrast_curve: |s| {
                if phone(s) {
                    get_curve(6.0)
                } else {
                    curve(4.5, 7.0, 11.0, 21.0)
                }
            },
            ..ColorSpec::new(|_| Palette::Error)
        },
        R::ErrorContainer => ColorSpec {
            tone: Some(|s| {
                if watch(s) {
                    30.0
                } else if s.is_dark {
                    t_min_c(&s.error_palette, 30.0, 93.0)
                } else {
                    t_max_c(&s.error_palette, 0.0, 90.0, 1.0)
                }
            }),
            is_background: true,
            background: phone_highest_surface,
            tone_delta_pair: |s| {
                if watch(s) {
                    delta_pair(
                        R::ErrorContainer,
                        R::ErrorDim,
                        10.0,
                        TonePolarity::Darker,
                        DeltaConstraint::Farther,
                    )
                } else {
                    None
                }
            },
            contrast_curve: |s| {
                if phone(s) {
                    container_curve(s)
                } else {
                    curve(1.0, 1.0, 3.0, 4.5)
                }
            },
            ..ColorSpec::new(|_| Palette::Error)
        },
        R::OnErrorContainer => ColorSpec {
            background: |_| Some(R::ErrorContainer),
            contrast_curve: |s| {
                if phone(s) {
                    get_curve(4.5)
                } else {
                    curve(3.0, 7.0, 7.0, 11.0)
                }
            },
            ..ColorSpec::new(|_| Palette::Error)
        },
        // 移植元は surfaceContainerHighest の複製に名前を付け直して使う
        R::SurfaceVariant => spec_of(R::SurfaceContainerHighest, version),
        // 移植元は primary の複製に名前を付け直して使う
        R::SurfaceTint => spec_of(R::Primary, version),
        // 移植元は surface の複製に名前を付け直して使う
        R::Background => spec_of(R::Surface, version),
        // 移植元は onSurface の複製のトーンだけを差し替えて使う
        R::OnBackground => ColorSpec {
            tone: Some(|s| {
                if watch(s) {
                    100.0
                } else {
                    get_tone(s, R::OnSurface)
                }
            }),
            ..spec_of(R::OnSurface, version)
        },
        R::PrimaryPaletteKeyColor
        | R::SecondaryPaletteKeyColor
        | R::TertiaryPaletteKeyColor
        | R::NeutralPaletteKeyColor
        | R::NeutralVariantPaletteKeyColor
        | R::ErrorPaletteKeyColor
        | R::Shadow
        | R::Scrim => return None,
    };
    Some(spec)
}

/// 2025 仕様で加わった `primary_dim` の定義
pub(crate) fn primary_dim() -> ColorSpec {
    ColorSpec {
        tone: Some(|s| {
            let p = &s.primary_palette;
            if watch(s) {
                if s.variant == Variant::Neutral {
                    85.0
                } else {
                    dim_tone(
                        p,
                        if s.variant == Variant::TonalSpot {
                            90.0
                        } else {
                            100.0
                        },
                    )
                }
            } else if s.variant == Variant::Neutral {
                85.0
            } else if s.variant == Variant::TonalSpot {
                t_max_c(p, 0.0, 90.0, 1.0)
            } else {
                t_max_c(p, 0.0, 100.0, 1.0)
            }
        }),
        is_background: true,
        background: |_| Some(R::SurfaceContainerHigh),
        contrast_curve: |s| {
            if watch(s) {
                curve(4.5, 4.5, 4.5, 4.5)
            } else {
                get_curve(4.5)
            }
        },
        tone_delta_pair: |s| {
            if watch(s) {
                None
            } else {
                delta_pair(
                    R::PrimaryDim,
                    R::Primary,
                    5.0,
                    TonePolarity::Darker,
                    DeltaConstraint::Farther,
                )
            }
        },
        ..ColorSpec::new(|_| Palette::Primary)
    }
}

/// 2025 仕様で加わった `secondary_dim` の定義
pub(crate) fn secondary_dim() -> ColorSpec {
    ColorSpec {
        tone: Some(|s| {
            let p = &s.secondary_palette;
            if watch(s) {
                if s.variant == Variant::Neutral {
                    85.0
                } else {
                    dim_tone(p, 90.0)
                }
            } else if s.variant == Variant::Neutral {
                85.0
            } else {
                t_max_c(p, 0.0, 90.0, 1.0)
            }
        }),
        is_background: true,
        background: |_| Some(R::SurfaceContainerHigh),
        contrast_curve: |s| {
            if watch(s) {
                curve(4.5, 4.5, 4.5, 4.5)
            } else {
                get_curve(4.5)
            }
        },
        tone_delta_pair: |s| {
            if watch(s) {
                None
            } else {
                delta_pair(
                    R::SecondaryDim,
                    R::Secondary,
                    5.0,
                    TonePolarity::Darker,
                    DeltaConstraint::Farther,
                )
            }
        },
        ..ColorSpec::new(|_| Palette::Secondary)
    }
}

/// 2025 仕様で加わった `tertiary_dim` の定義
pub(crate) fn tertiary_dim() -> ColorSpec {
    ColorSpec {
        tone: Some(|s| {
            let p = &s.tertiary_palette;
            if watch(s) {
                dim_tone(
                    p,
                    if s.variant == Variant::TonalSpot {
                        90.0
                    } else {
                        100.0
                    },
                )
            } else if s.variant == Variant::TonalSpot {
                t_max_c(p, 0.0, 90.0, 1.0)
            } else {
                t_max_c(p, 0.0, 100.0, 1.0)
            }
        }),
        is_background: true,
        background: |_| Some(R::SurfaceContainerHigh),
        contrast_curve: |s| {
            if watch(s) {
                curve(4.5, 4.5, 4.5, 4.5)
            } else {
                get_curve(4.5)
            }
        },
        tone_delta_pair: |s| {
            if watch(s) {
                None
            } else {
                delta_pair(
                    R::TertiaryDim,
                    R::Tertiary,
                    5.0,
                    TonePolarity::Darker,
                    DeltaConstraint::Farther,
                )
            }
        },
        ..ColorSpec::new(|_| Palette::Tertiary)
    }
}

/// 2025 仕様で加わった `error_dim` の定義
pub(crate) fn error_dim() -> ColorSpec {
    ColorSpec {
        tone: Some(|s| t_min_c(&s.error_palette, 0.0, 100.0)),
        is_background: true,
        background: |_| Some(R::SurfaceContainerHigh),
        contrast_curve: |_| get_curve(4.5),
        tone_delta_pair: |_| {
            delta_pair(
                R::ErrorDim,
                R::Error,
                5.0,
                TonePolarity::Darker,
                DeltaConstraint::Farther,
            )
        },
        ..ColorSpec::new(|_| Palette::Error)
    }
}
