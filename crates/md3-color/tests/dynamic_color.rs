// Ported from material-color-utilities (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/dynamiccolor/dynamic_color_test.ts, fixed_dynamic_color_test.ts,
//         typescript/scheme/dynamic_scheme_test.ts @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2021 Google LLC. Licensed under the Apache License, Version 2.0.

//! 配色がコントラスト・トーン差・背景トーンの制約を満たすことの確認。

use md3_color::scheme::rotated_hue;
use md3_color::{
    Argb, ColorRole, ContrastCurve, ContrastLevel, DynamicScheme, Hct, Tone, Variant, contrast,
};

const SEEDS: [u32; 4] = [0xFFFF0000, 0xFFFFFF00, 0xFF00FF00, 0xFF0000FF];
const CONTRAST_LEVELS: [f64; 9] = [-1.0, -0.75, -0.5, -0.25, 0.0, 0.25, 0.5, 0.75, 1.0];

fn schemes() -> Vec<DynamicScheme> {
    let mut schemes = Vec::new();
    for seed in SEEDS {
        let hct = Hct::from_argb(Argb::new(seed));
        for contrast_level in CONTRAST_LEVELS {
            for is_dark in [false, true] {
                for variant in [
                    Variant::Content,
                    Variant::Expressive,
                    Variant::Fidelity,
                    Variant::Monochrome,
                    Variant::Neutral,
                    Variant::TonalSpot,
                    Variant::Vibrant,
                ] {
                    schemes.push(DynamicScheme::new(
                        hct,
                        variant,
                        is_dark,
                        ContrastLevel::new(contrast_level),
                    ));
                }
            }
        }
    }
    schemes
}

fn ratio(a: f64, b: f64) -> f64 {
    contrast::ratio_of_tones(Tone::new(a), Tone::new(b))
}

fn describe(s: &DynamicScheme) -> String {
    format!(
        "{:?} dark={} contrast={} seed={:#010x}",
        s.variant(),
        s.is_dark(),
        s.contrast_level().value(),
        s.source_color().to_argb().to_u32()
    )
}

#[test]
fn generates_colors_respecting_contrast() {
    use ColorRole::*;
    let text_surface_pairs = [
        (OnPrimary, Primary),
        (OnPrimaryContainer, PrimaryContainer),
        (OnSecondary, Secondary),
        (OnSecondaryContainer, SecondaryContainer),
        (OnTertiary, Tertiary),
        (OnTertiaryContainer, TertiaryContainer),
        (OnError, Error),
        (OnErrorContainer, ErrorContainer),
        (OnBackground, Background),
        (OnSurfaceVariant, SurfaceBright),
        (OnSurfaceVariant, SurfaceDim),
    ];
    for scheme in schemes() {
        for (fg, bg) in text_surface_pairs {
            let foreground_tone = scheme.hct(fg).tone().value();
            let background_tone = scheme.hct(bg).tone().value();
            let contrast = ratio(foreground_tone, background_tone);
            let minimum_requirement = if scheme.contrast_level().value() >= 0.0 {
                4.5
            } else {
                3.0
            };
            assert!(
                contrast >= minimum_requirement,
                "{} on {} is {contrast}, needed {minimum_requirement} ({})",
                fg.name(),
                bg.name(),
                describe(&scheme)
            );
        }
    }
}

enum Constraint {
    Contrast {
        values: ContrastCurve,
        fore: &'static [ColorRole],
        back: &'static [ColorRole],
        respectively: bool,
    },
    Delta {
        delta: f64,
        fore: &'static [ColorRole],
        back: &'static [ColorRole],
        polarity: Polarity,
    },
    Background {
        objects: &'static [ColorRole],
    },
}

#[derive(Clone, Copy)]
enum Polarity {
    Farther,
    Darker,
}

fn min_requirement(curve: ContrastCurve, level: f64) -> f64 {
    if level >= 1.0 {
        curve.high
    } else if level >= 0.5 {
        curve.medium
    } else if level >= 0.0 {
        curve.normal
    } else {
        curve.low
    }
}

fn pairs(
    respectively: bool,
    fores: &[ColorRole],
    backs: &[ColorRole],
) -> Vec<(ColorRole, ColorRole)> {
    if respectively {
        fores.iter().copied().zip(backs.iter().copied()).collect()
    } else {
        fores
            .iter()
            .flat_map(|&f| backs.iter().map(move |&b| (f, b)))
            .collect()
    }
}

#[test]
fn constraint_conformance() {
    use ColorRole::*;
    const LIMITING_SURFACES: &[ColorRole] = &[SurfaceDim, SurfaceBright];
    let constraints = [
        Constraint::Contrast {
            values: ContrastCurve::new(4.5, 7.0, 11.0, 21.0),
            fore: &[OnSurface],
            back: LIMITING_SURFACES,
            respectively: false,
        },
        Constraint::Contrast {
            values: ContrastCurve::new(3.0, 4.5, 7.0, 11.0),
            fore: &[OnSurfaceVariant],
            back: LIMITING_SURFACES,
            respectively: false,
        },
        Constraint::Contrast {
            values: ContrastCurve::new(3.0, 4.5, 7.0, 7.0),
            fore: &[Primary, Secondary, Tertiary, Error],
            back: LIMITING_SURFACES,
            respectively: false,
        },
        Constraint::Contrast {
            values: ContrastCurve::new(1.5, 3.0, 4.5, 7.0),
            fore: &[Outline],
            back: LIMITING_SURFACES,
            respectively: false,
        },
        Constraint::Contrast {
            values: ContrastCurve::new(0.0, 0.0, 3.0, 4.5),
            fore: &[
                PrimaryContainer,
                PrimaryFixed,
                PrimaryFixedDim,
                SecondaryContainer,
                SecondaryFixed,
                SecondaryFixedDim,
                TertiaryContainer,
                TertiaryFixed,
                TertiaryFixedDim,
                ErrorContainer,
                OutlineVariant,
            ],
            back: LIMITING_SURFACES,
            respectively: false,
        },
        Constraint::Contrast {
            values: ContrastCurve::new(4.5, 7.0, 11.0, 21.0),
            fore: &[InverseOnSurface],
            back: &[InverseSurface],
            respectively: false,
        },
        Constraint::Contrast {
            values: ContrastCurve::new(3.0, 4.5, 7.0, 7.0),
            fore: &[InversePrimary],
            back: &[InverseSurface],
            respectively: false,
        },
        Constraint::Contrast {
            values: ContrastCurve::new(4.5, 7.0, 11.0, 21.0),
            fore: &[OnPrimary, OnSecondary, OnTertiary, OnError],
            back: &[Primary, Secondary, Tertiary, Error],
            respectively: true,
        },
        Constraint::Contrast {
            values: ContrastCurve::new(3.0, 4.5, 7.0, 11.0),
            fore: &[
                OnPrimaryContainer,
                OnSecondaryContainer,
                OnTertiaryContainer,
                OnErrorContainer,
            ],
            back: &[
                PrimaryContainer,
                SecondaryContainer,
                TertiaryContainer,
                ErrorContainer,
            ],
            respectively: true,
        },
        Constraint::Contrast {
            values: ContrastCurve::new(4.5, 7.0, 11.0, 21.0),
            fore: &[OnPrimaryFixed],
            back: &[PrimaryFixed, PrimaryFixedDim],
            respectively: false,
        },
        Constraint::Contrast {
            values: ContrastCurve::new(4.5, 7.0, 11.0, 21.0),
            fore: &[OnSecondaryFixed],
            back: &[SecondaryFixed, SecondaryFixedDim],
            respectively: false,
        },
        Constraint::Contrast {
            values: ContrastCurve::new(4.5, 7.0, 11.0, 21.0),
            fore: &[OnTertiaryFixed],
            back: &[TertiaryFixed, TertiaryFixedDim],
            respectively: false,
        },
        Constraint::Contrast {
            values: ContrastCurve::new(3.0, 4.5, 7.0, 11.0),
            fore: &[OnPrimaryFixedVariant],
            back: &[PrimaryFixed, PrimaryFixedDim],
            respectively: false,
        },
        Constraint::Contrast {
            values: ContrastCurve::new(3.0, 4.5, 7.0, 11.0),
            fore: &[OnSecondaryFixedVariant],
            back: &[SecondaryFixed, SecondaryFixedDim],
            respectively: false,
        },
        Constraint::Contrast {
            values: ContrastCurve::new(3.0, 4.5, 7.0, 11.0),
            fore: &[OnTertiaryFixedVariant],
            back: &[TertiaryFixed, TertiaryFixedDim],
            respectively: false,
        },
        Constraint::Delta {
            delta: 10.0,
            fore: &[Primary, Secondary, Tertiary, Error],
            back: &[
                PrimaryContainer,
                SecondaryContainer,
                TertiaryContainer,
                ErrorContainer,
            ],
            polarity: Polarity::Farther,
        },
        Constraint::Delta {
            delta: 10.0,
            fore: &[PrimaryFixedDim, SecondaryFixedDim, TertiaryFixedDim],
            back: &[PrimaryFixed, SecondaryFixed, TertiaryFixed],
            polarity: Polarity::Darker,
        },
        Constraint::Background {
            objects: &[
                Background,
                Error,
                ErrorContainer,
                Primary,
                PrimaryContainer,
                PrimaryFixed,
                PrimaryFixedDim,
                Secondary,
                SecondaryContainer,
                SecondaryFixed,
                SecondaryFixedDim,
                Surface,
                SurfaceBright,
                SurfaceContainer,
                SurfaceContainerHigh,
                SurfaceContainerHighest,
                SurfaceContainerLow,
                SurfaceContainerLowest,
                SurfaceDim,
                SurfaceTint,
                SurfaceVariant,
                Tertiary,
                TertiaryContainer,
                TertiaryFixed,
                TertiaryFixedDim,
            ],
        },
    ];

    let mut failures = Vec::new();
    for scheme in schemes() {
        let tone = |role: ColorRole| scheme.argb(role).lstar().value();
        for constraint in &constraints {
            match *constraint {
                Constraint::Contrast {
                    values,
                    fore,
                    back,
                    respectively,
                } => {
                    let contrast_tolerance = 0.05;
                    let min_requirement = min_requirement(values, scheme.contrast_level().value());
                    for (f, b) in pairs(respectively, fore, back) {
                        let ftone = tone(f);
                        let btone = tone(b);
                        let contrast = ratio(ftone, btone);
                        let failing = if min_requirement <= 4.5 {
                            contrast < min_requirement - contrast_tolerance
                        } else {
                            ftone != 0.0
                                && btone != 0.0
                                && ftone != 100.0
                                && btone != 100.0
                                && contrast < min_requirement - contrast_tolerance
                        };
                        if failing {
                            failures.push(format!(
                                "Contrast {} {ftone:.2} {} {btone:.2} {contrast:.2} {min_requirement} ({})",
                                f.name(),
                                b.name(),
                                describe(&scheme)
                            ));
                        }
                    }
                }
                Constraint::Delta {
                    delta,
                    fore,
                    back,
                    polarity,
                } => {
                    for (f, b) in pairs(true, fore, back) {
                        let ftone = tone(f);
                        let btone = tone(b);
                        let is_lighter = match polarity {
                            Polarity::Farther => scheme.is_dark(),
                            Polarity::Darker => false,
                        };
                        let observed_delta = if is_lighter {
                            ftone - btone
                        } else {
                            btone - ftone
                        };
                        if observed_delta < delta - 0.5 {
                            failures.push(format!(
                                "Delta {} {ftone:.2} {} {btone:.2} {observed_delta:.2} {delta} ({})",
                                f.name(),
                                b.name(),
                                describe(&scheme)
                            ));
                        }
                    }
                }
                Constraint::Background { objects } => {
                    for &bg in objects {
                        let bgtone = tone(bg);
                        if (50.5..59.5).contains(&bgtone) {
                            failures.push(format!(
                                "Background {} {bgtone:.2} ({})",
                                bg.name(),
                                describe(&scheme)
                            ));
                        }
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn red_scheme(variant: Variant, is_dark: bool) -> DynamicScheme {
    DynamicScheme::new(
        Hct::from_argb(Argb::new(0xFFFF0000)),
        variant,
        is_dark,
        ContrastLevel::STANDARD,
    )
}

#[track_caller]
fn assert_tones(scheme: &DynamicScheme, expected: &[(ColorRole, f64)]) {
    for &(role, tone) in expected {
        let actual = scheme.hct(role).tone().value();
        assert!(
            (actual - tone).abs() < 0.5,
            "{}: {actual} != {tone}",
            role.name()
        );
    }
}

#[test]
fn fixed_colors_in_non_monochrome_schemes() {
    use ColorRole::*;
    assert_tones(
        &red_scheme(Variant::TonalSpot, true),
        &[
            (PrimaryFixed, 90.0),
            (PrimaryFixedDim, 80.0),
            (OnPrimaryFixed, 10.0),
            (OnPrimaryFixedVariant, 30.0),
            (SecondaryFixed, 90.0),
            (SecondaryFixedDim, 80.0),
            (OnSecondaryFixed, 10.0),
            (OnSecondaryFixedVariant, 30.0),
            (TertiaryFixed, 90.0),
            (TertiaryFixedDim, 80.0),
            (OnTertiaryFixed, 10.0),
            (OnTertiaryFixedVariant, 30.0),
        ],
    );
}

#[test]
fn fixed_argb_colors_in_non_monochrome_schemes() {
    use ColorRole::*;
    let scheme = red_scheme(Variant::TonalSpot, true);
    let expected = [
        (PrimaryFixed, 0xFFFFDAD4),
        (PrimaryFixedDim, 0xFFFFB4A8),
        (OnPrimaryFixed, 0xFF3A0905),
        (OnPrimaryFixedVariant, 0xFF73342A),
        (SecondaryFixed, 0xFFFFDAD4),
        (SecondaryFixedDim, 0xFFE7BDB6),
        (OnSecondaryFixed, 0xFF2C1512),
        (OnSecondaryFixedVariant, 0xFF5D3F3B),
        (TertiaryFixed, 0xFFFBDFA6),
        (TertiaryFixedDim, 0xFFDEC48C),
        (OnTertiaryFixed, 0xFF251A00),
        (OnTertiaryFixedVariant, 0xFF564419),
    ];
    for (role, argb) in expected {
        assert_eq!(scheme.argb(role), Argb::new(argb), "{}", role.name());
    }
}

const MONOCHROME_FIXED_TONES: [(ColorRole, f64); 12] = [
    (ColorRole::PrimaryFixed, 40.0),
    (ColorRole::PrimaryFixedDim, 30.0),
    (ColorRole::OnPrimaryFixed, 100.0),
    (ColorRole::OnPrimaryFixedVariant, 90.0),
    (ColorRole::SecondaryFixed, 80.0),
    (ColorRole::SecondaryFixedDim, 70.0),
    (ColorRole::OnSecondaryFixed, 10.0),
    (ColorRole::OnSecondaryFixedVariant, 25.0),
    (ColorRole::TertiaryFixed, 40.0),
    (ColorRole::TertiaryFixedDim, 30.0),
    (ColorRole::OnTertiaryFixed, 100.0),
    (ColorRole::OnTertiaryFixedVariant, 90.0),
];

#[test]
fn fixed_colors_in_light_monochrome_schemes() {
    assert_tones(
        &red_scheme(Variant::Monochrome, false),
        &MONOCHROME_FIXED_TONES,
    );
}

#[test]
fn fixed_colors_in_dark_monochrome_schemes() {
    assert_tones(
        &red_scheme(Variant::Monochrome, true),
        &MONOCHROME_FIXED_TONES,
    );
}

fn hct_43() -> Hct {
    Hct::new(
        md3_color::Hue::new(43.0),
        md3_color::Chroma::new(16.0),
        Tone::new(16.0),
    )
}

#[track_caller]
fn assert_hue(actual: f64, expected: f64) {
    // jasmine の toBeCloseTo(expected, 0.4) は誤差 10^-0.4 / 2 ≒ 0.199 未満。
    assert!(
        (actual - expected).abs() < 10f64.powf(-0.4) / 2.0,
        "{actual} != {expected}"
    );
}

#[test]
fn rotated_hue_0_length_input() {
    assert_hue(rotated_hue(&hct_43(), &[], &[]).degrees(), 43.0);
}

#[test]
fn rotated_hue_1_length_input_no_rotation() {
    assert_hue(rotated_hue(&hct_43(), &[0.0], &[0.0]).degrees(), 43.0);
}

#[test]
fn rotated_hue_input_length_mismatch() {
    assert_hue(rotated_hue(&hct_43(), &[0.0], &[0.0, 1.0]).degrees(), 43.0);
}

#[test]
fn rotated_hue_on_boundary_rotation_correct() {
    assert_hue(
        rotated_hue(&hct_43(), &[0.0, 42.0, 360.0], &[0.0, 15.0, 0.0]).degrees(),
        43.0 + 15.0,
    );
}

#[test]
fn rotated_hue_result_larger_than_360_degrees_wraps() {
    assert_hue(
        rotated_hue(&hct_43(), &[0.0, 42.0, 360.0], &[0.0, 480.0, 0.0]).degrees(),
        163.0,
    );
}
