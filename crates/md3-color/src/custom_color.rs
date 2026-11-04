// Ported from material-color-utilities
// (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/utils/theme_utils.ts (customColor),
//         typescript/palettes/core_palette.ts (CorePalette の a1)
//         @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2021 Google LLC. Licensed under the Apache License, Version 2.0.

//! アプリ独自の色 (Material Theme Builder の custom color)

use crate::blend;
use crate::hct::Hct;
use crate::palettes::TonalPalette;
use crate::units::{Argb, Chroma, Tone};

/// アプリ独自の色から作る 4 つの役割の色
///
/// トーンは明暗ごとに固定で、配色のコントラストレベルや種類には追従しない。
/// 前景と背景のトーン差が固定のため、組み合わせのコントラストは色によらず保たれる
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CustomColorGroup {
    /// 色
    pub color: Argb,
    /// 色の上の文字・アイコン
    pub on_color: Argb,
    /// コンテナ
    pub container: Argb,
    /// コンテナの上の文字・アイコン
    pub on_container: Argb,
}

impl CustomColorGroup {
    /// 元の色が `source` の配色で、`color` から作る役割の色
    ///
    /// `harmonize` なら `color` の色相を `source` に近づけてから作る。`is_dark` でダークのトーンを使う
    pub fn new(source: Argb, color: Argb, harmonize: bool, is_dark: bool) -> Self {
        let value = if harmonize {
            blend::harmonize(color, source)
        } else {
            color
        };
        // CorePalette.of(value).a1 は彩度を 48 以上にしたパレット
        let hct = Hct::from_argb(value);
        let palette = TonalPalette::from_hue_and_chroma(
            hct.hue(),
            Chroma::new(hct.chroma().value().max(48.0)),
        );
        let tone = |tone: f64| palette.tone(Tone::new(tone));
        if is_dark {
            Self {
                color: tone(80.0),
                on_color: tone(20.0),
                container: tone(30.0),
                on_container: tone(90.0),
            }
        } else {
            Self {
                color: tone(40.0),
                on_color: tone(100.0),
                container: tone(90.0),
                on_container: tone(10.0),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 明暗ごとに固定のトーンを使う() {
        // theme_utils.ts の customColor はライトで 40 / 100 / 90 / 10、ダークで 80 / 20 / 30 / 90
        let green = Argb::from_rgb(0x00, 0x80, 0x00);
        let hct = Hct::from_argb(green);
        let palette = TonalPalette::from_hue_and_chroma(
            hct.hue(),
            Chroma::new(hct.chroma().value().max(48.0)),
        );
        let tones = |group: CustomColorGroup| {
            [
                group.color,
                group.on_color,
                group.container,
                group.on_container,
            ]
        };
        let expected = |values: [f64; 4]| values.map(|tone| palette.tone(Tone::new(tone)));
        let source = Argb::from_rgb(0x67, 0x50, 0xa4);
        assert_eq!(
            tones(CustomColorGroup::new(source, green, false, false)),
            expected([40.0, 100.0, 90.0, 10.0])
        );
        assert_eq!(
            tones(CustomColorGroup::new(source, green, false, true)),
            expected([80.0, 20.0, 30.0, 90.0])
        );
    }

    #[test]
    fn 調和させると色相が元の色に近づく() {
        let green = Argb::from_rgb(0x00, 0x80, 0x00);
        let source = Argb::from_rgb(0x67, 0x50, 0xa4);
        let hue = |harmonize| {
            Hct::from_argb(CustomColorGroup::new(source, green, harmonize, false).color)
                .hue()
                .degrees()
        };
        let source_hue = Hct::from_argb(source).hue().degrees();
        let distance = |hue: f64| {
            (hue - source_hue)
                .abs()
                .min(360.0 - (hue - source_hue).abs())
        };
        assert!(distance(hue(true)) < distance(hue(false)));
    }
}
