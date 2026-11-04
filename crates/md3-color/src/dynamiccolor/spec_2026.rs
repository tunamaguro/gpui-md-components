// Ported from material-color-utilities
// (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/dynamiccolor/color_spec_2026.ts (ColorSpecDelegateImpl2026)
//         @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2026 Google LLC. Licensed under the Apache License, Version 2.0.

//! 2026 仕様の色の役割の定義
//!
//! 2026 仕様の計算を使うのは [`Variant::Cmf`] の配色だけ。他の [`Variant`] で値を返す分岐は、
//! 移植元が「未定義の用途」として 0 を返すものをそのまま移している

use super::calc::get_tone;
use super::contrast_curve::ContrastCurve;
use super::scheme::Palette;
use super::spec::{ColorSpec, spec as spec_of};
use super::spec_2021::highest_surface;
use super::spec_2025::{get_curve, t_max_c, t_min_c};
use super::tone_delta_pair::{DeltaConstraint, ToneDeltaPair, TonePolarity};
use super::{ColorRole as R, DynamicScheme, Platform, SpecVersion, Variant};
use crate::utils::math;

fn cmf(s: &DynamicScheme) -> bool {
    s.variant == Variant::Cmf
}

/// [`Variant::Cmf`] なら `value`、それ以外は 0 (移植元の未定義の用途)
fn cmf_or_zero(s: &DynamicScheme, value: f64) -> f64 {
    if cmf(s) { value } else { 0.0 }
}

fn dark_light(s: &DynamicScheme, dark: f64, light: f64) -> f64 {
    if s.is_dark { dark } else { light }
}

fn highest(s: &DynamicScheme) -> Option<R> {
    Some(highest_surface(s))
}

/// コントラストを上げたときだけ、背景との比 1.5 を求める
fn container_curve(s: &DynamicScheme) -> Option<ContrastCurve> {
    if s.contrast_level > 0.0 {
        get_curve(1.5)
    } else {
        None
    }
}

/// 2 つ目の元の色。なければ最初の元の色
fn secondary_source_tone(s: &DynamicScheme) -> f64 {
    s.source_color_hcts
        .get(1)
        .unwrap_or(&s.source_color_hct)
        .tone_raw()
}

/// スマートフォンでは、コンテナとのトーンの差を 5 以上に広げる
fn phone_pair(s: &DynamicScheme, container: R, role: R) -> Option<ToneDeltaPair> {
    if s.platform == Platform::Phone {
        Some(
            ToneDeltaPair::new(container, role, 5.0, TonePolarity::RelativeLighter, true)
                .with_constraint(DeltaConstraint::Farther),
        )
    } else {
        None
    }
}

fn fixed_dim_pair(fixed_dim: R, fixed: R) -> Option<ToneDeltaPair> {
    Some(
        ToneDeltaPair::new(fixed_dim, fixed, 5.0, TonePolarity::Darker, true)
            .with_constraint(DeltaConstraint::Exact),
    )
}

/// 共通の色のトーンが 57 より大きければ暗い方、そうでなければ明るい方を背景にする
fn fixed_background(s: &DynamicScheme, fixed: R, fixed_dim: R) -> Option<R> {
    Some(if get_tone(s, fixed) > 57.0 {
        fixed_dim
    } else {
        fixed
    })
}

/// 2026 仕様での `role` の定義。2026 仕様で定義が変わらない役割は `None`
///
/// 移植元が別の役割の複製で定義する役割 (dim の役割) は、`version` での複製元の定義を返す
pub(crate) fn spec(role: R, version: SpecVersion) -> Option<ColorSpec> {
    let spec = match role {
        R::Surface => ColorSpec {
            tone: Some(|s| {
                if cmf(s) {
                    dark_light(s, 4.0, 98.0)
                } else {
                    0.0
                }
            }),
            is_background: true,
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::SurfaceDim => ColorSpec {
            tone: Some(|s| {
                if cmf(s) {
                    dark_light(s, 4.0, 87.0)
                } else {
                    0.0
                }
            }),
            chroma_multiplier: |s| if cmf(s) { dark_light(s, 1.0, 1.7) } else { 0.0 },
            is_background: true,
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::SurfaceBright => ColorSpec {
            tone: Some(|s| {
                if cmf(s) {
                    dark_light(s, 18.0, 98.0)
                } else {
                    0.0
                }
            }),
            chroma_multiplier: |s| if cmf(s) { dark_light(s, 1.7, 1.0) } else { 0.0 },
            is_background: true,
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::SurfaceContainerLowest => ColorSpec {
            tone: Some(|s| {
                if cmf(s) {
                    dark_light(s, 0.0, 100.0)
                } else {
                    0.0
                }
            }),
            is_background: true,
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::SurfaceContainerLow => ColorSpec {
            tone: Some(|s| {
                if cmf(s) {
                    dark_light(s, 6.0, 96.0)
                } else {
                    0.0
                }
            }),
            chroma_multiplier: |s| cmf_or_zero(s, 1.25),
            is_background: true,
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::SurfaceContainer => ColorSpec {
            tone: Some(|s| {
                if cmf(s) {
                    dark_light(s, 9.0, 94.0)
                } else {
                    0.0
                }
            }),
            chroma_multiplier: |s| cmf_or_zero(s, 1.4),
            is_background: true,
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::SurfaceContainerHigh => ColorSpec {
            tone: Some(|s| {
                if cmf(s) {
                    dark_light(s, 12.0, 92.0)
                } else {
                    0.0
                }
            }),
            chroma_multiplier: |s| cmf_or_zero(s, 1.5),
            is_background: true,
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::SurfaceContainerHighest => ColorSpec {
            tone: Some(|s| {
                if cmf(s) {
                    dark_light(s, 15.0, 90.0)
                } else {
                    0.0
                }
            }),
            chroma_multiplier: |s| cmf_or_zero(s, 1.7),
            is_background: true,
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::OnSurface => ColorSpec {
            chroma_multiplier: |s| cmf_or_zero(s, 1.7),
            background: highest,
            contrast_curve: |s| {
                if s.is_dark {
                    get_curve(11.0)
                } else {
                    get_curve(9.0)
                }
            },
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::OnSurfaceVariant => ColorSpec {
            chroma_multiplier: |s| cmf_or_zero(s, 1.7),
            background: highest,
            contrast_curve: |s| {
                if s.is_dark {
                    get_curve(6.0)
                } else {
                    get_curve(4.5)
                }
            },
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::Outline => ColorSpec {
            chroma_multiplier: |s| cmf_or_zero(s, 1.7),
            background: highest,
            contrast_curve: |_| get_curve(3.0),
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::OutlineVariant => ColorSpec {
            chroma_multiplier: |s| cmf_or_zero(s, 1.7),
            background: highest,
            contrast_curve: |_| get_curve(1.5),
            ..ColorSpec::new(|_| Palette::Neutral)
        },
        R::InverseSurface => ColorSpec {
            tone: Some(|s| dark_light(s, 98.0, 4.0)),
            chroma_multiplier: |s| cmf_or_zero(s, 1.7),
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
                if s.source_color_hct.chroma_raw() <= 12.0 {
                    dark_light(s, 80.0, 40.0)
                } else {
                    s.source_color_hct.tone_raw()
                }
            }),
            is_background: true,
            background: highest,
            contrast_curve: |_| get_curve(4.5),
            tone_delta_pair: |s| phone_pair(s, R::PrimaryContainer, R::Primary),
            ..ColorSpec::new(|_| Palette::Primary)
        },
        R::OnPrimary => ColorSpec {
            background: |_| Some(R::Primary),
            contrast_curve: |_| get_curve(6.0),
            ..ColorSpec::new(|_| Palette::Primary)
        },
        R::PrimaryContainer => ColorSpec {
            tone: Some(|s| {
                let source = &s.source_color_hct;
                if !s.is_dark && source.chroma_raw() <= 12.0 {
                    return 90.0;
                }
                if source.tone_raw() > 55.0 {
                    math::clamp_double(61.0, 90.0, source.tone_raw())
                } else {
                    math::clamp_double(30.0, 49.0, source.tone_raw())
                }
            }),
            is_background: true,
            background: highest,
            contrast_curve: container_curve,
            ..ColorSpec::new(|_| Palette::Primary)
        },
        R::OnPrimaryContainer => ColorSpec {
            background: |_| Some(R::PrimaryContainer),
            contrast_curve: |_| get_curve(6.0),
            ..ColorSpec::new(|_| Palette::Primary)
        },
        R::PrimaryFixed => ColorSpec {
            tone: Some(|s| get_tone(&s.light_standard_contrast(), R::PrimaryContainer)),
            is_background: true,
            background: highest,
            contrast_curve: container_curve,
            ..ColorSpec::new(|_| Palette::Primary)
        },
        R::PrimaryFixedDim => ColorSpec {
            tone: Some(|s| get_tone(s, R::PrimaryFixed)),
            is_background: true,
            background: highest,
            tone_delta_pair: |_| fixed_dim_pair(R::PrimaryFixedDim, R::PrimaryFixed),
            contrast_curve: container_curve,
            ..ColorSpec::new(|_| Palette::Primary)
        },
        R::OnPrimaryFixed => ColorSpec {
            background: |s| fixed_background(s, R::PrimaryFixed, R::PrimaryFixedDim),
            contrast_curve: |_| get_curve(7.0),
            ..ColorSpec::new(|_| Palette::Primary)
        },
        R::OnPrimaryFixedVariant => ColorSpec {
            background: |s| fixed_background(s, R::PrimaryFixed, R::PrimaryFixedDim),
            contrast_curve: |_| get_curve(4.5),
            ..ColorSpec::new(|_| Palette::Primary)
        },
        R::Secondary => ColorSpec {
            tone: Some(|s| {
                if s.is_dark {
                    t_min_c(&s.secondary_palette, 0.0, 100.0)
                } else {
                    t_max_c(&s.secondary_palette, 0.0, 100.0, 1.0)
                }
            }),
            is_background: true,
            background: highest,
            contrast_curve: |_| get_curve(4.5),
            tone_delta_pair: |s| phone_pair(s, R::SecondaryContainer, R::Secondary),
            ..ColorSpec::new(|_| Palette::Secondary)
        },
        R::OnSecondary => ColorSpec {
            background: |_| Some(R::Secondary),
            contrast_curve: |_| get_curve(6.0),
            ..ColorSpec::new(|_| Palette::Secondary)
        },
        R::SecondaryContainer => ColorSpec {
            tone: Some(|s| {
                if s.is_dark {
                    t_min_c(&s.secondary_palette, 20.0, 49.0)
                } else {
                    t_max_c(&s.secondary_palette, 61.0, 90.0, 1.0)
                }
            }),
            is_background: true,
            background: highest,
            contrast_curve: container_curve,
            ..ColorSpec::new(|_| Palette::Secondary)
        },
        R::OnSecondaryContainer => ColorSpec {
            background: |_| Some(R::SecondaryContainer),
            contrast_curve: |_| get_curve(6.0),
            ..ColorSpec::new(|_| Palette::Secondary)
        },
        R::SecondaryFixed => ColorSpec {
            tone: Some(|s| get_tone(&s.light_standard_contrast(), R::SecondaryContainer)),
            is_background: true,
            background: highest,
            contrast_curve: container_curve,
            ..ColorSpec::new(|_| Palette::Secondary)
        },
        R::SecondaryFixedDim => ColorSpec {
            tone: Some(|s| get_tone(s, R::SecondaryFixed)),
            is_background: true,
            background: highest,
            tone_delta_pair: |_| fixed_dim_pair(R::SecondaryFixedDim, R::SecondaryFixed),
            contrast_curve: container_curve,
            ..ColorSpec::new(|_| Palette::Secondary)
        },
        R::OnSecondaryFixed => ColorSpec {
            background: |s| fixed_background(s, R::SecondaryFixed, R::SecondaryFixedDim),
            contrast_curve: |_| get_curve(7.0),
            ..ColorSpec::new(|_| Palette::Secondary)
        },
        R::OnSecondaryFixedVariant => ColorSpec {
            background: |s| fixed_background(s, R::SecondaryFixed, R::SecondaryFixedDim),
            contrast_curve: |_| get_curve(4.5),
            ..ColorSpec::new(|_| Palette::Secondary)
        },
        R::Tertiary => ColorSpec {
            tone: Some(secondary_source_tone),
            is_background: true,
            background: highest,
            contrast_curve: |_| get_curve(4.5),
            tone_delta_pair: |s| phone_pair(s, R::TertiaryContainer, R::Tertiary),
            ..ColorSpec::new(|_| Palette::Tertiary)
        },
        R::OnTertiary => ColorSpec {
            background: |_| Some(R::Tertiary),
            contrast_curve: |_| get_curve(6.0),
            ..ColorSpec::new(|_| Palette::Tertiary)
        },
        R::TertiaryContainer => ColorSpec {
            tone: Some(|s| {
                let tone = secondary_source_tone(s);
                if tone > 55.0 {
                    math::clamp_double(61.0, 90.0, tone)
                } else {
                    math::clamp_double(20.0, 49.0, tone)
                }
            }),
            is_background: true,
            background: highest,
            contrast_curve: container_curve,
            ..ColorSpec::new(|_| Palette::Tertiary)
        },
        R::OnTertiaryContainer => ColorSpec {
            background: |_| Some(R::TertiaryContainer),
            contrast_curve: |_| get_curve(6.0),
            ..ColorSpec::new(|_| Palette::Tertiary)
        },
        R::TertiaryFixed => ColorSpec {
            tone: Some(|s| get_tone(&s.light_standard_contrast(), R::TertiaryContainer)),
            is_background: true,
            background: highest,
            contrast_curve: container_curve,
            ..ColorSpec::new(|_| Palette::Tertiary)
        },
        R::TertiaryFixedDim => ColorSpec {
            tone: Some(|s| get_tone(s, R::TertiaryFixed)),
            is_background: true,
            background: highest,
            tone_delta_pair: |_| fixed_dim_pair(R::TertiaryFixedDim, R::TertiaryFixed),
            contrast_curve: container_curve,
            ..ColorSpec::new(|_| Palette::Tertiary)
        },
        R::OnTertiaryFixed => ColorSpec {
            background: |s| fixed_background(s, R::TertiaryFixed, R::TertiaryFixedDim),
            contrast_curve: |_| get_curve(7.0),
            ..ColorSpec::new(|_| Palette::Tertiary)
        },
        R::OnTertiaryFixedVariant => ColorSpec {
            background: |s| fixed_background(s, R::TertiaryFixed, R::TertiaryFixedDim),
            contrast_curve: |_| get_curve(4.5),
            ..ColorSpec::new(|_| Palette::Tertiary)
        },
        R::Error => ColorSpec {
            tone: Some(|s| t_max_c(&s.error_palette, 0.0, 100.0, 1.0)),
            is_background: true,
            background: highest,
            contrast_curve: |_| get_curve(4.5),
            tone_delta_pair: |s| phone_pair(s, R::ErrorContainer, R::Error),
            ..ColorSpec::new(|_| Palette::Error)
        },
        R::OnError => ColorSpec {
            background: |_| Some(R::Error),
            contrast_curve: |_| get_curve(6.0),
            ..ColorSpec::new(|_| Palette::Error)
        },
        R::ErrorContainer => ColorSpec {
            tone: Some(|s| {
                if s.is_dark {
                    t_min_c(&s.error_palette, 0.0, 100.0)
                } else {
                    t_max_c(&s.error_palette, 0.0, 100.0, 1.0)
                }
            }),
            is_background: true,
            background: highest,
            contrast_curve: container_curve,
            ..ColorSpec::new(|_| Palette::Error)
        },
        R::OnErrorContainer => ColorSpec {
            background: |_| Some(R::ErrorContainer),
            contrast_curve: |_| get_curve(6.0),
            ..ColorSpec::new(|_| Palette::Error)
        },
        // 移植元は各色の複製に名前を付け直して使う
        R::PrimaryDim => spec_of(R::Primary, version),
        R::SecondaryDim => spec_of(R::Secondary, version),
        R::TertiaryDim => spec_of(R::Tertiary, version),
        R::ErrorDim => spec_of(R::Error, version),
        // 移植元の inversePrimary は super の定義をそのまま返す
        R::InversePrimary
        | R::PrimaryPaletteKeyColor
        | R::SecondaryPaletteKeyColor
        | R::TertiaryPaletteKeyColor
        | R::NeutralPaletteKeyColor
        | R::NeutralVariantPaletteKeyColor
        | R::ErrorPaletteKeyColor
        | R::Background
        | R::OnBackground
        | R::SurfaceVariant
        | R::Shadow
        | R::Scrim
        | R::SurfaceTint => return None,
    };
    Some(spec)
}
