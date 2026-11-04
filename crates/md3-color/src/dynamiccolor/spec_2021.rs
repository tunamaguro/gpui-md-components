// Ported from material-color-utilities
// (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/dynamiccolor/color_spec_2021.ts (ColorSpecDelegateImpl2021)
//         @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2025 Google LLC. Licensed under the Apache License, Version 2.0.

//! 2021 仕様の色の役割の定義

use super::calc::{foreground_tone, initial_tone};
use super::contrast_curve::ContrastCurve;
use super::scheme::Palette;
use super::spec::ColorSpec;
use super::tone_delta_pair::{ToneDeltaPair, TonePolarity};
use super::{ColorRole as R, DynamicScheme, Variant, spec_2025};
use crate::dislike;
use crate::hct::Hct;
use crate::utils::math;

fn is_fidelity(s: &DynamicScheme) -> bool {
    s.variant == Variant::Fidelity || s.variant == Variant::Content
}

fn is_monochrome(s: &DynamicScheme) -> bool {
    s.variant == Variant::Monochrome
}

/// `tone` から 1 ずつトーンを動かし、指定の彩度に最も近い彩度を出せるトーンを探す
fn find_desired_chroma_by_tone(hue: f64, chroma: f64, tone: f64, by_decreasing_tone: bool) -> f64 {
    let mut answer = tone;
    let mut closest_to_chroma = Hct::from_hct_raw(hue, chroma, tone);
    if closest_to_chroma.chroma_raw() < chroma {
        let mut chroma_peak = closest_to_chroma.chroma_raw();
        while closest_to_chroma.chroma_raw() < chroma {
            answer += if by_decreasing_tone { -1.0 } else { 1.0 };
            let potential_solution = Hct::from_hct_raw(hue, chroma, answer);
            if chroma_peak > potential_solution.chroma_raw() {
                break;
            }
            if (potential_solution.chroma_raw() - chroma).abs() < 0.4 {
                break;
            }

            let potential_delta = (potential_solution.chroma_raw() - chroma).abs();
            let current_delta = (closest_to_chroma.chroma_raw() - chroma).abs();
            if potential_delta < current_delta {
                closest_to_chroma = potential_solution;
            }
            chroma_peak = math::max_js(chroma_peak, potential_solution.chroma_raw());
        }
    }
    answer
}

pub(crate) fn highest_surface(s: &DynamicScheme) -> R {
    if s.is_dark {
        R::SurfaceBright
    } else {
        R::SurfaceDim
    }
}

fn dark_light(s: &DynamicScheme, dark: f64, light: f64) -> f64 {
    if s.is_dark { dark } else { light }
}

const fn curve(low: f64, normal: f64, medium: f64, high: f64) -> Option<ContrastCurve> {
    Some(ContrastCurve::new(low, normal, medium, high))
}

fn pair(
    role_a: R,
    role_b: R,
    polarity: TonePolarity,
    stay_together: bool,
) -> Option<ToneDeltaPair> {
    Some(ToneDeltaPair::new(
        role_a,
        role_b,
        10.0,
        polarity,
        stay_together,
    ))
}

/// 2021 仕様での `role` の定義
///
/// 2021 仕様に dim の役割はない。移植元の `MaterialDynamicColors` は 2021 仕様の配色でも
/// 2025 仕様の定義で dim の役割を計算するため、2025 仕様の定義を返す
pub(crate) fn spec(role: R) -> ColorSpec {
    match role {
        R::PrimaryPaletteKeyColor => ColorSpec {
            tone: Some(|s| s.primary_palette.key_color().tone_raw()),
            ..ColorSpec::new(|_| Palette::Primary)
        },
        R::SecondaryPaletteKeyColor => ColorSpec {
            tone: Some(|s| s.secondary_palette.key_color().tone_raw()),
            ..ColorSpec::new(|_| Palette::Secondary)
        },
        R::TertiaryPaletteKeyColor => ColorSpec {
            tone: Some(|s| s.tertiary_palette.key_color().tone_raw()),
            ..ColorSpec::new(|_| Palette::Tertiary)
        },
        R::NeutralPaletteKeyColor => ColorSpec {
            tone: Some(|s| s.neutral_palette.key_color().tone_raw()),
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::NeutralVariantPaletteKeyColor => ColorSpec {
            tone: Some(|s| s.neutral_variant_palette.key_color().tone_raw()),
            ..ColorSpec::new(|_| Palette::NeutralVariant)
        },
        R::ErrorPaletteKeyColor => ColorSpec {
            tone: Some(|s| s.error_palette.key_color().tone_raw()),
            ..ColorSpec::new(|_| Palette::Error)
        },
        R::Background => ColorSpec {
            tone: Some(|s| dark_light(s, 6.0, 98.0)),
            is_background: true,
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::OnBackground => ColorSpec {
            tone: Some(|s| dark_light(s, 90.0, 10.0)),
            background: |_| Some(R::Background),
            contrast_curve: |_| curve(3.0, 3.0, 4.5, 7.0),
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::Surface => ColorSpec {
            tone: Some(|s| dark_light(s, 6.0, 98.0)),
            is_background: true,
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::SurfaceDim => ColorSpec {
            tone: Some(|s| {
                if s.is_dark {
                    6.0
                } else {
                    ContrastCurve::new(87.0, 87.0, 80.0, 75.0).get(s.contrast_level)
                }
            }),
            is_background: true,
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::SurfaceBright => ColorSpec {
            tone: Some(|s| {
                if s.is_dark {
                    ContrastCurve::new(24.0, 24.0, 29.0, 34.0).get(s.contrast_level)
                } else {
                    98.0
                }
            }),
            is_background: true,
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::SurfaceContainerLowest => ColorSpec {
            tone: Some(|s| {
                if s.is_dark {
                    ContrastCurve::new(4.0, 4.0, 2.0, 0.0).get(s.contrast_level)
                } else {
                    100.0
                }
            }),
            is_background: true,
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::SurfaceContainerLow => ColorSpec {
            tone: Some(|s| {
                if s.is_dark {
                    ContrastCurve::new(10.0, 10.0, 11.0, 12.0).get(s.contrast_level)
                } else {
                    ContrastCurve::new(96.0, 96.0, 96.0, 95.0).get(s.contrast_level)
                }
            }),
            is_background: true,
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::SurfaceContainer => ColorSpec {
            tone: Some(|s| {
                if s.is_dark {
                    ContrastCurve::new(12.0, 12.0, 16.0, 20.0).get(s.contrast_level)
                } else {
                    ContrastCurve::new(94.0, 94.0, 92.0, 90.0).get(s.contrast_level)
                }
            }),
            is_background: true,
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::SurfaceContainerHigh => ColorSpec {
            tone: Some(|s| {
                if s.is_dark {
                    ContrastCurve::new(17.0, 17.0, 21.0, 25.0).get(s.contrast_level)
                } else {
                    ContrastCurve::new(92.0, 92.0, 88.0, 85.0).get(s.contrast_level)
                }
            }),
            is_background: true,
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::SurfaceContainerHighest => ColorSpec {
            tone: Some(|s| {
                if s.is_dark {
                    ContrastCurve::new(22.0, 22.0, 26.0, 30.0).get(s.contrast_level)
                } else {
                    ContrastCurve::new(90.0, 90.0, 84.0, 80.0).get(s.contrast_level)
                }
            }),
            is_background: true,
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::OnSurface => ColorSpec {
            tone: Some(|s| dark_light(s, 90.0, 10.0)),
            background: |s| Some(highest_surface(s)),
            contrast_curve: |_| curve(4.5, 7.0, 11.0, 21.0),
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::SurfaceVariant => ColorSpec {
            tone: Some(|s| dark_light(s, 30.0, 90.0)),
            is_background: true,
            ..ColorSpec::new(|_| Palette::NeutralVariant)
        },
        R::OnSurfaceVariant => ColorSpec {
            tone: Some(|s| dark_light(s, 80.0, 30.0)),
            background: |s| Some(highest_surface(s)),
            contrast_curve: |_| curve(3.0, 4.5, 7.0, 11.0),
            ..ColorSpec::new(|_| Palette::NeutralVariant)
        },
        R::InverseSurface => ColorSpec {
            tone: Some(|s| dark_light(s, 90.0, 20.0)),
            is_background: true,
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::InverseOnSurface => ColorSpec {
            tone: Some(|s| dark_light(s, 20.0, 95.0)),
            background: |_| Some(R::InverseSurface),
            contrast_curve: |_| curve(4.5, 7.0, 11.0, 21.0),
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::Outline => ColorSpec {
            tone: Some(|s| dark_light(s, 60.0, 50.0)),
            background: |s| Some(highest_surface(s)),
            contrast_curve: |_| curve(1.5, 3.0, 4.5, 7.0),
            ..ColorSpec::new(|_| Palette::NeutralVariant)
        },
        R::OutlineVariant => ColorSpec {
            tone: Some(|s| dark_light(s, 30.0, 80.0)),
            background: |s| Some(highest_surface(s)),
            contrast_curve: |_| curve(1.0, 1.0, 3.0, 4.5),
            ..ColorSpec::new(|_| Palette::NeutralVariant)
        },
        R::Shadow => ColorSpec {
            tone: Some(|_| 0.0),
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::Scrim => ColorSpec {
            tone: Some(|_| 0.0),
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::SurfaceTint => ColorSpec {
            tone: Some(|s| dark_light(s, 80.0, 40.0)),
            is_background: true,
            ..ColorSpec::new(|_| Palette::Primary)
        },
        R::Primary => ColorSpec {
            tone: Some(|s| {
                if is_monochrome(s) {
                    return dark_light(s, 100.0, 0.0);
                }
                dark_light(s, 80.0, 40.0)
            }),
            is_background: true,
            background: |s| Some(highest_surface(s)),
            contrast_curve: |_| curve(3.0, 4.5, 7.0, 7.0),
            tone_delta_pair: |_| pair(R::PrimaryContainer, R::Primary, TonePolarity::Nearer, false),
            ..ColorSpec::new(|_| Palette::Primary)
        },
        R::PrimaryDim => spec_2025::primary_dim(),
        R::OnPrimary => ColorSpec {
            tone: Some(|s| {
                if is_monochrome(s) {
                    return dark_light(s, 10.0, 90.0);
                }
                dark_light(s, 20.0, 100.0)
            }),
            background: |_| Some(R::Primary),
            contrast_curve: |_| curve(4.5, 7.0, 11.0, 21.0),
            ..ColorSpec::new(|_| Palette::Primary)
        },
        R::PrimaryContainer => ColorSpec {
            tone: Some(|s| {
                if is_fidelity(s) {
                    return s.source_color_hct.tone_raw();
                }
                if is_monochrome(s) {
                    return dark_light(s, 85.0, 25.0);
                }
                dark_light(s, 30.0, 90.0)
            }),
            is_background: true,
            background: |s| Some(highest_surface(s)),
            contrast_curve: |_| curve(1.0, 1.0, 3.0, 4.5),
            tone_delta_pair: |_| pair(R::PrimaryContainer, R::Primary, TonePolarity::Nearer, false),
            ..ColorSpec::new(|_| Palette::Primary)
        },
        R::OnPrimaryContainer => ColorSpec {
            tone: Some(|s| {
                if is_fidelity(s) {
                    return foreground_tone(initial_tone(s, R::PrimaryContainer), 4.5);
                }
                if is_monochrome(s) {
                    return dark_light(s, 0.0, 100.0);
                }
                dark_light(s, 90.0, 30.0)
            }),
            background: |_| Some(R::PrimaryContainer),
            contrast_curve: |_| curve(3.0, 4.5, 7.0, 11.0),
            ..ColorSpec::new(|_| Palette::Primary)
        },
        R::InversePrimary => ColorSpec {
            tone: Some(|s| dark_light(s, 40.0, 80.0)),
            background: |_| Some(R::InverseSurface),
            contrast_curve: |_| curve(3.0, 4.5, 7.0, 7.0),
            ..ColorSpec::new(|_| Palette::Primary)
        },
        R::Secondary => ColorSpec {
            tone: Some(|s| dark_light(s, 80.0, 40.0)),
            is_background: true,
            background: |s| Some(highest_surface(s)),
            contrast_curve: |_| curve(3.0, 4.5, 7.0, 7.0),
            tone_delta_pair: |_| {
                pair(
                    R::SecondaryContainer,
                    R::Secondary,
                    TonePolarity::Nearer,
                    false,
                )
            },
            ..ColorSpec::new(|_| Palette::Secondary)
        },
        R::SecondaryDim => spec_2025::secondary_dim(),
        R::OnSecondary => ColorSpec {
            tone: Some(|s| {
                if is_monochrome(s) {
                    dark_light(s, 10.0, 100.0)
                } else {
                    dark_light(s, 20.0, 100.0)
                }
            }),
            background: |_| Some(R::Secondary),
            contrast_curve: |_| curve(4.5, 7.0, 11.0, 21.0),
            ..ColorSpec::new(|_| Palette::Secondary)
        },
        R::SecondaryContainer => ColorSpec {
            tone: Some(|s| {
                let initial_tone = dark_light(s, 30.0, 90.0);
                if is_monochrome(s) {
                    return dark_light(s, 30.0, 85.0);
                }
                if !is_fidelity(s) {
                    return initial_tone;
                }
                find_desired_chroma_by_tone(
                    s.secondary_palette.hue_raw(),
                    s.secondary_palette.chroma_raw(),
                    initial_tone,
                    !s.is_dark,
                )
            }),
            is_background: true,
            background: |s| Some(highest_surface(s)),
            contrast_curve: |_| curve(1.0, 1.0, 3.0, 4.5),
            tone_delta_pair: |_| {
                pair(
                    R::SecondaryContainer,
                    R::Secondary,
                    TonePolarity::Nearer,
                    false,
                )
            },
            ..ColorSpec::new(|_| Palette::Secondary)
        },
        R::OnSecondaryContainer => ColorSpec {
            tone: Some(|s| {
                if is_monochrome(s) {
                    return dark_light(s, 90.0, 10.0);
                }
                if !is_fidelity(s) {
                    return dark_light(s, 90.0, 30.0);
                }
                foreground_tone(initial_tone(s, R::SecondaryContainer), 4.5)
            }),
            background: |_| Some(R::SecondaryContainer),
            contrast_curve: |_| curve(3.0, 4.5, 7.0, 11.0),
            ..ColorSpec::new(|_| Palette::Secondary)
        },
        R::Tertiary => ColorSpec {
            tone: Some(|s| {
                if is_monochrome(s) {
                    return dark_light(s, 90.0, 25.0);
                }
                dark_light(s, 80.0, 40.0)
            }),
            is_background: true,
            background: |s| Some(highest_surface(s)),
            contrast_curve: |_| curve(3.0, 4.5, 7.0, 7.0),
            tone_delta_pair: |_| {
                pair(
                    R::TertiaryContainer,
                    R::Tertiary,
                    TonePolarity::Nearer,
                    false,
                )
            },
            ..ColorSpec::new(|_| Palette::Tertiary)
        },
        R::TertiaryDim => spec_2025::tertiary_dim(),
        R::OnTertiary => ColorSpec {
            tone: Some(|s| {
                if is_monochrome(s) {
                    return dark_light(s, 10.0, 90.0);
                }
                dark_light(s, 20.0, 100.0)
            }),
            background: |_| Some(R::Tertiary),
            contrast_curve: |_| curve(4.5, 7.0, 11.0, 21.0),
            ..ColorSpec::new(|_| Palette::Tertiary)
        },
        R::TertiaryContainer => ColorSpec {
            tone: Some(|s| {
                if is_monochrome(s) {
                    return dark_light(s, 60.0, 49.0);
                }
                if !is_fidelity(s) {
                    return dark_light(s, 30.0, 90.0);
                }
                let proposed_hct = s.tertiary_palette.hct_raw(s.source_color_hct.tone_raw());
                dislike::fix_if_disliked(proposed_hct).tone_raw()
            }),
            is_background: true,
            background: |s| Some(highest_surface(s)),
            contrast_curve: |_| curve(1.0, 1.0, 3.0, 4.5),
            tone_delta_pair: |_| {
                pair(
                    R::TertiaryContainer,
                    R::Tertiary,
                    TonePolarity::Nearer,
                    false,
                )
            },
            ..ColorSpec::new(|_| Palette::Tertiary)
        },
        R::OnTertiaryContainer => ColorSpec {
            tone: Some(|s| {
                if is_monochrome(s) {
                    return dark_light(s, 0.0, 100.0);
                }
                if !is_fidelity(s) {
                    return dark_light(s, 90.0, 30.0);
                }
                foreground_tone(initial_tone(s, R::TertiaryContainer), 4.5)
            }),
            background: |_| Some(R::TertiaryContainer),
            contrast_curve: |_| curve(3.0, 4.5, 7.0, 11.0),
            ..ColorSpec::new(|_| Palette::Tertiary)
        },
        R::Error => ColorSpec {
            tone: Some(|s| dark_light(s, 80.0, 40.0)),
            is_background: true,
            background: |s| Some(highest_surface(s)),
            contrast_curve: |_| curve(3.0, 4.5, 7.0, 7.0),
            tone_delta_pair: |_| pair(R::ErrorContainer, R::Error, TonePolarity::Nearer, false),
            ..ColorSpec::new(|_| Palette::Error)
        },
        R::ErrorDim => spec_2025::error_dim(),
        R::OnError => ColorSpec {
            tone: Some(|s| dark_light(s, 20.0, 100.0)),
            background: |_| Some(R::Error),
            contrast_curve: |_| curve(4.5, 7.0, 11.0, 21.0),
            ..ColorSpec::new(|_| Palette::Error)
        },
        R::ErrorContainer => ColorSpec {
            tone: Some(|s| dark_light(s, 30.0, 90.0)),
            is_background: true,
            background: |s| Some(highest_surface(s)),
            contrast_curve: |_| curve(1.0, 1.0, 3.0, 4.5),
            tone_delta_pair: |_| pair(R::ErrorContainer, R::Error, TonePolarity::Nearer, false),
            ..ColorSpec::new(|_| Palette::Error)
        },
        R::OnErrorContainer => ColorSpec {
            tone: Some(|s| {
                if is_monochrome(s) {
                    return dark_light(s, 90.0, 10.0);
                }
                dark_light(s, 90.0, 30.0)
            }),
            background: |_| Some(R::ErrorContainer),
            contrast_curve: |_| curve(3.0, 4.5, 7.0, 11.0),
            ..ColorSpec::new(|_| Palette::Error)
        },
        R::PrimaryFixed => ColorSpec {
            tone: Some(|s| if is_monochrome(s) { 40.0 } else { 90.0 }),
            is_background: true,
            background: |s| Some(highest_surface(s)),
            contrast_curve: |_| curve(1.0, 1.0, 3.0, 4.5),
            tone_delta_pair: |_| {
                pair(
                    R::PrimaryFixed,
                    R::PrimaryFixedDim,
                    TonePolarity::Lighter,
                    true,
                )
            },
            ..ColorSpec::new(|_| Palette::Primary)
        },
        R::PrimaryFixedDim => ColorSpec {
            tone: Some(|s| if is_monochrome(s) { 30.0 } else { 80.0 }),
            is_background: true,
            background: |s| Some(highest_surface(s)),
            contrast_curve: |_| curve(1.0, 1.0, 3.0, 4.5),
            tone_delta_pair: |_| {
                pair(
                    R::PrimaryFixed,
                    R::PrimaryFixedDim,
                    TonePolarity::Lighter,
                    true,
                )
            },
            ..ColorSpec::new(|_| Palette::Primary)
        },
        R::OnPrimaryFixed => ColorSpec {
            tone: Some(|s| if is_monochrome(s) { 100.0 } else { 10.0 }),
            background: |_| Some(R::PrimaryFixedDim),
            second_background: |_| Some(R::PrimaryFixed),
            contrast_curve: |_| curve(4.5, 7.0, 11.0, 21.0),
            ..ColorSpec::new(|_| Palette::Primary)
        },
        R::OnPrimaryFixedVariant => ColorSpec {
            tone: Some(|s| if is_monochrome(s) { 90.0 } else { 30.0 }),
            background: |_| Some(R::PrimaryFixedDim),
            second_background: |_| Some(R::PrimaryFixed),
            contrast_curve: |_| curve(3.0, 4.5, 7.0, 11.0),
            ..ColorSpec::new(|_| Palette::Primary)
        },
        R::SecondaryFixed => ColorSpec {
            tone: Some(|s| if is_monochrome(s) { 80.0 } else { 90.0 }),
            is_background: true,
            background: |s| Some(highest_surface(s)),
            contrast_curve: |_| curve(1.0, 1.0, 3.0, 4.5),
            tone_delta_pair: |_| {
                pair(
                    R::SecondaryFixed,
                    R::SecondaryFixedDim,
                    TonePolarity::Lighter,
                    true,
                )
            },
            ..ColorSpec::new(|_| Palette::Secondary)
        },
        R::SecondaryFixedDim => ColorSpec {
            tone: Some(|s| if is_monochrome(s) { 70.0 } else { 80.0 }),
            is_background: true,
            background: |s| Some(highest_surface(s)),
            contrast_curve: |_| curve(1.0, 1.0, 3.0, 4.5),
            tone_delta_pair: |_| {
                pair(
                    R::SecondaryFixed,
                    R::SecondaryFixedDim,
                    TonePolarity::Lighter,
                    true,
                )
            },
            ..ColorSpec::new(|_| Palette::Secondary)
        },
        R::OnSecondaryFixed => ColorSpec {
            tone: Some(|_| 10.0),
            background: |_| Some(R::SecondaryFixedDim),
            second_background: |_| Some(R::SecondaryFixed),
            contrast_curve: |_| curve(4.5, 7.0, 11.0, 21.0),
            ..ColorSpec::new(|_| Palette::Secondary)
        },
        R::OnSecondaryFixedVariant => ColorSpec {
            tone: Some(|s| if is_monochrome(s) { 25.0 } else { 30.0 }),
            background: |_| Some(R::SecondaryFixedDim),
            second_background: |_| Some(R::SecondaryFixed),
            contrast_curve: |_| curve(3.0, 4.5, 7.0, 11.0),
            ..ColorSpec::new(|_| Palette::Secondary)
        },
        R::TertiaryFixed => ColorSpec {
            tone: Some(|s| if is_monochrome(s) { 40.0 } else { 90.0 }),
            is_background: true,
            background: |s| Some(highest_surface(s)),
            contrast_curve: |_| curve(1.0, 1.0, 3.0, 4.5),
            tone_delta_pair: |_| {
                pair(
                    R::TertiaryFixed,
                    R::TertiaryFixedDim,
                    TonePolarity::Lighter,
                    true,
                )
            },
            ..ColorSpec::new(|_| Palette::Tertiary)
        },
        R::TertiaryFixedDim => ColorSpec {
            tone: Some(|s| if is_monochrome(s) { 30.0 } else { 80.0 }),
            is_background: true,
            background: |s| Some(highest_surface(s)),
            contrast_curve: |_| curve(1.0, 1.0, 3.0, 4.5),
            tone_delta_pair: |_| {
                pair(
                    R::TertiaryFixed,
                    R::TertiaryFixedDim,
                    TonePolarity::Lighter,
                    true,
                )
            },
            ..ColorSpec::new(|_| Palette::Tertiary)
        },
        R::OnTertiaryFixed => ColorSpec {
            tone: Some(|s| if is_monochrome(s) { 100.0 } else { 10.0 }),
            background: |_| Some(R::TertiaryFixedDim),
            second_background: |_| Some(R::TertiaryFixed),
            contrast_curve: |_| curve(4.5, 7.0, 11.0, 21.0),
            ..ColorSpec::new(|_| Palette::Tertiary)
        },
        R::OnTertiaryFixedVariant => ColorSpec {
            tone: Some(|s| if is_monochrome(s) { 90.0 } else { 30.0 }),
            background: |_| Some(R::TertiaryFixedDim),
            second_background: |_| Some(R::TertiaryFixed),
            contrast_curve: |_| curve(3.0, 4.5, 7.0, 11.0),
            ..ColorSpec::new(|_| Palette::Tertiary)
        },
    }
}
