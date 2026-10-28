// Ported from material-color-utilities (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/palettes/tonal_palette.ts @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2021 Google LLC. Licensed under the Apache License, Version 2.0.

use crate::hct::Hct;
use crate::units::{Argb, Chroma, Hue, Tone};
use crate::utils::math::js_round;

/// 色相と彩度を固定し、トーンだけを変えた色の集まり。
///
/// 移植元と違いキャッシュは持たず、[`TonalPalette::tone`] のたびに計算する。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TonalPalette {
    // 移植元と同じく、正規化前の色相（例: 380）をそのまま保持する。
    hue: f64,
    chroma: f64,
    key_color: Hct,
}

impl TonalPalette {
    /// ARGB の色相と彩度を持つパレット。
    pub fn from_argb(argb: Argb) -> Self {
        Self::from_hct(Hct::from_argb(argb))
    }

    /// HCT の色相と彩度を持つパレット。キー色は `hct` そのもの。
    pub fn from_hct(hct: Hct) -> Self {
        Self {
            hue: hct.hue().degrees(),
            chroma: hct.chroma().value(),
            key_color: hct,
        }
    }

    /// 色相と彩度からパレットを作る。キー色は彩度を出せるトーンから選ぶ。
    pub fn from_hue_and_chroma(hue: Hue, chroma: Chroma) -> Self {
        Self::from_hue_and_chroma_raw(hue.degrees(), chroma.value())
    }

    pub(crate) fn from_hue_and_chroma_raw(hue: f64, chroma: f64) -> Self {
        Self {
            hue,
            chroma,
            key_color: create_key_color(hue, chroma),
        }
    }

    /// 色相。
    pub fn hue(&self) -> Hue {
        Hue::new(self.hue)
    }

    /// 彩度。
    pub fn chroma(&self) -> Chroma {
        Chroma::new(self.chroma)
    }

    /// パレットを代表するキー色。
    pub fn key_color(&self) -> Hct {
        self.key_color
    }

    /// 指定トーンの色。
    pub fn tone(&self, tone: Tone) -> Argb {
        self.tone_raw(tone.value())
    }

    /// 指定トーンの色を HCT で返す。
    pub fn hct(&self, tone: Tone) -> Hct {
        Hct::from_argb(self.tone(tone))
    }

    pub(crate) fn hue_raw(&self) -> f64 {
        self.hue
    }

    pub(crate) fn chroma_raw(&self) -> f64 {
        self.chroma
    }

    pub(crate) fn hct_raw(&self, tone: f64) -> Hct {
        Hct::from_argb(self.tone_raw(tone))
    }

    pub(crate) fn tone_raw(&self, tone: f64) -> Argb {
        if tone == 99.0 && Hct::is_yellow(Hue::raw(self.hue)) {
            average_argb(self.tone_raw(98.0), self.tone_raw(100.0))
        } else {
            Hct::from_hct(self.hue, self.chroma, tone).to_argb()
        }
    }
}

fn average_argb(argb1: Argb, argb2: Argb) -> Argb {
    let average = |a: u8, b: u8| {
        let rounded = js_round((f64::from(a) + f64::from(b)) / 2.0);
        // 0〜255 の平均なので必ず範囲内に収まる。
        rounded as u8
    };
    Argb::from_rgb(
        average(argb1.red(), argb2.red()),
        average(argb1.green(), argb2.green()),
        average(argb1.blue(), argb2.blue()),
    )
}

const MAX_CHROMA_VALUE: f64 = 200.0;

fn max_chroma(hue: f64, tone: i32) -> f64 {
    Hct::from_hct(hue, MAX_CHROMA_VALUE, f64::from(tone))
        .chroma()
        .value()
}

/// 色相と彩度に合うキー色を二分探索で求める。移植元の `KeyColor.create`。
fn create_key_color(hue: f64, requested_chroma: f64) -> Hct {
    const PIVOT_TONE: i32 = 50;
    const TONE_STEP_SIZE: i32 = 1;
    const EPSILON: f64 = 0.01;

    let mut lower_tone: i32 = 0;
    let mut upper_tone: i32 = 100;
    while lower_tone < upper_tone {
        let mid_tone = (lower_tone + upper_tone).div_euclid(2);
        let mid_chroma = max_chroma(hue, mid_tone);
        let is_ascending = mid_chroma < max_chroma(hue, mid_tone + TONE_STEP_SIZE);
        let sufficient_chroma = mid_chroma >= requested_chroma - EPSILON;

        if sufficient_chroma {
            if (lower_tone - PIVOT_TONE).abs() < (upper_tone - PIVOT_TONE).abs() {
                upper_tone = mid_tone;
            } else {
                if lower_tone == mid_tone {
                    return Hct::from_hct(hue, requested_chroma, f64::from(lower_tone));
                }
                lower_tone = mid_tone;
            }
        } else if is_ascending {
            lower_tone = mid_tone + TONE_STEP_SIZE;
        } else {
            upper_tone = mid_tone;
        }
    }
    Hct::from_hct(hue, requested_chroma, f64::from(lower_tone))
}

#[cfg(test)]
mod tests {
    // Ported from typescript/palettes/palettes_test.ts

    use super::*;

    fn assert_tones(palette: &TonalPalette, expected: &[(u8, u32)]) {
        for &(tone, argb) in expected {
            assert_eq!(
                palette.tone(Tone::new(f64::from(tone))),
                Argb::new(argb),
                "tone {tone}"
            );
        }
    }

    #[test]
    fn of_blue() {
        let blue = TonalPalette::from_argb(Argb::new(0xff0000ff));
        assert_tones(
            &blue,
            &[
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
            ],
        );
    }

    #[test]
    fn key_color_with_exact_chroma() {
        let result = TonalPalette::from_hue_and_chroma_raw(50.0, 60.0).key_color();
        assert!((result.hue().degrees() - 50.0).abs() < 10.0);
        assert!((result.chroma().value() - 60.0).abs() < 0.5);
        assert!(result.tone().value() > 0.0);
        assert!(result.tone().value() < 100.0);
    }

    #[test]
    fn key_color_with_unusually_high_chroma() {
        let result = TonalPalette::from_hue_and_chroma_raw(149.0, 200.0).key_color();
        assert!((result.hue().degrees() - 149.0).abs() < 10.0);
        assert!(result.chroma().value() > 89.0);
        assert!(result.tone().value() > 0.0);
        assert!(result.tone().value() < 100.0);
    }

    #[test]
    fn key_color_with_unusually_low_chroma() {
        let result = TonalPalette::from_hue_and_chroma_raw(50.0, 3.0).key_color();
        assert!((result.hue().degrees() - 50.0).abs() < 10.0);
        assert!((result.chroma().value() - 3.0).abs() < 0.5);
        assert!((result.tone().value() - 50.0).abs() < 0.5);
    }
}
