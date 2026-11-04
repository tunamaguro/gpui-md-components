// Ported from material-color-utilities
// (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/{hct/hct_test.ts, hct/hct_round_trip_test.ts, contrast/contrast_test.ts,
//   dislike/dislike_analyzer_test.ts, palettes/palettes_test.ts,
//   temperature/temperature_cache_test.ts, blend/blend_test.ts,
//   dynamiccolor/dynamic_color_test.ts, dynamiccolor/fixed_dynamic_color_test.ts}
//   @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2021 Google LLC. Licensed under the Apache License, Version 2.0.
// contrast_test.ts: Copyright 2022 Google LLC.
// dislike_analyzer_test.ts, temperature_cache_test.ts, dynamic_color_test.ts,
// fixed_dynamic_color_test.ts: Copyright 2023 Google LLC.

//! 移植元の単体テスト
//!
//! 移植元から外したもの (CorePalette、旧 Scheme) のテストは含めない

use md3_color::{
    Argb, Cam16, Chroma, ColorRole, ContrastLevel, DynamicScheme, Hct, Hue, SchemeError,
    SpecVersion, TemperatureCache, TonalPalette, Tone, Variant, blend, contrast, dislike,
    temperature,
};

const RED: Argb = Argb::new(0xffff0000);
const GREEN: Argb = Argb::new(0xff00ff00);
const BLUE: Argb = Argb::new(0xff0000ff);
const YELLOW: Argb = Argb::new(0xffffff00);
const WHITE: Argb = Argb::new(0xffffffff);
const BLACK: Argb = Argb::new(0xff000000);

/// Jasmine の `toBeCloseTo(expected, digits)` と同じ判定
#[track_caller]
fn assert_close(actual: f64, expected: f64, digits: i32) {
    let tolerance = 10f64.powi(-digits) / 2.0;
    assert!(
        (actual - expected).abs() < tolerance,
        "{actual} が {expected} に近くない (桁数 {digits})"
    );
}

fn hct(hue: f64, chroma: f64, tone: f64) -> Hct {
    Hct::new(Hue::new(hue), Chroma::new(chroma), Tone::new(tone))
}

#[test]
fn cam16_の各値() {
    // [argb, hue, chroma, j, m, s, q]
    let cases = [
        (RED, 27.408, 113.358, 46.445, 89.494, 91.890, 105.989),
        (GREEN, 142.140, 108.410, 79.332, 85.588, 78.605, 138.520),
        (BLUE, 282.788, 87.231, 25.466, 68.867, 93.675, 78.481),
        (WHITE, 209.492, 2.869, 100.0, 2.265, 12.068, 155.521),
        (BLACK, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
    ];
    for (argb, hue, chroma, j, m, s, q) in cases {
        let cam = Cam16::from_argb(argb);
        assert_close(cam.hue(), hue, 3);
        assert_close(cam.chroma(), chroma, 3);
        assert_close(cam.j(), j, 3);
        assert_close(cam.m(), m, 3);
        assert_close(cam.s(), s, 3);
        assert_close(cam.q(), q, 3);
    }
}

#[test]
fn cam16_から_argb_に戻せる() {
    for argb in [RED, GREEN, BLUE] {
        assert_eq!(Cam16::from_argb(argb).to_argb(), argb);
    }
}

#[test]
fn argb_から_hct() {
    let green = Hct::from_argb(GREEN);
    assert_close(green.hue().degrees(), 142.139, 2);
    assert_close(green.chroma().value(), 108.410, 2);
    assert_close(green.tone().value(), 87.737, 2);

    let blue = Hct::from_argb(BLUE);
    assert_close(blue.hue().degrees(), 282.788, 2);
    assert_close(blue.chroma().value(), 87.230, 2);
    assert_close(blue.tone().value(), 32.302, 2);

    let blue_tone_90 = hct(282.788, 87.230, 90.0);
    assert_close(blue_tone_90.hue().degrees(), 282.239, 2);
    assert_close(blue_tone_90.chroma().value(), 19.144, 2);
    assert_close(blue_tone_90.tone().value(), 90.035, 2);
}

#[test]
fn hct_solver_は十分近い色を返す() {
    for hue in (15..360).step_by(30) {
        for chroma in (0..=100).step_by(10) {
            for tone in (20..=80).step_by(10) {
                let (hue, chroma, tone) = (hue as f64, chroma as f64, tone as f64);
                let color = hct(hue, chroma, tone);
                if chroma > 0.0 {
                    assert!((color.hue().degrees() - hue).abs() <= 4.0);
                }
                assert!(color.chroma().value() >= 0.0);
                assert!(color.chroma().value() <= chroma + 2.5);
                if color.chroma().value() < chroma - 2.5 {
                    let argb = color.to_argb();
                    let on_boundary = [argb.red(), argb.green(), argb.blue()]
                        .iter()
                        .any(|&c| c == 0 || c == 255);
                    assert!(
                        on_boundary,
                        "彩度が届かない色が sRGB の境界にない: {argb:?}"
                    );
                }
                assert!((color.tone().value() - tone).abs() <= 0.5);
            }
        }
    }
}

#[test]
fn hct_の往復で元の色に戻る() {
    for r in (0..296u32).step_by(37) {
        for g in (0..296u32).step_by(37) {
            for b in (0..296u32).step_by(37) {
                let argb = Argb::from_rgb(r.min(255) as u8, g.min(255) as u8, b.min(255) as u8);
                let h = Hct::from_argb(argb);
                assert_eq!(Hct::new(h.hue(), h.chroma(), h.tone()).to_argb(), argb);
            }
        }
    }
}

#[test]
fn contrast_の境界() {
    // Tone は範囲外の入力を 0〜100 に丸めるため、移植元の範囲外入力の例は範囲端の値になる
    assert_close(
        contrast::ratio_of_tones(Tone::new(-10.0), Tone::new(110.0)),
        21.0,
        3,
    );
    assert_eq!(contrast::lighter(Tone::new(90.0), 10.0), None);
    assert_eq!(contrast::lighter(Tone::new(110.0), 2.0), None);
    assert_close(
        contrast::lighter_unsafe(Tone::new(100.0), 2.0).value(),
        100.0,
        3,
    );
    assert_eq!(contrast::darker(Tone::new(10.0), 20.0), None);
    assert_eq!(contrast::darker(Tone::new(-10.0), 2.0), None);
    assert_close(contrast::darker_unsafe(Tone::new(0.0), 2.0).value(), 0.0, 3);
}

#[test]
fn monk_skin_tone_scale_は好まれる() {
    // From https://skintone.google#/get-started
    let colors = [
        0xfff6ede4, 0xfff3e7db, 0xfff7ead0, 0xffeadaba, 0xffd7bd96, 0xffa07e56, 0xff825c43,
        0xff604134, 0xff3a312a, 0xff292420,
    ];
    for color in colors {
        assert!(!dislike::is_disliked(&Hct::from_argb(Argb::new(color))));
    }
}

#[test]
fn 胆汁色は好まれず_修正すると好まれる() {
    let colors = [0xff95884b, 0xff716b40, 0xffb08e00, 0xff4c4308, 0xff464521];
    for color in colors {
        let h = Hct::from_argb(Argb::new(color));
        assert!(dislike::is_disliked(&h));
        assert!(!dislike::is_disliked(&dislike::fix_if_disliked(h)));
    }
}

#[test]
fn tone_67_の色は好まれる() {
    let color = hct(100.0, 50.0, 67.0);
    assert!(!dislike::is_disliked(&color));
    assert_eq!(dislike::fix_if_disliked(color).to_argb(), color.to_argb());
}

#[test]
fn 青の_tonal_palette() {
    let blue = TonalPalette::from_argb(BLUE);
    let cases = [
        (100.0, 0xffffffff),
        (95.0, 0xfff1efff),
        (90.0, 0xffe0e0ff),
        (80.0, 0xffbec2ff),
        (70.0, 0xff9da3ff),
        (60.0, 0xff7c84ff),
        (50.0, 0xff5a64ff),
        (40.0, 0xff343dff),
        (30.0, 0xff0000ef),
        (20.0, 0xff0001ac),
        (10.0, 0xff00006e),
        (0.0, 0xff000000),
    ];
    for (tone, expected) in cases {
        assert_eq!(blue.tone(Tone::new(tone)).to_u32(), expected, "tone {tone}");
    }
}

#[test]
fn key_color_は要求した色相と彩度に近い() {
    let exact = TonalPalette::from_hue_and_chroma(Hue::new(50.0), Chroma::new(60.0)).key_color();
    assert!((exact.hue().degrees() - 50.0).abs() < 10.0);
    assert!((exact.chroma().value() - 60.0).abs() < 0.5);
    assert!(exact.tone().value() > 0.0 && exact.tone().value() < 100.0);

    // 色相 149 の彩度の最大は tone 87.9 での 89.6
    let high = TonalPalette::from_hue_and_chroma(Hue::new(149.0), Chroma::new(200.0)).key_color();
    assert!((high.hue().degrees() - 149.0).abs() < 10.0);
    assert!(high.chroma().value() > 89.0);
    assert!(high.tone().value() > 0.0 && high.tone().value() < 100.0);

    let low = TonalPalette::from_hue_and_chroma(Hue::new(50.0), Chroma::new(3.0)).key_color();
    assert!((low.hue().degrees() - 50.0).abs() < 10.0);
    assert!((low.chroma().value() - 3.0).abs() < 0.5);
    assert!((low.tone().value() - 50.0).abs() < 0.5);
}

#[test]
fn 色温度() {
    // [argb, raw, relative, complement, analogous]
    let cases = [
        (
            BLUE,
            -1.393,
            0.0,
            0xff9d0002,
            [0xff00590c, 0xff00564e, 0xff0000ff, 0xff6700cc, 0xff81009f],
        ),
        (
            RED,
            2.351,
            1.0,
            0xff007bfc,
            [0xfff60082, 0xfffc004c, 0xffff0000, 0xffd95500, 0xffaf7200],
        ),
        (
            GREEN,
            -0.267,
            0.467,
            0xffffd2c9,
            [0xffcee900, 0xff92f500, 0xff00ff00, 0xff00fd6f, 0xff00fab3],
        ),
        (WHITE, -0.5, 0.5, 0xffffffff, [0xffffffff; 5]),
        (BLACK, -0.5, 0.5, 0xff000000, [0xff000000; 5]),
    ];
    for (argb, raw, relative, complement, analogous) in cases {
        let input = Hct::from_argb(argb);
        assert_close(temperature::raw_temperature(&input), raw, 3);
        let cache = TemperatureCache::new(input);
        assert_close(cache.input_relative_temperature(), relative, 3);
        assert_eq!(cache.complement().to_argb().to_u32(), complement);
        let actual: Vec<u32> = cache
            .analogous(5, 12)
            .iter()
            .map(|h| h.to_argb().to_u32())
            .collect();
        assert_eq!(actual, analogous);
    }
}

#[test]
fn harmonize() {
    let cases = [
        (RED, BLUE, 0xfffb0057),
        (RED, GREEN, 0xffd85600),
        (RED, YELLOW, 0xffd85600),
        (BLUE, GREEN, 0xff0047a3),
        (BLUE, RED, 0xff5700dc),
        (BLUE, YELLOW, 0xff0047a3),
        (GREEN, BLUE, 0xff00fc94),
        (GREEN, RED, 0xffb1f000),
        (GREEN, YELLOW, 0xffb1f000),
        (YELLOW, BLUE, 0xffebffba),
        (YELLOW, GREEN, 0xffebffba),
        (YELLOW, RED, 0xfffff6e3),
    ];
    for (design, source, expected) in cases {
        assert_eq!(blend::harmonize(design, source).to_u32(), expected);
    }
}

fn scheme_2021(
    source: Argb,
    variant: Variant,
    dark: bool,
    contrast_level: f64,
) -> Result<DynamicScheme, SchemeError> {
    DynamicScheme::builder(Hct::from_argb(source), variant)
        .dark(dark)
        .contrast_level(ContrastLevel::new(contrast_level))
        .spec_version(SpecVersion::Spec2021)
        .build()
}

#[test]
fn 文字と面の組は最低限のコントラストを満たす() {
    use ColorRole::*;
    let pairs = [
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
    let variants = [
        Variant::Content,
        Variant::Expressive,
        Variant::Fidelity,
        Variant::Monochrome,
        Variant::Neutral,
        Variant::TonalSpot,
        Variant::Vibrant,
    ];
    for source in [RED, YELLOW, GREEN, BLUE] {
        for contrast_level in [-1.0, -0.75, -0.5, -0.25, 0.0, 0.25, 0.5, 0.75, 1.0] {
            for dark in [false, true] {
                for variant in variants {
                    let scheme = scheme_2021(source, variant, dark, contrast_level).unwrap();
                    let minimum = if contrast_level >= 0.0 { 4.5 } else { 3.0 };
                    for (fg, bg) in pairs {
                        let ratio =
                            contrast::ratio_of_tones(scheme.hct(fg).tone(), scheme.hct(bg).tone());
                        assert!(
                            ratio >= minimum,
                            "{variant:?} {source:?} dark={dark} contrast={contrast_level}: {} on {} is {ratio}",
                            fg.name(),
                            bg.name()
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn 固定色の_tone() {
    use ColorRole::*;
    let roles = [
        PrimaryFixed,
        PrimaryFixedDim,
        OnPrimaryFixed,
        OnPrimaryFixedVariant,
        SecondaryFixed,
        SecondaryFixedDim,
        OnSecondaryFixed,
        OnSecondaryFixedVariant,
        TertiaryFixed,
        TertiaryFixedDim,
        OnTertiaryFixed,
        OnTertiaryFixedVariant,
    ];
    let tonal_spot = scheme_2021(RED, Variant::TonalSpot, true, 0.0).unwrap();
    let tones = [
        90.0, 80.0, 10.0, 30.0, 90.0, 80.0, 10.0, 30.0, 90.0, 80.0, 10.0, 30.0,
    ];
    for (role, tone) in roles.iter().zip(tones) {
        assert_close(tonal_spot.hct(*role).tone().value(), tone, 0);
    }
    let monochrome_tones = [
        40.0, 30.0, 100.0, 90.0, 80.0, 70.0, 10.0, 25.0, 40.0, 30.0, 100.0, 90.0,
    ];
    for dark in [false, true] {
        let monochrome = scheme_2021(RED, Variant::Monochrome, dark, 0.0).unwrap();
        for (role, tone) in roles.iter().zip(monochrome_tones) {
            assert_close(monochrome.hct(*role).tone().value(), tone, 0);
        }
    }
    let argbs = [
        0xffffdad4, 0xffffb4a8, 0xff3a0905, 0xff73342a, 0xffffdad4, 0xffe7bdb6, 0xff2c1512,
        0xff5d3f3b, 0xfffbdfa6, 0xffdec48c, 0xff251a00, 0xff564419,
    ];
    for (role, argb) in roles.iter().zip(argbs) {
        assert_eq!(tonal_spot.argb(*role).to_u32(), argb, "{}", role.name());
    }
}

#[test]
fn 極端な彩度は移植元と同じく_nan_を経て黒になる() {
    // 移植元の Hct.from(30, Number.MAX_VALUE, 50).toInt() を実行した結果
    assert_eq!(hct(30.0, f64::MAX, 50.0).to_argb().to_u32(), 0xff000000);
}
