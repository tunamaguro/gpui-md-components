// Ported from material-color-utilities (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/dynamiccolor/color_spec_2021.ts, tone_delta_pair.ts
//         @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2021 Google LLC. Licensed under the Apache License, Version 2.0.

//! 各カラーロールの定義（2021 仕様）。
//!
//! 移植元の `DynamicColor` はクロージャの集まりだが、ここでは役割ごとの
//! 性質を `match` で網羅して表す。

use super::{ContrastCurve, DynamicScheme, Variant};
use crate::dislike;
use crate::hct::Hct;
use crate::palettes::TonalPalette;

/// 配色の役割（MD3 のカラーロール）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColorRole {
    /// `primary_palette_key_color`
    PrimaryPaletteKeyColor,
    /// `secondary_palette_key_color`
    SecondaryPaletteKeyColor,
    /// `tertiary_palette_key_color`
    TertiaryPaletteKeyColor,
    /// `neutral_palette_key_color`
    NeutralPaletteKeyColor,
    /// `neutral_variant_palette_key_color`
    NeutralVariantPaletteKeyColor,
    /// `error_palette_key_color`
    ErrorPaletteKeyColor,
    /// `background`
    Background,
    /// `on_background`
    OnBackground,
    /// `surface`
    Surface,
    /// `surface_dim`
    SurfaceDim,
    /// `surface_bright`
    SurfaceBright,
    /// `surface_container_lowest`
    SurfaceContainerLowest,
    /// `surface_container_low`
    SurfaceContainerLow,
    /// `surface_container`
    SurfaceContainer,
    /// `surface_container_high`
    SurfaceContainerHigh,
    /// `surface_container_highest`
    SurfaceContainerHighest,
    /// `on_surface`
    OnSurface,
    /// `surface_variant`
    SurfaceVariant,
    /// `on_surface_variant`
    OnSurfaceVariant,
    /// `inverse_surface`
    InverseSurface,
    /// `inverse_on_surface`
    InverseOnSurface,
    /// `outline`
    Outline,
    /// `outline_variant`
    OutlineVariant,
    /// `shadow`
    Shadow,
    /// `scrim`
    Scrim,
    /// `surface_tint`
    SurfaceTint,
    /// `primary`
    Primary,
    /// `on_primary`
    OnPrimary,
    /// `primary_container`
    PrimaryContainer,
    /// `on_primary_container`
    OnPrimaryContainer,
    /// `inverse_primary`
    InversePrimary,
    /// `secondary`
    Secondary,
    /// `on_secondary`
    OnSecondary,
    /// `secondary_container`
    SecondaryContainer,
    /// `on_secondary_container`
    OnSecondaryContainer,
    /// `tertiary`
    Tertiary,
    /// `on_tertiary`
    OnTertiary,
    /// `tertiary_container`
    TertiaryContainer,
    /// `on_tertiary_container`
    OnTertiaryContainer,
    /// `error`
    Error,
    /// `on_error`
    OnError,
    /// `error_container`
    ErrorContainer,
    /// `on_error_container`
    OnErrorContainer,
    /// `primary_fixed`
    PrimaryFixed,
    /// `primary_fixed_dim`
    PrimaryFixedDim,
    /// `on_primary_fixed`
    OnPrimaryFixed,
    /// `on_primary_fixed_variant`
    OnPrimaryFixedVariant,
    /// `secondary_fixed`
    SecondaryFixed,
    /// `secondary_fixed_dim`
    SecondaryFixedDim,
    /// `on_secondary_fixed`
    OnSecondaryFixed,
    /// `on_secondary_fixed_variant`
    OnSecondaryFixedVariant,
    /// `tertiary_fixed`
    TertiaryFixed,
    /// `tertiary_fixed_dim`
    TertiaryFixedDim,
    /// `on_tertiary_fixed`
    OnTertiaryFixed,
    /// `on_tertiary_fixed_variant`
    OnTertiaryFixedVariant,
}

impl ColorRole {
    /// すべての役割。
    pub const ALL: [ColorRole; 55] = [
        ColorRole::PrimaryPaletteKeyColor,
        ColorRole::SecondaryPaletteKeyColor,
        ColorRole::TertiaryPaletteKeyColor,
        ColorRole::NeutralPaletteKeyColor,
        ColorRole::NeutralVariantPaletteKeyColor,
        ColorRole::ErrorPaletteKeyColor,
        ColorRole::Background,
        ColorRole::OnBackground,
        ColorRole::Surface,
        ColorRole::SurfaceDim,
        ColorRole::SurfaceBright,
        ColorRole::SurfaceContainerLowest,
        ColorRole::SurfaceContainerLow,
        ColorRole::SurfaceContainer,
        ColorRole::SurfaceContainerHigh,
        ColorRole::SurfaceContainerHighest,
        ColorRole::OnSurface,
        ColorRole::SurfaceVariant,
        ColorRole::OnSurfaceVariant,
        ColorRole::InverseSurface,
        ColorRole::InverseOnSurface,
        ColorRole::Outline,
        ColorRole::OutlineVariant,
        ColorRole::Shadow,
        ColorRole::Scrim,
        ColorRole::SurfaceTint,
        ColorRole::Primary,
        ColorRole::OnPrimary,
        ColorRole::PrimaryContainer,
        ColorRole::OnPrimaryContainer,
        ColorRole::InversePrimary,
        ColorRole::Secondary,
        ColorRole::OnSecondary,
        ColorRole::SecondaryContainer,
        ColorRole::OnSecondaryContainer,
        ColorRole::Tertiary,
        ColorRole::OnTertiary,
        ColorRole::TertiaryContainer,
        ColorRole::OnTertiaryContainer,
        ColorRole::Error,
        ColorRole::OnError,
        ColorRole::ErrorContainer,
        ColorRole::OnErrorContainer,
        ColorRole::PrimaryFixed,
        ColorRole::PrimaryFixedDim,
        ColorRole::OnPrimaryFixed,
        ColorRole::OnPrimaryFixedVariant,
        ColorRole::SecondaryFixed,
        ColorRole::SecondaryFixedDim,
        ColorRole::OnSecondaryFixed,
        ColorRole::OnSecondaryFixedVariant,
        ColorRole::TertiaryFixed,
        ColorRole::TertiaryFixedDim,
        ColorRole::OnTertiaryFixed,
        ColorRole::OnTertiaryFixedVariant,
    ];

    /// 移植元と同じ snake_case の名前（例: `on_primary_container`）。
    pub const fn name(self) -> &'static str {
        match self {
            ColorRole::PrimaryPaletteKeyColor => "primary_palette_key_color",
            ColorRole::SecondaryPaletteKeyColor => "secondary_palette_key_color",
            ColorRole::TertiaryPaletteKeyColor => "tertiary_palette_key_color",
            ColorRole::NeutralPaletteKeyColor => "neutral_palette_key_color",
            ColorRole::NeutralVariantPaletteKeyColor => "neutral_variant_palette_key_color",
            ColorRole::ErrorPaletteKeyColor => "error_palette_key_color",
            ColorRole::Background => "background",
            ColorRole::OnBackground => "on_background",
            ColorRole::Surface => "surface",
            ColorRole::SurfaceDim => "surface_dim",
            ColorRole::SurfaceBright => "surface_bright",
            ColorRole::SurfaceContainerLowest => "surface_container_lowest",
            ColorRole::SurfaceContainerLow => "surface_container_low",
            ColorRole::SurfaceContainer => "surface_container",
            ColorRole::SurfaceContainerHigh => "surface_container_high",
            ColorRole::SurfaceContainerHighest => "surface_container_highest",
            ColorRole::OnSurface => "on_surface",
            ColorRole::SurfaceVariant => "surface_variant",
            ColorRole::OnSurfaceVariant => "on_surface_variant",
            ColorRole::InverseSurface => "inverse_surface",
            ColorRole::InverseOnSurface => "inverse_on_surface",
            ColorRole::Outline => "outline",
            ColorRole::OutlineVariant => "outline_variant",
            ColorRole::Shadow => "shadow",
            ColorRole::Scrim => "scrim",
            ColorRole::SurfaceTint => "surface_tint",
            ColorRole::Primary => "primary",
            ColorRole::OnPrimary => "on_primary",
            ColorRole::PrimaryContainer => "primary_container",
            ColorRole::OnPrimaryContainer => "on_primary_container",
            ColorRole::InversePrimary => "inverse_primary",
            ColorRole::Secondary => "secondary",
            ColorRole::OnSecondary => "on_secondary",
            ColorRole::SecondaryContainer => "secondary_container",
            ColorRole::OnSecondaryContainer => "on_secondary_container",
            ColorRole::Tertiary => "tertiary",
            ColorRole::OnTertiary => "on_tertiary",
            ColorRole::TertiaryContainer => "tertiary_container",
            ColorRole::OnTertiaryContainer => "on_tertiary_container",
            ColorRole::Error => "error",
            ColorRole::OnError => "on_error",
            ColorRole::ErrorContainer => "error_container",
            ColorRole::OnErrorContainer => "on_error_container",
            ColorRole::PrimaryFixed => "primary_fixed",
            ColorRole::PrimaryFixedDim => "primary_fixed_dim",
            ColorRole::OnPrimaryFixed => "on_primary_fixed",
            ColorRole::OnPrimaryFixedVariant => "on_primary_fixed_variant",
            ColorRole::SecondaryFixed => "secondary_fixed",
            ColorRole::SecondaryFixedDim => "secondary_fixed_dim",
            ColorRole::OnSecondaryFixed => "on_secondary_fixed",
            ColorRole::OnSecondaryFixedVariant => "on_secondary_fixed_variant",
            ColorRole::TertiaryFixed => "tertiary_fixed",
            ColorRole::TertiaryFixedDim => "tertiary_fixed_dim",
            ColorRole::OnTertiaryFixed => "on_tertiary_fixed",
            ColorRole::OnTertiaryFixedVariant => "on_tertiary_fixed_variant",
        }
    }
}

/// トーン差の向き。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TonePolarity {
    // 2021 仕様の役割では使わないが、移植元の定義に合わせて持っておく。
    #[allow(dead_code)]
    Darker,
    Lighter,
    Nearer,
    // 2021 仕様の役割では使わないが、移植元の定義に合わせて持っておく。
    #[allow(dead_code)]
    Farther,
}

/// 2 つの役割のトーン差の制約。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ToneDeltaPair {
    pub(crate) role_a: ColorRole,
    pub(crate) role_b: ColorRole,
    pub(crate) delta: f64,
    pub(crate) polarity: TonePolarity,
    pub(crate) stay_together: bool,
}

const fn pair(
    role_a: ColorRole,
    role_b: ColorRole,
    polarity: TonePolarity,
    stay_together: bool,
) -> ToneDeltaPair {
    ToneDeltaPair {
        role_a,
        role_b,
        delta: 10.0,
        polarity,
        stay_together,
    }
}

fn is_fidelity(s: &DynamicScheme) -> bool {
    matches!(s.variant, Variant::Fidelity | Variant::Content)
}

fn is_monochrome(s: &DynamicScheme) -> bool {
    s.variant == Variant::Monochrome
}

/// `dark` / `light` のどちらかを選ぶ。
fn dl(s: &DynamicScheme, dark: f64, light: f64) -> f64 {
    if s.is_dark { dark } else { light }
}

fn curve(s: &DynamicScheme, low: f64, normal: f64, medium: f64, high: f64) -> f64 {
    ContrastCurve::new(low, normal, medium, high).get(s.contrast_level)
}

fn find_desired_chroma_by_tone(hue: f64, chroma: f64, tone: f64, by_decreasing_tone: bool) -> f64 {
    let mut answer = tone;
    let mut closest_to_chroma = Hct::from_hct(hue, chroma, tone);
    if closest_to_chroma.chroma().value() < chroma {
        let mut chroma_peak = closest_to_chroma.chroma().value();
        while closest_to_chroma.chroma().value() < chroma {
            answer += if by_decreasing_tone { -1.0 } else { 1.0 };
            let potential_solution = Hct::from_hct(hue, chroma, answer);
            let potential_chroma = potential_solution.chroma().value();
            if chroma_peak > potential_chroma {
                break;
            }
            if (potential_chroma - chroma).abs() < 0.4 {
                break;
            }
            let potential_delta = (potential_chroma - chroma).abs();
            let current_delta = (closest_to_chroma.chroma().value() - chroma).abs();
            if potential_delta < current_delta {
                closest_to_chroma = potential_solution;
            }
            chroma_peak = f64::max(chroma_peak, potential_chroma);
        }
    }
    answer
}

impl ColorRole {
    /// この役割の色を取り出すパレット。
    pub(crate) fn palette(self, s: &DynamicScheme) -> &TonalPalette {
        use ColorRole::*;
        match self {
            PrimaryPaletteKeyColor
            | SurfaceTint
            | Primary
            | OnPrimary
            | PrimaryContainer
            | OnPrimaryContainer
            | InversePrimary
            | PrimaryFixed
            | PrimaryFixedDim
            | OnPrimaryFixed
            | OnPrimaryFixedVariant => &s.primary_palette,
            SecondaryPaletteKeyColor
            | Secondary
            | OnSecondary
            | SecondaryContainer
            | OnSecondaryContainer
            | SecondaryFixed
            | SecondaryFixedDim
            | OnSecondaryFixed
            | OnSecondaryFixedVariant => &s.secondary_palette,
            TertiaryPaletteKeyColor
            | Tertiary
            | OnTertiary
            | TertiaryContainer
            | OnTertiaryContainer
            | TertiaryFixed
            | TertiaryFixedDim
            | OnTertiaryFixed
            | OnTertiaryFixedVariant => &s.tertiary_palette,
            NeutralPaletteKeyColor
            | Background
            | OnBackground
            | Surface
            | SurfaceDim
            | SurfaceBright
            | SurfaceContainerLowest
            | SurfaceContainerLow
            | SurfaceContainer
            | SurfaceContainerHigh
            | SurfaceContainerHighest
            | OnSurface
            | InverseSurface
            | InverseOnSurface
            | Shadow
            | Scrim => &s.neutral_palette,
            NeutralVariantPaletteKeyColor
            | SurfaceVariant
            | OnSurfaceVariant
            | Outline
            | OutlineVariant => &s.neutral_variant_palette,
            ErrorPaletteKeyColor | Error | OnError | ErrorContainer | OnErrorContainer => {
                &s.error_palette
            }
        }
    }

    /// 制約を適用する前のトーン（移植元の `DynamicColor.tone`）。
    pub(crate) fn tone(self, s: &DynamicScheme) -> f64 {
        use ColorRole::*;
        let mono = is_monochrome(s);
        let fidelity = is_fidelity(s);
        match self {
            PrimaryPaletteKeyColor => s.primary_palette.key_color().tone().value(),
            SecondaryPaletteKeyColor => s.secondary_palette.key_color().tone().value(),
            TertiaryPaletteKeyColor => s.tertiary_palette.key_color().tone().value(),
            NeutralPaletteKeyColor => s.neutral_palette.key_color().tone().value(),
            NeutralVariantPaletteKeyColor => s.neutral_variant_palette.key_color().tone().value(),
            ErrorPaletteKeyColor => s.error_palette.key_color().tone().value(),
            Background | Surface => dl(s, 6.0, 98.0),
            OnBackground | OnSurface => dl(s, 90.0, 10.0),
            SurfaceDim => {
                if s.is_dark {
                    6.0
                } else {
                    curve(s, 87.0, 87.0, 80.0, 75.0)
                }
            }
            SurfaceBright => {
                if s.is_dark {
                    curve(s, 24.0, 24.0, 29.0, 34.0)
                } else {
                    98.0
                }
            }
            SurfaceContainerLowest => {
                if s.is_dark {
                    curve(s, 4.0, 4.0, 2.0, 0.0)
                } else {
                    100.0
                }
            }
            SurfaceContainerLow => {
                if s.is_dark {
                    curve(s, 10.0, 10.0, 11.0, 12.0)
                } else {
                    curve(s, 96.0, 96.0, 96.0, 95.0)
                }
            }
            SurfaceContainer => {
                if s.is_dark {
                    curve(s, 12.0, 12.0, 16.0, 20.0)
                } else {
                    curve(s, 94.0, 94.0, 92.0, 90.0)
                }
            }
            SurfaceContainerHigh => {
                if s.is_dark {
                    curve(s, 17.0, 17.0, 21.0, 25.0)
                } else {
                    curve(s, 92.0, 92.0, 88.0, 85.0)
                }
            }
            SurfaceContainerHighest => {
                if s.is_dark {
                    curve(s, 22.0, 22.0, 26.0, 30.0)
                } else {
                    curve(s, 90.0, 90.0, 84.0, 80.0)
                }
            }
            SurfaceVariant => dl(s, 30.0, 90.0),
            OnSurfaceVariant => dl(s, 80.0, 30.0),
            InverseSurface => dl(s, 90.0, 20.0),
            InverseOnSurface => dl(s, 20.0, 95.0),
            Outline => dl(s, 60.0, 50.0),
            OutlineVariant => dl(s, 30.0, 80.0),
            Shadow | Scrim => 0.0,
            SurfaceTint => dl(s, 80.0, 40.0),

            Primary => {
                if mono {
                    dl(s, 100.0, 0.0)
                } else {
                    dl(s, 80.0, 40.0)
                }
            }
            OnPrimary => {
                if mono {
                    dl(s, 10.0, 90.0)
                } else {
                    dl(s, 20.0, 100.0)
                }
            }
            PrimaryContainer => {
                if fidelity {
                    s.source_color_hct.tone().value()
                } else if mono {
                    dl(s, 85.0, 25.0)
                } else {
                    dl(s, 30.0, 90.0)
                }
            }
            OnPrimaryContainer => {
                if fidelity {
                    super::tone::foreground_tone(PrimaryContainer.tone(s), 4.5)
                } else if mono {
                    dl(s, 0.0, 100.0)
                } else {
                    dl(s, 90.0, 30.0)
                }
            }
            InversePrimary => dl(s, 40.0, 80.0),

            Secondary => dl(s, 80.0, 40.0),
            OnSecondary => {
                if mono {
                    dl(s, 10.0, 100.0)
                } else {
                    dl(s, 20.0, 100.0)
                }
            }
            SecondaryContainer => {
                let initial_tone = dl(s, 30.0, 90.0);
                if mono {
                    dl(s, 30.0, 85.0)
                } else if !fidelity {
                    initial_tone
                } else {
                    find_desired_chroma_by_tone(
                        s.secondary_palette.hue_raw(),
                        s.secondary_palette.chroma_raw(),
                        initial_tone,
                        !s.is_dark,
                    )
                }
            }
            OnSecondaryContainer => {
                if mono {
                    dl(s, 90.0, 10.0)
                } else if !fidelity {
                    dl(s, 90.0, 30.0)
                } else {
                    super::tone::foreground_tone(SecondaryContainer.tone(s), 4.5)
                }
            }

            Tertiary => {
                if mono {
                    dl(s, 90.0, 25.0)
                } else {
                    dl(s, 80.0, 40.0)
                }
            }
            OnTertiary => {
                if mono {
                    dl(s, 10.0, 90.0)
                } else {
                    dl(s, 20.0, 100.0)
                }
            }
            TertiaryContainer => {
                if mono {
                    dl(s, 60.0, 49.0)
                } else if !fidelity {
                    dl(s, 30.0, 90.0)
                } else {
                    let proposed_hct = s
                        .tertiary_palette
                        .hct_raw(s.source_color_hct.tone().value());
                    dislike::fix_if_disliked(proposed_hct).tone().value()
                }
            }
            OnTertiaryContainer => {
                if mono {
                    dl(s, 0.0, 100.0)
                } else if !fidelity {
                    dl(s, 90.0, 30.0)
                } else {
                    super::tone::foreground_tone(TertiaryContainer.tone(s), 4.5)
                }
            }

            Error => dl(s, 80.0, 40.0),
            OnError => dl(s, 20.0, 100.0),
            ErrorContainer => dl(s, 30.0, 90.0),
            OnErrorContainer => {
                if mono {
                    dl(s, 90.0, 10.0)
                } else {
                    dl(s, 90.0, 30.0)
                }
            }

            PrimaryFixed | TertiaryFixed => {
                if mono {
                    40.0
                } else {
                    90.0
                }
            }
            PrimaryFixedDim | TertiaryFixedDim => {
                if mono {
                    30.0
                } else {
                    80.0
                }
            }
            OnPrimaryFixed | OnTertiaryFixed => {
                if mono {
                    100.0
                } else {
                    10.0
                }
            }
            OnPrimaryFixedVariant | OnTertiaryFixedVariant => {
                if mono {
                    90.0
                } else {
                    30.0
                }
            }
            SecondaryFixed => {
                if mono {
                    80.0
                } else {
                    90.0
                }
            }
            SecondaryFixedDim => {
                if mono {
                    70.0
                } else {
                    80.0
                }
            }
            OnSecondaryFixed => 10.0,
            OnSecondaryFixedVariant => {
                if mono {
                    25.0
                } else {
                    30.0
                }
            }
        }
    }

    /// 背景として使う役割か。
    pub fn is_background(self) -> bool {
        use ColorRole::*;
        matches!(
            self,
            Background
                | Surface
                | SurfaceDim
                | SurfaceBright
                | SurfaceContainerLowest
                | SurfaceContainerLow
                | SurfaceContainer
                | SurfaceContainerHigh
                | SurfaceContainerHighest
                | SurfaceVariant
                | InverseSurface
                | SurfaceTint
                | Primary
                | PrimaryContainer
                | Secondary
                | SecondaryContainer
                | Tertiary
                | TertiaryContainer
                | Error
                | ErrorContainer
                | PrimaryFixed
                | PrimaryFixedDim
                | SecondaryFixed
                | SecondaryFixedDim
                | TertiaryFixed
                | TertiaryFixedDim
        )
    }

    /// コントラストを確保する相手の背景。
    pub(crate) fn background(self, s: &DynamicScheme) -> Option<ColorRole> {
        use ColorRole::*;
        let highest_surface = if s.is_dark { SurfaceBright } else { SurfaceDim };
        match self {
            PrimaryPaletteKeyColor
            | SecondaryPaletteKeyColor
            | TertiaryPaletteKeyColor
            | NeutralPaletteKeyColor
            | NeutralVariantPaletteKeyColor
            | ErrorPaletteKeyColor
            | Background
            | Surface
            | SurfaceDim
            | SurfaceBright
            | SurfaceContainerLowest
            | SurfaceContainerLow
            | SurfaceContainer
            | SurfaceContainerHigh
            | SurfaceContainerHighest
            | SurfaceVariant
            | InverseSurface
            | Shadow
            | Scrim
            | SurfaceTint => None,
            OnBackground => Some(Background),
            OnSurface | OnSurfaceVariant | Outline | OutlineVariant => Some(highest_surface),
            InverseOnSurface | InversePrimary => Some(InverseSurface),
            Primary | PrimaryContainer | Secondary | SecondaryContainer | Tertiary
            | TertiaryContainer | Error | ErrorContainer | PrimaryFixed | PrimaryFixedDim
            | SecondaryFixed | SecondaryFixedDim | TertiaryFixed | TertiaryFixedDim => {
                Some(highest_surface)
            }
            OnPrimary => Some(Primary),
            OnPrimaryContainer => Some(PrimaryContainer),
            OnSecondary => Some(Secondary),
            OnSecondaryContainer => Some(SecondaryContainer),
            OnTertiary => Some(Tertiary),
            OnTertiaryContainer => Some(TertiaryContainer),
            OnError => Some(Error),
            OnErrorContainer => Some(ErrorContainer),
            OnPrimaryFixed | OnPrimaryFixedVariant => Some(PrimaryFixedDim),
            OnSecondaryFixed | OnSecondaryFixedVariant => Some(SecondaryFixedDim),
            OnTertiaryFixed | OnTertiaryFixedVariant => Some(TertiaryFixedDim),
        }
    }

    /// 2 つ目の背景（fixed 系の前景色のみ）。
    pub(crate) fn second_background(self) -> Option<ColorRole> {
        use ColorRole::*;
        match self {
            OnPrimaryFixed | OnPrimaryFixedVariant => Some(PrimaryFixed),
            OnSecondaryFixed | OnSecondaryFixedVariant => Some(SecondaryFixed),
            OnTertiaryFixed | OnTertiaryFixedVariant => Some(TertiaryFixed),
            _ => None,
        }
    }

    /// 背景に対して確保するコントラスト比。
    pub(crate) fn contrast_curve(self) -> Option<ContrastCurve> {
        use ColorRole::*;
        match self {
            PrimaryPaletteKeyColor
            | SecondaryPaletteKeyColor
            | TertiaryPaletteKeyColor
            | NeutralPaletteKeyColor
            | NeutralVariantPaletteKeyColor
            | ErrorPaletteKeyColor
            | Background
            | Surface
            | SurfaceDim
            | SurfaceBright
            | SurfaceContainerLowest
            | SurfaceContainerLow
            | SurfaceContainer
            | SurfaceContainerHigh
            | SurfaceContainerHighest
            | SurfaceVariant
            | InverseSurface
            | Shadow
            | Scrim
            | SurfaceTint => None,
            OnBackground => Some(ContrastCurve::new(3.0, 3.0, 4.5, 7.0)),
            OnSurface | InverseOnSurface | OnPrimary | OnSecondary | OnTertiary | OnError
            | OnPrimaryFixed | OnSecondaryFixed | OnTertiaryFixed => {
                Some(ContrastCurve::new(4.5, 7.0, 11.0, 21.0))
            }
            OnSurfaceVariant
            | OnPrimaryContainer
            | OnSecondaryContainer
            | OnTertiaryContainer
            | OnErrorContainer
            | OnPrimaryFixedVariant
            | OnSecondaryFixedVariant
            | OnTertiaryFixedVariant => Some(ContrastCurve::new(3.0, 4.5, 7.0, 11.0)),
            Outline => Some(ContrastCurve::new(1.5, 3.0, 4.5, 7.0)),
            OutlineVariant | PrimaryContainer | SecondaryContainer | TertiaryContainer
            | ErrorContainer | PrimaryFixed | PrimaryFixedDim | SecondaryFixed
            | SecondaryFixedDim | TertiaryFixed | TertiaryFixedDim => {
                Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5))
            }
            Primary | Secondary | Tertiary | Error | InversePrimary => {
                Some(ContrastCurve::new(3.0, 4.5, 7.0, 7.0))
            }
        }
    }

    /// 他の役割とのトーン差の制約。
    pub(crate) fn tone_delta_pair(self) -> Option<ToneDeltaPair> {
        use ColorRole::*;
        use TonePolarity::{Lighter, Nearer};
        match self {
            Primary | PrimaryContainer => Some(pair(PrimaryContainer, Primary, Nearer, false)),
            Secondary | SecondaryContainer => {
                Some(pair(SecondaryContainer, Secondary, Nearer, false))
            }
            Tertiary | TertiaryContainer => Some(pair(TertiaryContainer, Tertiary, Nearer, false)),
            Error | ErrorContainer => Some(pair(ErrorContainer, Error, Nearer, false)),
            PrimaryFixed | PrimaryFixedDim => {
                Some(pair(PrimaryFixed, PrimaryFixedDim, Lighter, true))
            }
            SecondaryFixed | SecondaryFixedDim => {
                Some(pair(SecondaryFixed, SecondaryFixedDim, Lighter, true))
            }
            TertiaryFixed | TertiaryFixedDim => {
                Some(pair(TertiaryFixed, TertiaryFixedDim, Lighter, true))
            }
            _ => None,
        }
    }
}
