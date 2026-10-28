// Ported from material-color-utilities (https://github.com/material-foundation/material-color-utilities)
// Source: swift/Tests/MaterialColorUtilitiesTests/SchemeTests/Scheme{Content,Expressive,Fidelity,
//         Monochrome,Neutral,TonalSpot,Vibrant}Tests.swift @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2021 Google LLC. Licensed under the Apache License, Version 2.0.

//! 7 種の配色が移植元と同じ ARGB を返すことの確認。

use md3_color::{Argb, ColorRole, ContrastLevel, DynamicScheme, Hct, Variant};

fn scheme(seed: u32, variant: Variant, is_dark: bool, contrast_level: f64) -> DynamicScheme {
    DynamicScheme::new(
        Hct::from_argb(Argb::new(seed)),
        variant,
        is_dark,
        ContrastLevel::new(contrast_level),
    )
}

#[track_caller]
fn assert_argb(s: &DynamicScheme, role: ColorRole, expected: u32) {
    let actual = s.argb(role);
    assert_eq!(
        actual,
        Argb::new(expected),
        "{}: {:#010x} != {expected:#010x}",
        role.name(),
        actual.to_u32()
    );
}

#[track_caller]
fn assert_tone(s: &DynamicScheme, role: ColorRole, expected: f64, accuracy: f64) {
    let actual = s.hct(role).tone().value();
    assert!(
        (actual - expected).abs() <= accuracy,
        "{}: {actual} != {expected}",
        role.name()
    );
}

#[test]
fn content_key_colors() {
    let s = scheme(0xff_0000ff, Variant::Content, false, 0.0);
    assert_argb(&s, ColorRole::PrimaryPaletteKeyColor, 0xff080cff);
    assert_argb(&s, ColorRole::SecondaryPaletteKeyColor, 0xff656dd3);
    assert_argb(&s, ColorRole::TertiaryPaletteKeyColor, 0xff81009f);
    assert_argb(&s, ColorRole::NeutralPaletteKeyColor, 0xff767684);
    assert_argb(&s, ColorRole::NeutralVariantPaletteKeyColor, 0xff757589);
}

#[test]
fn content_light_theme_min_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Content, false, -1.0);
    assert_argb(&s, ColorRole::Primary, 0xff5660ff);
}

#[test]
fn content_light_theme_standard_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Content, false, 0.0);
    assert_argb(&s, ColorRole::Primary, 0xff0001bb);
}

#[test]
fn content_light_theme_max_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Content, false, 1.0);
    assert_argb(&s, ColorRole::Primary, 0xff00019f);
}

#[test]
fn content_light_theme_min_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Content, false, -1.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xffd5d6ff);
}

#[test]
fn content_light_theme_standard_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Content, false, 0.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xff0000ff);
}

#[test]
fn content_light_theme_max_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Content, false, 1.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xff0000f6);
}

#[test]
fn content_light_theme_min_contrast_tertiary_container() {
    let s = scheme(0xff_0000ff, Variant::Content, false, -1.0);
    assert_argb(&s, ColorRole::TertiaryContainer, 0xfffac9ff);
}

#[test]
fn content_light_theme_standard_contrast_tertiary_container() {
    let s = scheme(0xff_0000ff, Variant::Content, false, 0.0);
    assert_argb(&s, ColorRole::TertiaryContainer, 0xff81009f);
}

#[test]
fn content_light_theme_max_contrast_tertiary_container() {
    let s = scheme(0xff_0000ff, Variant::Content, false, 1.0);
    assert_argb(&s, ColorRole::TertiaryContainer, 0xff7d009a);
}

#[test]
fn content_light_theme_min_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Content, false, -1.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xff5e68ff);
}

#[test]
fn content_light_theme_standard_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Content, false, 0.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xffb3b7ff);
}

#[test]
fn content_light_theme_max_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Content, false, 1.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xffffffff);
}

#[test]
fn content_light_theme_min_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Content, false, -1.0);
    assert_argb(&s, ColorRole::Surface, 0xfffbf8ff);
}

#[test]
fn content_light_theme_standard_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Content, false, 0.0);
    assert_argb(&s, ColorRole::Surface, 0xfffbf8ff);
}

#[test]
fn content_light_theme_max_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Content, false, 1.0);
    assert_argb(&s, ColorRole::Surface, 0xfffbf8ff);
}

#[test]
fn content_dark_theme_min_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Content, true, -1.0);
    assert_argb(&s, ColorRole::Primary, 0xff7c84ff);
}

#[test]
fn content_dark_theme_standard_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Content, true, 0.0);
    assert_argb(&s, ColorRole::Primary, 0xffbec2ff);
}

#[test]
fn content_dark_theme_max_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Content, true, 1.0);
    assert_argb(&s, ColorRole::Primary, 0xfff0eeff);
}

#[test]
fn content_dark_theme_min_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Content, true, -1.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xff0001c9);
}

#[test]
fn content_dark_theme_standard_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Content, true, 0.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xff0000ff);
}

#[test]
fn content_dark_theme_max_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Content, true, 1.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xffbabdff);
}

#[test]
fn content_dark_theme_min_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Content, true, -1.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xff6b75ff);
}

#[test]
fn content_dark_theme_standard_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Content, true, 0.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xffb3b7ff);
}

#[test]
fn content_dark_theme_max_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Content, true, 1.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xff00003d);
}

#[test]
fn content_dark_theme_min_contrast_on_tertiary_container() {
    let s = scheme(0xff_0000ff, Variant::Content, true, -1.0);
    assert_argb(&s, ColorRole::OnTertiaryContainer, 0xffc254de);
}

#[test]
fn content_dark_theme_standard_contrast_on_tertiary_container() {
    let s = scheme(0xff_0000ff, Variant::Content, true, 0.0);
    assert_argb(&s, ColorRole::OnTertiaryContainer, 0xfff09fff);
}

#[test]
fn content_dark_theme_max_contrast_on_tertiary_container() {
    let s = scheme(0xff_0000ff, Variant::Content, true, 1.0);
    assert_argb(&s, ColorRole::OnTertiaryContainer, 0xff1a0022);
}

#[test]
fn content_dark_theme_min_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Content, true, -1.0);
    assert_argb(&s, ColorRole::Surface, 0xff12121d);
}

#[test]
fn content_dark_theme_standard_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Content, true, 0.0);
    assert_argb(&s, ColorRole::Surface, 0xff12121d);
}

#[test]
fn content_dark_theme_max_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Content, true, 1.0);
    assert_argb(&s, ColorRole::Surface, 0xff12121d);
}

#[test]
fn content_light_theme_min_contrast_objectionabe_tertiary_container_lightens() {
    let s = scheme(0xff_850096, Variant::Content, false, -1.0);
    assert_argb(&s, ColorRole::TertiaryContainer, 0xffffccd7);
}

#[test]
fn content_light_theme_standard_contrast_objectionabe_tertiary_container_lightens() {
    let s = scheme(0xff_850096, Variant::Content, false, 0.0);
    assert_argb(&s, ColorRole::TertiaryContainer, 0xff980249);
}

#[test]
fn content_light_theme_max_contrast_objectionabe_tertiary_container_darkens() {
    let s = scheme(0xff_850096, Variant::Content, false, 1.0);
    assert_argb(&s, ColorRole::TertiaryContainer, 0xff930046);
}

#[test]
fn expressive_key_colors() {
    let s = scheme(0xff_0000ff, Variant::Expressive, false, 0.0);
    assert_argb(&s, ColorRole::PrimaryPaletteKeyColor, 0xff35855f);
    assert_argb(&s, ColorRole::SecondaryPaletteKeyColor, 0xff8c6d8c);
    assert_argb(&s, ColorRole::TertiaryPaletteKeyColor, 0xff806ea1);
    assert_argb(&s, ColorRole::NeutralPaletteKeyColor, 0xff79757f);
    assert_argb(&s, ColorRole::NeutralVariantPaletteKeyColor, 0xff7a7585);
}

#[test]
fn expressive_light_theme_min_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Expressive, false, -1.0);
    assert_argb(&s, ColorRole::Primary, 0xff32835d);
}

#[test]
fn expressive_light_theme_standard_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Expressive, false, 0.0);
    assert_argb(&s, ColorRole::Primary, 0xff146c48);
}

#[test]
fn expressive_light_theme_max_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Expressive, false, 1.0);
    assert_argb(&s, ColorRole::Primary, 0xff00341f);
}

#[test]
fn expressive_light_theme_min_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Expressive, false, -1.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xff99eabd);
}

#[test]
fn expressive_light_theme_standard_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Expressive, false, 0.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xffa2f4c6);
}

#[test]
fn expressive_light_theme_max_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Expressive, false, 1.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xff005436);
}

#[test]
fn expressive_light_theme_min_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Expressive, false, -1.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xff388862);
}

#[test]
fn expressive_light_theme_standard_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Expressive, false, 0.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xff005234);
}

#[test]
fn expressive_light_theme_max_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Expressive, false, 1.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xffffffff);
}

#[test]
fn expressive_light_theme_min_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Expressive, false, -1.0);
    assert_argb(&s, ColorRole::Surface, 0xfffdf7ff);
}

#[test]
fn expressive_light_theme_standard_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Expressive, false, 0.0);
    assert_argb(&s, ColorRole::Surface, 0xfffdf7ff);
}

#[test]
fn expressive_light_theme_max_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Expressive, false, 1.0);
    assert_argb(&s, ColorRole::Surface, 0xfffdf7ff);
}

#[test]
fn expressive_dark_theme_min_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Expressive, true, -1.0);
    assert_argb(&s, ColorRole::Primary, 0xff51a078);
}

#[test]
fn expressive_dark_theme_standard_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Expressive, true, 0.0);
    assert_argb(&s, ColorRole::Primary, 0xff87d7ab);
}

#[test]
fn expressive_dark_theme_max_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Expressive, true, 1.0);
    assert_argb(&s, ColorRole::Primary, 0xffbbffd7);
}

#[test]
fn expressive_dark_theme_min_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Expressive, true, -1.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xff00432a);
}

#[test]
fn expressive_dark_theme_standard_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Expressive, true, 0.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xff005234);
}

#[test]
fn expressive_dark_theme_max_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Expressive, true, 1.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xff83d3a8);
}

#[test]
fn expressive_dark_theme_min_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Expressive, true, -1.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xff43936c);
}

#[test]
fn expressive_dark_theme_standard_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Expressive, true, 0.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xffa2f4c6);
}

#[test]
fn expressive_dark_theme_max_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Expressive, true, 1.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xff000e06);
}

#[test]
fn expressive_dark_theme_min_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Expressive, true, -1.0);
    assert_argb(&s, ColorRole::Surface, 0xff14121a);
}

#[test]
fn expressive_dark_theme_standard_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Expressive, true, 0.0);
    assert_argb(&s, ColorRole::Surface, 0xff14121a);
}

#[test]
fn expressive_dark_theme_max_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Expressive, true, 1.0);
    assert_argb(&s, ColorRole::Surface, 0xff14121a);
}

#[test]
fn fidelity_key_colors() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, false, 0.0);
    assert_argb(&s, ColorRole::PrimaryPaletteKeyColor, 0xff080cff);
    assert_argb(&s, ColorRole::SecondaryPaletteKeyColor, 0xff656dd3);
    assert_argb(&s, ColorRole::TertiaryPaletteKeyColor, 0xff9d0002);
    assert_argb(&s, ColorRole::NeutralPaletteKeyColor, 0xff767684);
    assert_argb(&s, ColorRole::NeutralVariantPaletteKeyColor, 0xff757589);
}

#[test]
fn fidelity_light_theme_min_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, false, -1.0);
    assert_argb(&s, ColorRole::Primary, 0xff5660ff);
}

#[test]
fn fidelity_light_theme_standard_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, false, 0.0);
    assert_argb(&s, ColorRole::Primary, 0xff0001bb);
}

#[test]
fn fidelity_light_theme_max_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, false, 1.0);
    assert_argb(&s, ColorRole::Primary, 0xff00019f);
}

#[test]
fn fidelity_light_theme_min_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, false, -1.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xffd5d6ff);
}

#[test]
fn fidelity_light_theme_standard_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, false, 0.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xff0000ff);
}

#[test]
fn fidelity_light_theme_max_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, false, 1.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xff0000f6);
}

#[test]
fn fidelity_light_theme_min_contrast_tertiary_container() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, false, -1.0);
    assert_argb(&s, ColorRole::TertiaryContainer, 0xffffcdc6);
}

#[test]
fn fidelity_light_theme_standard_contrast_tertiary_container() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, false, 0.0);
    assert_argb(&s, ColorRole::TertiaryContainer, 0xff9d0002);
}

#[test]
fn fidelity_light_theme_max_contrast_tertiary_container() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, false, 1.0);
    assert_argb(&s, ColorRole::TertiaryContainer, 0xff980002);
}

#[test]
fn fidelity_light_theme_min_contrast_objectionable_tertiary_container_lightens() {
    let s = scheme(0xff_850096, Variant::Fidelity, false, -1.0);
    assert_argb(&s, ColorRole::TertiaryContainer, 0xffebd982);
}

#[test]
fn fidelity_light_theme_standard_contrast_objectionable_tertiary_container_lightens() {
    let s = scheme(0xff_850096, Variant::Fidelity, false, 0.0);
    assert_argb(&s, ColorRole::TertiaryContainer, 0xffbcac5a);
}

#[test]
fn fidelity_light_theme_max_contrast_objectionable_tertiary_container_darkens() {
    let s = scheme(0xff_850096, Variant::Fidelity, false, 1.0);
    assert_argb(&s, ColorRole::TertiaryContainer, 0xff544900);
}

#[test]
fn fidelity_light_theme_min_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, false, -1.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xff5e68ff);
}

#[test]
fn fidelity_light_theme_standard_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, false, 0.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xffb3b7ff);
}

#[test]
fn fidelity_light_theme_max_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, false, 1.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xffffffff);
}

#[test]
fn fidelity_light_theme_min_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, false, -1.0);
    assert_argb(&s, ColorRole::Surface, 0xfffbf8ff);
}

#[test]
fn fidelity_light_theme_standard_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, false, 0.0);
    assert_argb(&s, ColorRole::Surface, 0xfffbf8ff);
}

#[test]
fn fidelity_light_theme_max_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, false, 1.0);
    assert_argb(&s, ColorRole::Surface, 0xfffbf8ff);
}

#[test]
fn fidelity_dark_theme_min_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, true, -1.0);
    assert_argb(&s, ColorRole::Primary, 0xff7c84ff);
}

#[test]
fn fidelity_dark_theme_standard_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, true, 0.0);
    assert_argb(&s, ColorRole::Primary, 0xffbec2ff);
}

#[test]
fn fidelity_dark_theme_max_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, true, 1.0);
    assert_argb(&s, ColorRole::Primary, 0xfff0eeff);
}

#[test]
fn fidelity_dark_theme_min_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, true, -1.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xff0001c9);
}

#[test]
fn fidelity_dark_theme_standard_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, true, 0.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xff0000ff);
}

#[test]
fn fidelity_dark_theme_max_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, true, 1.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xffbabdff);
}

#[test]
fn fidelity_dark_theme_min_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, true, -1.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xff6b75ff);
}

#[test]
fn fidelity_dark_theme_standard_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, true, 0.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xffb3b7ff);
}

#[test]
fn fidelity_dark_theme_max_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, true, 1.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xff00003d);
}

#[test]
fn fidelity_dark_theme_min_contrast_on_tertiary_container() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, true, -1.0);
    assert_argb(&s, ColorRole::OnTertiaryContainer, 0xffef4635);
}

#[test]
fn fidelity_dark_theme_standard_contrast_on_tertiary_container() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, true, 0.0);
    assert_argb(&s, ColorRole::OnTertiaryContainer, 0xffffa598);
}

#[test]
fn fidelity_dark_theme_max_contrast_on_tertiary_container() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, true, 1.0);
    assert_argb(&s, ColorRole::OnTertiaryContainer, 0xff220000);
}

#[test]
fn fidelity_dark_theme_min_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, true, -1.0);
    assert_argb(&s, ColorRole::Surface, 0xff12121d);
}

#[test]
fn fidelity_dark_theme_standard_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, true, 0.0);
    assert_argb(&s, ColorRole::Surface, 0xff12121d);
}

#[test]
fn fidelity_dark_theme_max_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Fidelity, true, 1.0);
    assert_argb(&s, ColorRole::Surface, 0xff12121d);
}

#[test]
fn monochrome_key_colors() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, false, 0.0);
    assert_argb(&s, ColorRole::PrimaryPaletteKeyColor, 0xff777777);
    assert_argb(&s, ColorRole::SecondaryPaletteKeyColor, 0xff777777);
    assert_argb(&s, ColorRole::TertiaryPaletteKeyColor, 0xff777777);
    assert_argb(&s, ColorRole::NeutralPaletteKeyColor, 0xff777777);
    assert_argb(&s, ColorRole::NeutralVariantPaletteKeyColor, 0xff777777);
}

#[test]
fn monochrome_light_theme_min_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, false, -1.0);
    assert_argb(&s, ColorRole::Primary, 0xff747474);
}

#[test]
fn monochrome_light_theme_standard_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, false, 0.0);
    assert_argb(&s, ColorRole::Primary, 0xff000000);
}

#[test]
fn monochrome_light_theme_max_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, false, 1.0);
    assert_argb(&s, ColorRole::Primary, 0xff000000);
}

#[test]
fn monochrome_light_theme_min_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, false, -1.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xffd9d9d9);
}

#[test]
fn monochrome_light_theme_standard_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, false, 0.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xff3b3b3b);
}

#[test]
fn monochrome_light_theme_max_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, false, 1.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xff3b3b3b);
}

#[test]
fn monochrome_light_theme_min_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, false, -1.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xff7a7a7a);
}

#[test]
fn monochrome_light_theme_standard_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, false, 0.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xffffffff);
}

#[test]
fn monochrome_light_theme_max_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, false, 1.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xffffffff);
}

#[test]
fn monochrome_light_theme_min_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, false, -1.0);
    assert_argb(&s, ColorRole::Surface, 0xfff9f9f9);
}

#[test]
fn monochrome_light_theme_standard_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, false, 0.0);
    assert_argb(&s, ColorRole::Surface, 0xfff9f9f9);
}

#[test]
fn monochrome_light_theme_max_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, false, 1.0);
    assert_argb(&s, ColorRole::Surface, 0xfff9f9f9);
}

#[test]
fn monochrome_dark_theme_min_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, true, -1.0);
    assert_argb(&s, ColorRole::Primary, 0xff919191);
}

#[test]
fn monochrome_dark_theme_standard_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, true, 0.0);
    assert_argb(&s, ColorRole::Primary, 0xffffffff);
}

#[test]
fn monochrome_dark_theme_max_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, true, 1.0);
    assert_argb(&s, ColorRole::Primary, 0xffffffff);
}

#[test]
fn monochrome_dark_theme_min_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, true, -1.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xff3a3a3a);
}

#[test]
fn monochrome_dark_theme_standard_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, true, 0.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xffd4d4d4);
}

#[test]
fn monochrome_dark_theme_max_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, true, 1.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xffd4d4d4);
}

#[test]
fn monochrome_dark_theme_min_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, true, -1.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xff848484);
}

#[test]
fn monochrome_dark_theme_standard_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, true, 0.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xff000000);
}

#[test]
fn monochrome_dark_theme_max_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, true, 1.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xff000000);
}

#[test]
fn monochrome_dark_theme_min_contrast_on_tertiary_container() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, true, -1.0);
    assert_argb(&s, ColorRole::OnTertiaryContainer, 0xff848484);
}

#[test]
fn monochrome_dark_theme_standard_contrast_on_tertiary_container() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, true, 0.0);
    assert_argb(&s, ColorRole::OnTertiaryContainer, 0xff000000);
}

#[test]
fn monochrome_dark_theme_max_contrast_on_tertiary_container() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, true, 1.0);
    assert_argb(&s, ColorRole::OnTertiaryContainer, 0xff000000);
}

#[test]
fn monochrome_dark_theme_min_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, true, -1.0);
    assert_argb(&s, ColorRole::Surface, 0xff131313);
}

#[test]
fn monochrome_dark_theme_standard_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, true, 0.0);
    assert_argb(&s, ColorRole::Surface, 0xff131313);
}

#[test]
fn monochrome_dark_theme_max_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, true, 1.0);
    assert_argb(&s, ColorRole::Surface, 0xff131313);
}

#[test]
fn monochrome_dark_theme_monochrome_spec() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, true, 0.0);
    assert_tone(&s, ColorRole::Primary, 100.0, 0.3);
    assert_tone(&s, ColorRole::OnPrimary, 10.0, 0.3);
    assert_tone(&s, ColorRole::PrimaryContainer, 85.0, 0.3);
    assert_tone(&s, ColorRole::OnPrimaryContainer, 0.0, 0.3);
    assert_tone(&s, ColorRole::Secondary, 80.0, 0.3);
    assert_tone(&s, ColorRole::OnSecondary, 10.0, 0.3);
    assert_tone(&s, ColorRole::SecondaryContainer, 30.0, 0.3);
    assert_tone(&s, ColorRole::OnSecondaryContainer, 90.0, 0.3);
    assert_tone(&s, ColorRole::Tertiary, 90.0, 0.3);
    assert_tone(&s, ColorRole::OnTertiary, 10.0, 0.3);
    assert_tone(&s, ColorRole::TertiaryContainer, 60.0, 0.3);
    assert_tone(&s, ColorRole::OnTertiaryContainer, 0.0, 0.3);
}

#[test]
fn monochrome_light_theme_monochrome_spec() {
    let s = scheme(0xff_0000ff, Variant::Monochrome, false, 0.0);
    assert_tone(&s, ColorRole::Primary, 0.0, 0.3);
    assert_tone(&s, ColorRole::OnPrimary, 90.0, 0.3);
    assert_tone(&s, ColorRole::PrimaryContainer, 25.0, 0.3);
    assert_tone(&s, ColorRole::OnPrimaryContainer, 100.0, 0.3);
    assert_tone(&s, ColorRole::Secondary, 40.0, 0.3);
    assert_tone(&s, ColorRole::OnSecondary, 100.0, 0.3);
    assert_tone(&s, ColorRole::SecondaryContainer, 85.0, 0.3);
    assert_tone(&s, ColorRole::OnSecondaryContainer, 10.0, 0.3);
    assert_tone(&s, ColorRole::Tertiary, 25.0, 0.3);
    assert_tone(&s, ColorRole::OnTertiary, 90.0, 0.3);
    assert_tone(&s, ColorRole::TertiaryContainer, 49.0, 0.3);
    assert_tone(&s, ColorRole::OnTertiaryContainer, 100.0, 0.3);
}

#[test]
fn neutral_key_colors() {
    let s = scheme(0xff_0000ff, Variant::Neutral, false, 0.0);
    assert_argb(&s, ColorRole::PrimaryPaletteKeyColor, 0xff767685);
    assert_argb(&s, ColorRole::SecondaryPaletteKeyColor, 0xff777680);
    assert_argb(&s, ColorRole::TertiaryPaletteKeyColor, 0xff75758b);
    assert_argb(&s, ColorRole::NeutralPaletteKeyColor, 0xff787678);
    assert_argb(&s, ColorRole::NeutralVariantPaletteKeyColor, 0xff787678);
}

#[test]
fn neutral_light_theme_min_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Neutral, false, -1.0);
    assert_argb(&s, ColorRole::Primary, 0xff737383);
}

#[test]
fn neutral_light_theme_standard_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Neutral, false, 0.0);
    assert_argb(&s, ColorRole::Primary, 0xff5d5d6c);
}

#[test]
fn neutral_light_theme_max_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Neutral, false, 1.0);
    assert_argb(&s, ColorRole::Primary, 0xff2b2b38);
}

#[test]
fn neutral_light_theme_min_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Neutral, false, -1.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xffd9d7e9);
}

#[test]
fn neutral_light_theme_standard_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Neutral, false, 0.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xffe2e1f3);
}

#[test]
fn neutral_light_theme_max_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Neutral, false, 1.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xff484856);
}

#[test]
fn neutral_light_theme_min_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Neutral, false, -1.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xff797888);
}

#[test]
fn neutral_light_theme_standard_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Neutral, false, 0.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xff454654);
}

#[test]
fn neutral_light_theme_max_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Neutral, false, 1.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xffffffff);
}

#[test]
fn neutral_light_theme_min_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Neutral, false, -1.0);
    assert_argb(&s, ColorRole::Surface, 0xfffcf8fa);
}

#[test]
fn neutral_light_theme_standard_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Neutral, false, 0.0);
    assert_argb(&s, ColorRole::Surface, 0xfffcf8fa);
}

#[test]
fn neutral_light_theme_max_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Neutral, false, 1.0);
    assert_argb(&s, ColorRole::Surface, 0xfffcf8fa);
}

#[test]
fn neutral_dark_theme_min_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Neutral, true, -1.0);
    assert_argb(&s, ColorRole::Primary, 0xff908f9f);
}

#[test]
fn neutral_dark_theme_standard_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Neutral, true, 0.0);
    assert_argb(&s, ColorRole::Primary, 0xffc6c5d6);
}

#[test]
fn neutral_dark_theme_max_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Neutral, true, 1.0);
    assert_argb(&s, ColorRole::Primary, 0xfff0eeff);
}

#[test]
fn neutral_dark_theme_min_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Neutral, true, -1.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xff393947);
}

#[test]
fn neutral_dark_theme_standard_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Neutral, true, 0.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xff454654);
}

#[test]
fn neutral_dark_theme_max_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Neutral, true, 1.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xffc2c1d2);
}

#[test]
fn neutral_dark_theme_min_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Neutral, true, -1.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xff838393);
}

#[test]
fn neutral_dark_theme_standard_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Neutral, true, 0.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xffe2e1f3);
}

#[test]
fn neutral_dark_theme_max_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Neutral, true, 1.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xff090a16);
}

#[test]
fn neutral_dark_theme_min_contrast_on_tertiary_container() {
    let s = scheme(0xff_0000ff, Variant::Neutral, true, -1.0);
    assert_argb(&s, ColorRole::OnTertiaryContainer, 0xff828299);
}

#[test]
fn neutral_dark_theme_standard_contrast_on_tertiary_container() {
    let s = scheme(0xff_0000ff, Variant::Neutral, true, 0.0);
    assert_argb(&s, ColorRole::OnTertiaryContainer, 0xffe1e0f9);
}

#[test]
fn neutral_dark_theme_max_contrast_on_tertiary_container() {
    let s = scheme(0xff_0000ff, Variant::Neutral, true, 1.0);
    assert_argb(&s, ColorRole::OnTertiaryContainer, 0xff080a1b);
}

#[test]
fn neutral_dark_theme_min_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Neutral, true, -1.0);
    assert_argb(&s, ColorRole::Surface, 0xff131315);
}

#[test]
fn neutral_dark_theme_standard_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Neutral, true, 0.0);
    assert_argb(&s, ColorRole::Surface, 0xff131315);
}

#[test]
fn neutral_dark_theme_max_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Neutral, true, 1.0);
    assert_argb(&s, ColorRole::Surface, 0xff131315);
}

#[test]
fn tonal_spot_blue_light_scheme() {
    let s = scheme(0xff_0000ff, Variant::TonalSpot, false, 0.0);
    assert_argb(&s, ColorRole::Primary, 0xff555992);
}

#[test]
fn tonal_spot_blue_dark_scheme() {
    let s = scheme(0xff_0000ff, Variant::TonalSpot, true, 0.0);
    assert_argb(&s, ColorRole::Primary, 0xffbec2ff);
}

#[test]
fn tonal_spot_3rd_party_light_scheme() {
    let s = scheme(0xff_6750a4, Variant::TonalSpot, false, 0.0);
    assert_argb(&s, ColorRole::Primary, 0xff65558f);
    assert_argb(&s, ColorRole::Secondary, 0xff625b71);
    assert_argb(&s, ColorRole::Tertiary, 0xff7e5260);
    assert_argb(&s, ColorRole::Surface, 0xfffdf7ff);
    assert_argb(&s, ColorRole::OnSurface, 0xff1d1b20);
}

#[test]
fn tonal_spot_3rd_party_dark_scheme() {
    let s = scheme(0xff_6750a4, Variant::TonalSpot, true, 0.0);
    assert_argb(&s, ColorRole::Primary, 0xffcfbdfe);
    assert_argb(&s, ColorRole::Secondary, 0xffcbc2db);
    assert_argb(&s, ColorRole::Tertiary, 0xffefb8c8);
    assert_argb(&s, ColorRole::Surface, 0xff141218);
    assert_argb(&s, ColorRole::OnSurface, 0xffe6e0e9);
}

#[test]
fn tonal_spot_light_scheme_from_high_chroma_color() {
    let s = scheme(0xff_fa2bec, Variant::TonalSpot, false, 0.0);
    assert_argb(&s, ColorRole::Primary, 0xff814c77);
    assert_argb(&s, ColorRole::OnPrimary, 0xffffffff);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xffffd7f3);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xff67355e);
    assert_argb(&s, ColorRole::Secondary, 0xff6e5868);
    assert_argb(&s, ColorRole::OnSecondary, 0xffffffff);
    assert_argb(&s, ColorRole::SecondaryContainer, 0xfff8daee);
    assert_argb(&s, ColorRole::OnSecondaryContainer, 0xff554050);
    assert_argb(&s, ColorRole::Tertiary, 0xff815343);
    assert_argb(&s, ColorRole::OnTertiary, 0xffffffff);
    assert_argb(&s, ColorRole::TertiaryContainer, 0xffffdbd0);
    assert_argb(&s, ColorRole::OnTertiaryContainer, 0xff663c2d);
    assert_argb(&s, ColorRole::Error, 0xffba1a1a);
    assert_argb(&s, ColorRole::OnError, 0xffffffff);
    assert_argb(&s, ColorRole::ErrorContainer, 0xffffdad6);
    assert_argb(&s, ColorRole::OnErrorContainer, 0xff93000a);
    assert_argb(&s, ColorRole::Background, 0xfffff7f9);
    assert_argb(&s, ColorRole::OnBackground, 0xff201a1e);
    assert_argb(&s, ColorRole::Surface, 0xfffff7f9);
    assert_argb(&s, ColorRole::OnSurface, 0xff201a1e);
    assert_argb(&s, ColorRole::SurfaceVariant, 0xffeedee7);
    assert_argb(&s, ColorRole::OnSurfaceVariant, 0xff4e444b);
    assert_argb(&s, ColorRole::Outline, 0xff80747b);
    assert_argb(&s, ColorRole::OutlineVariant, 0xffd2c2cb);
    assert_argb(&s, ColorRole::Shadow, 0xff000000);
    assert_argb(&s, ColorRole::Scrim, 0xff000000);
    assert_argb(&s, ColorRole::InverseSurface, 0xff362e33);
    assert_argb(&s, ColorRole::InverseOnSurface, 0xfffaedf3);
    assert_argb(&s, ColorRole::InversePrimary, 0xfff3b2e4);
}

#[test]
fn tonal_spot_dark_scheme_from_high_chroma_color() {
    let s = scheme(0xff_fa2bec, Variant::TonalSpot, true, 0.0);
    assert_argb(&s, ColorRole::Primary, 0xfff3b2e4);
    assert_argb(&s, ColorRole::OnPrimary, 0xff4d1f47);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xff67355e);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xffffd7f3);
    assert_argb(&s, ColorRole::Secondary, 0xffdbbed1);
    assert_argb(&s, ColorRole::OnSecondary, 0xff3e2a39);
    assert_argb(&s, ColorRole::SecondaryContainer, 0xff554050);
    assert_argb(&s, ColorRole::OnSecondaryContainer, 0xfff8daee);
    assert_argb(&s, ColorRole::Tertiary, 0xfff5b9a5);
    assert_argb(&s, ColorRole::OnTertiary, 0xff4c2619);
    assert_argb(&s, ColorRole::TertiaryContainer, 0xff663c2d);
    assert_argb(&s, ColorRole::OnTertiaryContainer, 0xffffdbd0);
    assert_argb(&s, ColorRole::Error, 0xffffb4ab);
    assert_argb(&s, ColorRole::OnError, 0xff690005);
    assert_argb(&s, ColorRole::ErrorContainer, 0xff93000a);
    assert_argb(&s, ColorRole::OnErrorContainer, 0xffffdad6);
    assert_argb(&s, ColorRole::Background, 0xff181216);
    assert_argb(&s, ColorRole::OnBackground, 0xffecdfe5);
    assert_argb(&s, ColorRole::Surface, 0xff181216);
    assert_argb(&s, ColorRole::OnSurface, 0xffecdfe5);
    assert_argb(&s, ColorRole::SurfaceVariant, 0xff4e444b);
    assert_argb(&s, ColorRole::OnSurfaceVariant, 0xffd2c2cb);
    assert_argb(&s, ColorRole::Outline, 0xff9a8d95);
    assert_argb(&s, ColorRole::OutlineVariant, 0xff4e444b);
    assert_argb(&s, ColorRole::Shadow, 0xff000000);
    assert_argb(&s, ColorRole::Scrim, 0xff000000);
    assert_argb(&s, ColorRole::InverseSurface, 0xffecdfe5);
    assert_argb(&s, ColorRole::InverseOnSurface, 0xff362e33);
    assert_argb(&s, ColorRole::InversePrimary, 0xff814c77);
}

#[test]
fn tonal_spot_light_content_scheme_from_high_chroma_color() {
    let s = scheme(0xff_fa2bec, Variant::Content, false, 0.0);
    assert_argb(&s, ColorRole::Primary, 0xffa7009e);
    assert_argb(&s, ColorRole::OnPrimary, 0xffffffff);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xffd100c6);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xfffffbff);
    assert_argb(&s, ColorRole::Secondary, 0xff9e2d93);
    assert_argb(&s, ColorRole::OnSecondary, 0xffffffff);
    assert_argb(&s, ColorRole::SecondaryContainer, 0xffff83ec);
    assert_argb(&s, ColorRole::OnSecondaryContainer, 0xff7c0276);
    assert_argb(&s, ColorRole::Tertiary, 0xffb51830);
    assert_argb(&s, ColorRole::OnTertiary, 0xffffffff);
    assert_argb(&s, ColorRole::TertiaryContainer, 0xffd83546);
    assert_argb(&s, ColorRole::OnTertiaryContainer, 0xfffffbff);
    assert_argb(&s, ColorRole::Error, 0xffba1a1a);
    assert_argb(&s, ColorRole::OnError, 0xffffffff);
    assert_argb(&s, ColorRole::ErrorContainer, 0xffffdad6);
    assert_argb(&s, ColorRole::OnErrorContainer, 0xff93000a);
    assert_argb(&s, ColorRole::Background, 0xfffff7f9);
    assert_argb(&s, ColorRole::OnBackground, 0xff251722);
    assert_argb(&s, ColorRole::Surface, 0xfffff7f9);
    assert_argb(&s, ColorRole::OnSurface, 0xff251722);
    assert_argb(&s, ColorRole::SurfaceVariant, 0xfff9daee);
    assert_argb(&s, ColorRole::OnSurfaceVariant, 0xff564050);
    assert_argb(&s, ColorRole::Outline, 0xff897081);
    assert_argb(&s, ColorRole::OutlineVariant, 0xffdcbed2);
    assert_argb(&s, ColorRole::Shadow, 0xff000000);
    assert_argb(&s, ColorRole::Scrim, 0xff000000);
    assert_argb(&s, ColorRole::InverseSurface, 0xff3b2c37);
    assert_argb(&s, ColorRole::InverseOnSurface, 0xffffebf6);
    assert_argb(&s, ColorRole::InversePrimary, 0xffffabee);
}

#[test]
fn tonal_spot_dark_content_scheme_from_high_chroma_color() {
    let s = scheme(0xff_fa2bec, Variant::Content, true, 0.0);
    assert_argb(&s, ColorRole::Primary, 0xffffabee);
    assert_argb(&s, ColorRole::OnPrimary, 0xff5c0057);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xfffa2bec);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xff51004c);
    assert_argb(&s, ColorRole::Secondary, 0xffffabee);
    assert_argb(&s, ColorRole::OnSecondary, 0xff5c0057);
    assert_argb(&s, ColorRole::SecondaryContainer, 0xff840f7c);
    assert_argb(&s, ColorRole::OnSecondaryContainer, 0xffff93ec);
    assert_argb(&s, ColorRole::Tertiary, 0xffffb3b2);
    assert_argb(&s, ColorRole::OnTertiary, 0xff680014);
    assert_argb(&s, ColorRole::TertiaryContainer, 0xffff525f);
    assert_argb(&s, ColorRole::OnTertiaryContainer, 0xff5b0010);
    assert_argb(&s, ColorRole::Error, 0xffffb4ab);
    assert_argb(&s, ColorRole::OnError, 0xff690005);
    assert_argb(&s, ColorRole::ErrorContainer, 0xff93000a);
    assert_argb(&s, ColorRole::OnErrorContainer, 0xffffdad6);
    assert_argb(&s, ColorRole::Background, 0xff1c0f19);
    assert_argb(&s, ColorRole::OnBackground, 0xfff4dceb);
    assert_argb(&s, ColorRole::Surface, 0xff1c0f19);
    assert_argb(&s, ColorRole::OnSurface, 0xfff4dceb);
    assert_argb(&s, ColorRole::SurfaceVariant, 0xff564050);
    assert_argb(&s, ColorRole::OnSurfaceVariant, 0xffdcbed2);
    assert_argb(&s, ColorRole::Outline, 0xffa4899b);
    assert_argb(&s, ColorRole::OutlineVariant, 0xff564050);
    assert_argb(&s, ColorRole::Shadow, 0xff000000);
    assert_argb(&s, ColorRole::Scrim, 0xff000000);
    assert_argb(&s, ColorRole::InverseSurface, 0xfff4dceb);
    assert_argb(&s, ColorRole::InverseOnSurface, 0xff3b2c37);
    assert_argb(&s, ColorRole::InversePrimary, 0xffab00a2);
}

#[test]
fn vibrant_key_colors() {
    let s = scheme(0xff_0000ff, Variant::Vibrant, false, 0.0);
    assert_argb(&s, ColorRole::PrimaryPaletteKeyColor, 0xff080cff);
    assert_argb(&s, ColorRole::SecondaryPaletteKeyColor, 0xff7b7296);
    assert_argb(&s, ColorRole::TertiaryPaletteKeyColor, 0xff886c9d);
    assert_argb(&s, ColorRole::NeutralPaletteKeyColor, 0xff777682);
    assert_argb(&s, ColorRole::NeutralVariantPaletteKeyColor, 0xff767685);
}

#[test]
fn vibrant_light_theme_min_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Vibrant, false, -1.0);
    assert_argb(&s, ColorRole::Primary, 0xff5660ff);
}

#[test]
fn vibrant_light_theme_standard_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Vibrant, false, 0.0);
    assert_argb(&s, ColorRole::Primary, 0xff343dff);
}

#[test]
fn vibrant_light_theme_max_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Vibrant, false, 1.0);
    assert_argb(&s, ColorRole::Primary, 0xff00019f);
}

#[test]
fn vibrant_light_theme_min_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Vibrant, false, -1.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xffd5d6ff);
}

#[test]
fn vibrant_light_theme_standard_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Vibrant, false, 0.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xffe0e0ff);
}

#[test]
fn vibrant_light_theme_max_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Vibrant, false, 1.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xff0000f6);
}

#[test]
fn vibrant_light_theme_min_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Vibrant, false, -1.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xff5e68ff);
}

#[test]
fn vibrant_light_theme_standard_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Vibrant, false, 0.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xff0000ef);
}

#[test]
fn vibrant_light_theme_max_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Vibrant, false, 1.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xffffffff);
}

#[test]
fn vibrant_light_theme_min_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Vibrant, false, -1.0);
    assert_argb(&s, ColorRole::Surface, 0xfffbf8ff);
}

#[test]
fn vibrant_light_theme_standard_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Vibrant, false, 0.0);
    assert_argb(&s, ColorRole::Surface, 0xfffbf8ff);
}

#[test]
fn vibrant_light_theme_max_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Vibrant, false, 1.0);
    assert_argb(&s, ColorRole::Surface, 0xfffbf8ff);
}

#[test]
fn vibrant_dark_theme_min_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Vibrant, true, -1.0);
    assert_argb(&s, ColorRole::Primary, 0xff7c84ff);
}

#[test]
fn vibrant_dark_theme_standard_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Vibrant, true, 0.0);
    assert_argb(&s, ColorRole::Primary, 0xffbec2ff);
}

#[test]
fn vibrant_dark_theme_max_contrast_primary() {
    let s = scheme(0xff_0000ff, Variant::Vibrant, true, 1.0);
    assert_argb(&s, ColorRole::Primary, 0xfff0eeff);
}

#[test]
fn vibrant_dark_theme_min_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Vibrant, true, -1.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xff0001c9);
}

#[test]
fn vibrant_dark_theme_standard_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Vibrant, true, 0.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xff0000ef);
}

#[test]
fn vibrant_dark_theme_max_contrast_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Vibrant, true, 1.0);
    assert_argb(&s, ColorRole::PrimaryContainer, 0xffbabdff);
}

#[test]
fn vibrant_dark_theme_min_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Vibrant, true, -1.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xff6b75ff);
}

#[test]
fn vibrant_dark_theme_standard_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Vibrant, true, 0.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xffe0e0ff);
}

#[test]
fn vibrant_dark_theme_max_contrast_on_primary_container() {
    let s = scheme(0xff_0000ff, Variant::Vibrant, true, 1.0);
    assert_argb(&s, ColorRole::OnPrimaryContainer, 0xff00003d);
}

#[test]
fn vibrant_dark_theme_min_contrast_on_tertiary_container() {
    let s = scheme(0xff_0000ff, Variant::Vibrant, true, -1.0);
    assert_argb(&s, ColorRole::OnTertiaryContainer, 0xff9679ab);
}

#[test]
fn vibrant_dark_theme_standard_contrast_on_tertiary_container() {
    let s = scheme(0xff_0000ff, Variant::Vibrant, true, 0.0);
    assert_argb(&s, ColorRole::OnTertiaryContainer, 0xfff2daff);
}

#[test]
fn vibrant_dark_theme_max_contrast_on_tertiary_container() {
    let s = scheme(0xff_0000ff, Variant::Vibrant, true, 1.0);
    assert_argb(&s, ColorRole::OnTertiaryContainer, 0xff16002a);
}

#[test]
fn vibrant_dark_theme_min_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Vibrant, true, -1.0);
    assert_argb(&s, ColorRole::Surface, 0xff12131c);
}

#[test]
fn vibrant_dark_theme_standard_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Vibrant, true, 0.0);
    assert_argb(&s, ColorRole::Surface, 0xff12131c);
}

#[test]
fn vibrant_dark_theme_max_contrast_surface() {
    let s = scheme(0xff_0000ff, Variant::Vibrant, true, 1.0);
    assert_argb(&s, ColorRole::Surface, 0xff12131c);
}

#[test]
fn role_index_matches_position_in_all() {
    for (i, role) in ColorRole::ALL.iter().enumerate() {
        assert_eq!(role.index(), i, "{}", role.name());
    }
}
