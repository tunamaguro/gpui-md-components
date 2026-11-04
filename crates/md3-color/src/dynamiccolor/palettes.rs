// Ported from material-color-utilities
// (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/dynamiccolor/dynamic_scheme.ts
//         (DynamicScheme.getPiecewiseHue, DynamicScheme.getRotatedHue,
//          DynamicSchemePalettesDelegateImpl2021, DynamicSchemePalettesDelegateImpl2025)
//         @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2022 Google LLC. Licensed under the Apache License, Version 2.0.

//! [`Variant`] ごとのパレットの規則

use super::scheme::Palettes;
use super::{Platform, SpecVersion, Variant};
use crate::dislike;
use crate::hct::Hct;
use crate::palettes::TonalPalette;
use crate::temperature::TemperatureCache;
use crate::utils::math;

/// `variant` の規則で 6 つのパレットを作る。[`Variant::Cmf`] は対象外
///
/// 移植元の規則はコントラストレベルを引数に取るが、どの規則も参照しない。移植元は 2025 仕様のときだけ 2025 仕様の規則を使い、2021 仕様と 2026 仕様では 2021 仕様の規則を使う
pub(crate) fn for_variant(
    spec_version: SpecVersion,
    variant: Variant,
    source: &Hct,
    is_dark: bool,
    platform: Platform,
) -> Palettes {
    let args = Args {
        variant,
        source,
        is_dark,
        platform,
    };
    if spec_version == SpecVersion::Spec2025 {
        Palettes {
            primary: primary_2025(&args),
            secondary: secondary_2025(&args),
            tertiary: tertiary_2025(&args),
            neutral: neutral_2025(&args),
            neutral_variant: neutral_variant_2025(&args),
            error: error_2025(&args).unwrap_or_else(default_error),
        }
    } else {
        Palettes {
            primary: primary_2021(&args),
            secondary: secondary_2021(&args),
            tertiary: tertiary_2021(&args),
            neutral: neutral_2021(&args),
            neutral_variant: neutral_variant_2021(&args),
            error: default_error(),
        }
    }
}

struct Args<'a> {
    variant: Variant,
    source: &'a Hct,
    is_dark: bool,
    platform: Platform,
}

fn palette(hue: f64, chroma: f64) -> TonalPalette {
    TonalPalette::from_hue_and_chroma_raw(hue, chroma)
}

/// 移植元で `getErrorPalette` が `undefined` を返したときに `DynamicScheme` が使うパレット
fn default_error() -> TonalPalette {
    palette(25.0, 84.0)
}

/// 元の色の色相が `hue_breakpoints[i]` 以上 `hue_breakpoints[i + 1]` 未満なら `hues[i]` を返す。
/// どの区間にも入らなければ元の色の色相を返す
pub(crate) fn piecewise_hue(source: &Hct, hue_breakpoints: &[f64], hues: &[f64]) -> f64 {
    let source_hue = source.hue_raw();
    for (breakpoints, hue) in hue_breakpoints.windows(2).zip(hues) {
        if let [start, end] = breakpoints
            && source_hue >= *start
            && source_hue < *end
        {
            return math::sanitize_degrees_double(*hue);
        }
    }
    source_hue
}

/// 元の色の色相を、区間ごとの回転量 `rotations` だけ回した色相
pub(crate) fn rotated_hue(source: &Hct, hue_breakpoints: &[f64], rotations: &[f64]) -> f64 {
    let mut rotation = piecewise_hue(source, hue_breakpoints, rotations);
    if usize::min(hue_breakpoints.len().saturating_sub(1), rotations.len()) == 0 {
        rotation = 0.0;
    }
    math::sanitize_degrees_double(source.hue_raw() + rotation)
}

fn primary_2021(a: &Args) -> TonalPalette {
    let s = a.source;
    match a.variant {
        Variant::Content | Variant::Fidelity => palette(s.hue_raw(), s.chroma_raw()),
        Variant::FruitSalad => palette(math::sanitize_degrees_double(s.hue_raw() - 50.0), 48.0),
        Variant::Monochrome => palette(s.hue_raw(), 0.0),
        Variant::Neutral => palette(s.hue_raw(), 12.0),
        Variant::Rainbow => palette(s.hue_raw(), 48.0),
        Variant::TonalSpot => palette(s.hue_raw(), 36.0),
        Variant::Expressive => palette(math::sanitize_degrees_double(s.hue_raw() + 240.0), 40.0),
        Variant::Vibrant => palette(s.hue_raw(), 200.0),
        // 移植元は例外を投げる。Cmf のパレットは scheme.rs の cmf_palettes で作る
        Variant::Cmf => palette(s.hue_raw(), s.chroma_raw()),
    }
}

fn secondary_2021(a: &Args) -> TonalPalette {
    let s = a.source;
    match a.variant {
        Variant::Content | Variant::Fidelity => palette(
            s.hue_raw(),
            math::max_js(s.chroma_raw() - 32.0, s.chroma_raw() * 0.5),
        ),
        Variant::FruitSalad => palette(math::sanitize_degrees_double(s.hue_raw() - 50.0), 36.0),
        Variant::Monochrome => palette(s.hue_raw(), 0.0),
        Variant::Neutral => palette(s.hue_raw(), 8.0),
        Variant::Rainbow => palette(s.hue_raw(), 16.0),
        Variant::TonalSpot => palette(s.hue_raw(), 16.0),
        Variant::Expressive => palette(
            rotated_hue(
                s,
                &[0.0, 21.0, 51.0, 121.0, 151.0, 191.0, 271.0, 321.0, 360.0],
                &[45.0, 95.0, 45.0, 20.0, 45.0, 90.0, 45.0, 45.0, 45.0],
            ),
            24.0,
        ),
        Variant::Vibrant => palette(
            rotated_hue(
                s,
                &[0.0, 41.0, 61.0, 101.0, 131.0, 181.0, 251.0, 301.0, 360.0],
                &[18.0, 15.0, 10.0, 12.0, 15.0, 18.0, 15.0, 12.0, 12.0],
            ),
            24.0,
        ),
        Variant::Cmf => palette(s.hue_raw(), s.chroma_raw()),
    }
}

fn tertiary_2021(a: &Args) -> TonalPalette {
    let s = a.source;
    match a.variant {
        Variant::Content => {
            let analogous = TemperatureCache::new(*s).analogous(3, 6);
            // analogous(3, _) は常に 3 色を返す
            let hct = analogous.get(2).copied().unwrap_or(*s);
            TonalPalette::from_hct(dislike::fix_if_disliked(hct))
        }
        Variant::Fidelity => TonalPalette::from_hct(dislike::fix_if_disliked(
            TemperatureCache::new(*s).complement(),
        )),
        Variant::FruitSalad => palette(s.hue_raw(), 36.0),
        Variant::Monochrome => palette(s.hue_raw(), 0.0),
        Variant::Neutral => palette(s.hue_raw(), 16.0),
        Variant::Rainbow | Variant::TonalSpot => {
            palette(math::sanitize_degrees_double(s.hue_raw() + 60.0), 24.0)
        }
        Variant::Expressive => palette(
            rotated_hue(
                s,
                &[0.0, 21.0, 51.0, 121.0, 151.0, 191.0, 271.0, 321.0, 360.0],
                &[120.0, 120.0, 20.0, 45.0, 20.0, 15.0, 20.0, 120.0, 120.0],
            ),
            32.0,
        ),
        Variant::Vibrant => palette(
            rotated_hue(
                s,
                &[0.0, 41.0, 61.0, 101.0, 131.0, 181.0, 251.0, 301.0, 360.0],
                &[35.0, 30.0, 20.0, 25.0, 30.0, 35.0, 30.0, 25.0, 25.0],
            ),
            32.0,
        ),
        Variant::Cmf => palette(s.hue_raw(), s.chroma_raw()),
    }
}

fn neutral_2021(a: &Args) -> TonalPalette {
    let s = a.source;
    match a.variant {
        Variant::Content | Variant::Fidelity => palette(s.hue_raw(), s.chroma_raw() / 8.0),
        Variant::FruitSalad => palette(s.hue_raw(), 10.0),
        Variant::Monochrome => palette(s.hue_raw(), 0.0),
        Variant::Neutral => palette(s.hue_raw(), 2.0),
        Variant::Rainbow => palette(s.hue_raw(), 0.0),
        Variant::TonalSpot => palette(s.hue_raw(), 6.0),
        Variant::Expressive => palette(math::sanitize_degrees_double(s.hue_raw() + 15.0), 8.0),
        Variant::Vibrant => palette(s.hue_raw(), 10.0),
        Variant::Cmf => palette(s.hue_raw(), s.chroma_raw()),
    }
}

fn neutral_variant_2021(a: &Args) -> TonalPalette {
    let s = a.source;
    match a.variant {
        Variant::Content | Variant::Fidelity => palette(s.hue_raw(), (s.chroma_raw() / 8.0) + 4.0),
        Variant::FruitSalad => palette(s.hue_raw(), 16.0),
        Variant::Monochrome => palette(s.hue_raw(), 0.0),
        Variant::Neutral => palette(s.hue_raw(), 2.0),
        Variant::Rainbow => palette(s.hue_raw(), 0.0),
        Variant::TonalSpot => palette(s.hue_raw(), 8.0),
        Variant::Expressive => palette(math::sanitize_degrees_double(s.hue_raw() + 15.0), 12.0),
        Variant::Vibrant => palette(s.hue_raw(), 12.0),
        Variant::Cmf => palette(s.hue_raw(), s.chroma_raw()),
    }
}

fn is_watch_blue_hue(source: &Hct) -> bool {
    250.0 <= source.hue_raw() && source.hue_raw() <= 270.0
}

fn primary_2025(a: &Args) -> TonalPalette {
    let s = a.source;
    let phone = a.platform == Platform::Phone;
    match a.variant {
        Variant::Neutral => palette(
            s.hue_raw(),
            if phone {
                if Hct::is_blue(s.hue_raw()) { 12.0 } else { 8.0 }
            } else if is_watch_blue_hue(s) {
                16.0
            } else {
                12.0
            },
        ),
        Variant::TonalSpot => palette(s.hue_raw(), if phone && a.is_dark { 26.0 } else { 32.0 }),
        Variant::Expressive => palette(
            s.hue_raw(),
            if phone {
                if a.is_dark { 36.0 } else { 48.0 }
            } else {
                40.0
            },
        ),
        Variant::Vibrant => palette(s.hue_raw(), if phone { 74.0 } else { 48.0 }),
        Variant::Content if !phone => palette(s.hue_raw(), s.chroma_raw()),
        _ => primary_2021(a),
    }
}

fn secondary_2025(a: &Args) -> TonalPalette {
    let s = a.source;
    let phone = a.platform == Platform::Phone;
    match a.variant {
        Variant::Neutral => palette(
            s.hue_raw(),
            if phone {
                if Hct::is_blue(s.hue_raw()) { 6.0 } else { 4.0 }
            } else if is_watch_blue_hue(s) {
                10.0
            } else {
                6.0
            },
        ),
        Variant::TonalSpot => palette(s.hue_raw(), 16.0),
        Variant::Expressive => palette(
            if !phone {
                rotated_hue(
                    s,
                    &[0.0, 21.0, 51.0, 121.0, 151.0, 191.0, 271.0, 321.0, 360.0],
                    &[
                        204.0, 175.0, 155.0, 150.0, 204.0, 260.0, 204.0, 204.0, 204.0,
                    ],
                )
            } else {
                rotated_hue(
                    s,
                    &[0.0, 105.0, 140.0, 204.0, 253.0, 278.0, 300.0, 333.0, 360.0],
                    &[-160.0, 155.0, -100.0, 96.0, -96.0, -156.0, -165.0, -160.0],
                )
            },
            if phone {
                if a.is_dark { 16.0 } else { 24.0 }
            } else {
                24.0
            },
        ),
        Variant::Vibrant => palette(
            if !phone {
                rotated_hue(
                    s,
                    &[0.0, 41.0, 61.0, 101.0, 131.0, 181.0, 251.0, 301.0, 360.0],
                    &[-14.0, 0.0, 355.0, 357.0, 0.0, 3.0, 10.0, 331.0, 331.0],
                )
            } else {
                rotated_hue(
                    s,
                    &[0.0, 38.0, 105.0, 140.0, 333.0, 360.0],
                    &[-14.0, 10.0, -14.0, 10.0, -14.0],
                )
            },
            if phone { 56.0 } else { 36.0 },
        ),
        Variant::Content if !phone => palette(
            s.hue_raw(),
            math::clamp_double(16.0, 36.0, s.chroma_raw() * 0.5),
        ),
        _ => secondary_2021(a),
    }
}

fn tertiary_2025(a: &Args) -> TonalPalette {
    let s = a.source;
    let phone = a.platform == Platform::Phone;
    match a.variant {
        Variant::Neutral => palette(
            if !phone {
                rotated_hue(
                    s,
                    &[0.0, 41.0, 61.0, 101.0, 131.0, 181.0, 250.0, 270.0, 360.0],
                    &[25.0, 25.0, 26.0, 25.0, -39.0, 25.0, 28.0, 25.0, 25.0],
                )
            } else {
                rotated_hue(
                    s,
                    &[0.0, 38.0, 105.0, 161.0, 204.0, 278.0, 333.0, 360.0],
                    &[-32.0, 26.0, 10.0, -39.0, 24.0, -15.0, -32.0],
                )
            },
            if phone { 20.0 } else { 36.0 },
        ),
        Variant::TonalSpot => palette(
            if !phone {
                rotated_hue(
                    s,
                    &[0.0, 40.0, 101.0, 210.0, 301.0, 360.0],
                    &[-32.0, 32.0, -32.0, 32.0, -32.0, -32.0],
                )
            } else {
                rotated_hue(
                    s,
                    &[0.0, 20.0, 71.0, 161.0, 333.0, 360.0],
                    &[-40.0, 48.0, -32.0, 40.0, -32.0],
                )
            },
            if phone { 28.0 } else { 32.0 },
        ),
        Variant::Expressive => palette(
            if !phone {
                rotated_hue(
                    s,
                    &[0.0, 21.0, 51.0, 121.0, 151.0, 191.0, 271.0, 321.0, 360.0],
                    &[
                        200.0, 200.0, 160.0, 169.0, 200.0, 220.0, 200.0, 200.0, 200.0,
                    ],
                )
            } else {
                rotated_hue(
                    s,
                    &[0.0, 105.0, 140.0, 204.0, 253.0, 278.0, 300.0, 333.0, 360.0],
                    &[-165.0, 160.0, -105.0, 101.0, -101.0, -160.0, -170.0, -165.0],
                )
            },
            48.0,
        ),
        Variant::Vibrant => palette(
            if !phone {
                rotated_hue(
                    s,
                    &[0.0, 41.0, 61.0, 101.0, 131.0, 181.0, 251.0, 301.0, 360.0],
                    &[-72.0, 55.0, 45.0, 50.0, 55.0, 50.0, 62.0, 303.0, 303.0],
                )
            } else {
                rotated_hue(
                    s,
                    &[0.0, 38.0, 71.0, 105.0, 140.0, 161.0, 253.0, 333.0, 360.0],
                    &[-72.0, 35.0, 24.0, -24.0, 62.0, 50.0, 62.0, -72.0],
                )
            },
            if phone { 56.0 } else { 48.0 },
        ),
        Variant::Content if !phone => palette(
            rotated_hue(
                s,
                &[0.0, 40.0, 101.0, 210.0, 301.0, 360.0],
                &[-32.0, 32.0, -32.0, 32.0, -32.0, -32.0],
            ),
            s.chroma_raw(),
        ),
        _ => tertiary_2021(a),
    }
}

fn expressive_neutral_hue(a: &Args) -> f64 {
    if a.platform == Platform::Watch {
        return rotated_hue(
            a.source,
            &[0.0, 21.0, 51.0, 121.0, 151.0, 191.0, 271.0, 321.0, 360.0],
            &[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 10.0, 0.0, 0.0],
        );
    }
    rotated_hue(
        a.source,
        &[0.0, 71.0, 124.0, 253.0, 278.0, 300.0, 360.0],
        &[10.0, 0.0, 10.0, 0.0, 10.0, 0.0],
    )
}

fn expressive_neutral_chroma(a: &Args) -> f64 {
    let neutral_hue = expressive_neutral_hue(a);
    if a.platform == Platform::Phone {
        if a.is_dark {
            if Hct::is_yellow(neutral_hue) {
                6.0
            } else {
                14.0
            }
        } else {
            18.0
        }
    } else {
        12.0
    }
}

fn vibrant_neutral_hue(a: &Args) -> f64 {
    if a.platform == Platform::Watch {
        return rotated_hue(
            a.source,
            &[0.0, 41.0, 61.0, 101.0, 131.0, 181.0, 251.0, 301.0, 360.0],
            &[-14.0, 0.0, 355.0, 357.0, 0.0, 3.0, 10.0, 331.0, 331.0],
        );
    }
    rotated_hue(
        a.source,
        &[0.0, 38.0, 105.0, 140.0, 333.0, 360.0],
        &[-14.0, 10.0, -14.0, 10.0, -14.0],
    )
}

fn vibrant_neutral_chroma(a: &Args) -> f64 {
    if a.platform == Platform::Phone || is_watch_blue_hue(a.source) {
        28.0
    } else {
        20.0
    }
}

fn neutral_2025(a: &Args) -> TonalPalette {
    let s = a.source;
    let phone = a.platform == Platform::Phone;
    match a.variant {
        Variant::Neutral => palette(s.hue_raw(), if phone { 1.4 } else { 6.0 }),
        Variant::TonalSpot => palette(s.hue_raw(), if phone { 5.0 } else { 10.0 }),
        Variant::Expressive => palette(expressive_neutral_hue(a), expressive_neutral_chroma(a)),
        Variant::Vibrant => palette(vibrant_neutral_hue(a), vibrant_neutral_chroma(a)),
        Variant::Content if !phone => palette(
            s.hue_raw(),
            math::clamp_double(6.0, 20.0, s.chroma_raw() * 0.333),
        ),
        _ => neutral_2021(a),
    }
}

fn neutral_variant_2025(a: &Args) -> TonalPalette {
    let s = a.source;
    if a.platform == Platform::Watch {
        return match a.variant {
            Variant::Neutral
            | Variant::TonalSpot
            | Variant::Expressive
            | Variant::Vibrant
            | Variant::Content => neutral_2025(a),
            _ => neutral_variant_2021(a),
        };
    }
    match a.variant {
        Variant::Neutral => palette(s.hue_raw(), 1.4 * 2.2),
        Variant::TonalSpot => palette(s.hue_raw(), 5.0 * 1.7),
        Variant::Expressive => {
            let expressive_neutral_hue = expressive_neutral_hue(a);
            let expressive_neutral_chroma = expressive_neutral_chroma(a);
            palette(
                expressive_neutral_hue,
                expressive_neutral_chroma
                    * if (105.0..125.0).contains(&expressive_neutral_hue) {
                        1.6
                    } else {
                        2.3
                    },
            )
        }
        Variant::Vibrant => palette(vibrant_neutral_hue(a), vibrant_neutral_chroma(a) * 1.29),
        _ => neutral_variant_2021(a),
    }
}

fn error_2025(a: &Args) -> Option<TonalPalette> {
    let s = a.source;
    let phone = a.platform == Platform::Phone;
    let error_hue = piecewise_hue(
        s,
        &[0.0, 3.0, 13.0, 23.0, 33.0, 43.0, 153.0, 273.0, 360.0],
        &[12.0, 22.0, 32.0, 12.0, 22.0, 32.0, 22.0, 12.0],
    );
    match a.variant {
        Variant::Neutral => Some(palette(error_hue, if phone { 50.0 } else { 40.0 })),
        Variant::TonalSpot => Some(palette(error_hue, if phone { 60.0 } else { 48.0 })),
        Variant::Expressive => Some(palette(error_hue, if phone { 64.0 } else { 48.0 })),
        Variant::Vibrant => Some(palette(error_hue, if phone { 80.0 } else { 48.0 })),
        Variant::Content if !phone => Some(palette(error_hue, math::max_js(40.0, s.chroma_raw()))),
        _ => None,
    }
}
