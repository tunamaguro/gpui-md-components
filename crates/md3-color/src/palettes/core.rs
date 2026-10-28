// Ported from material-color-utilities (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/palettes/core_palette.ts @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2021 Google LLC. Licensed under the Apache License, Version 2.0.

use super::TonalPalette;
use crate::hct::Hct;
use crate::units::Argb;

/// 1 色から作る、主要な 6 つのトーンパレットの組。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CorePalette {
    /// primary（移植元の `a1`）。
    pub primary: TonalPalette,
    /// secondary（移植元の `a2`）。
    pub secondary: TonalPalette,
    /// tertiary（移植元の `a3`）。
    pub tertiary: TonalPalette,
    /// neutral（移植元の `n1`）。
    pub neutral: TonalPalette,
    /// neutral variant（移植元の `n2`）。
    pub neutral_variant: TonalPalette,
    /// error。
    pub error: TonalPalette,
}

impl CorePalette {
    /// 色から作る。primary は彩度 48 以上を保つ。
    pub fn of(argb: Argb) -> Self {
        Self::new(argb, false)
    }

    /// コンテンツ（画像など）の色から作る。元の彩度をなるべく保つ。
    pub fn content_of(argb: Argb) -> Self {
        Self::new(argb, true)
    }

    fn new(argb: Argb, is_content: bool) -> Self {
        let hct = Hct::from_argb(argb);
        let hue = hct.hue().degrees();
        let chroma = hct.chroma().value();
        let palette = TonalPalette::from_hue_and_chroma_raw;
        let error = palette(25.0, 84.0);
        if is_content {
            Self {
                primary: palette(hue, chroma),
                secondary: palette(hue, chroma / 3.0),
                tertiary: palette(hue + 60.0, chroma / 2.0),
                neutral: palette(hue, f64::min(chroma / 12.0, 4.0)),
                neutral_variant: palette(hue, f64::min(chroma / 6.0, 8.0)),
                error,
            }
        } else {
            Self {
                primary: palette(hue, f64::max(48.0, chroma)),
                secondary: palette(hue, 16.0),
                tertiary: palette(hue + 60.0, 24.0),
                neutral: palette(hue, 4.0),
                neutral_variant: palette(hue, 8.0),
                error,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // Ported from typescript/palettes/palettes_test.ts

    use super::*;
    use crate::units::Tone;

    fn assert_tones(palette: &TonalPalette, expected: &[(u8, u32)]) {
        for &(tone, argb) in expected {
            assert_eq!(
                palette.tone(Tone::new(f64::from(tone))),
                Argb::new(argb),
                "tone {tone}"
            );
        }
    }

    const BLUE_PRIMARY: [(u8, u32); 12] = [
        (100, 0xffffffff),
        (95, 0xfff1efff),
        (90, 0xffe0e0ff),
        (80, 0xffbec2ff),
        (70, 0xff9da3ff),
        (60, 0xff7c84ff),
        (50, 0xff5a64ff),
        (40, 0xff343dff),
        (30, 0xff0000ef),
        (20, 0xff0001ac),
        (10, 0xff00006e),
        (0, 0xff000000),
    ];

    #[test]
    fn of_blue() {
        let core = CorePalette::of(Argb::new(0xff0000ff));
        assert_tones(&core.primary, &BLUE_PRIMARY);
        assert_tones(
            &core.secondary,
            &[
                (100, 0xffffffff),
                (95, 0xfff1efff),
                (90, 0xffe1e0f9),
                (80, 0xffc5c4dd),
                (70, 0xffa9a9c1),
                (60, 0xff8f8fa6),
                (50, 0xff75758b),
                (40, 0xff5c5d72),
                (30, 0xff444559),
                (20, 0xff2e2f42),
                (10, 0xff191a2c),
                (0, 0xff000000),
            ],
        );
    }

    #[test]
    fn content_of_blue() {
        let core = CorePalette::content_of(Argb::new(0xff0000ff));
        assert_tones(&core.primary, &BLUE_PRIMARY);
        assert_tones(
            &core.secondary,
            &[
                (100, 0xffffffff),
                (95, 0xfff1efff),
                (90, 0xffe0e0ff),
                (80, 0xffc1c3f4),
                (70, 0xffa5a7d7),
                (60, 0xff8b8dbb),
                (50, 0xff7173a0),
                (40, 0xff585b86),
                (30, 0xff40436d),
                (20, 0xff2a2d55),
                (10, 0xff14173f),
                (0, 0xff000000),
            ],
        );
    }
}
