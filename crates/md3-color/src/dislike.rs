// Ported from material-color-utilities (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/dislike/dislike_analyzer.ts @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2021 Google LLC. Licensed under the Apache License, Version 2.0.

//! 一般に好まれにくい色（暗い黄緑系）の判定と補正。

use crate::hct::Hct;
use crate::utils::math::js_round;

/// 好まれにくい色か。色相 90〜111、彩度 16 超、トーン 65 未満が該当する。
pub fn is_disliked(hct: &Hct) -> bool {
    let hue = js_round(hct.hue().degrees());
    let hue_passes = (90.0..=111.0).contains(&hue);
    let chroma_passes = js_round(hct.chroma().value()) > 16.0;
    let tone_passes = js_round(hct.tone().value()) < 65.0;
    hue_passes && chroma_passes && tone_passes
}

/// 好まれにくい色ならトーンを 70 に上げた色を、そうでなければそのまま返す。
pub fn fix_if_disliked(hct: Hct) -> Hct {
    if is_disliked(&hct) {
        Hct::from_hct(hct.hue().degrees(), hct.chroma().value(), 70.0)
    } else {
        hct
    }
}

#[cfg(test)]
mod tests {
    // Ported from typescript/dislike/dislike_analyzer_test.ts

    use super::*;
    use crate::units::Argb;

    const UNLIKABLE: [u32; 5] = [0xff95884b, 0xff716b40, 0xffb08e00, 0xff4c4308, 0xff464521];

    #[test]
    fn likes_monk_skin_tone_scale_colors() {
        let colors = [
            0xfff6ede4, 0xfff3e7db, 0xfff7ead0, 0xffeadaba, 0xffd7bd96, 0xffa07e56, 0xff825c43,
            0xff604134, 0xff3a312a, 0xff292420,
        ];
        for color in colors {
            assert!(!is_disliked(&Hct::from_argb(Argb::new(color))));
        }
    }

    #[test]
    fn dislikes_bile_colors() {
        for color in UNLIKABLE {
            assert!(is_disliked(&Hct::from_argb(Argb::new(color))));
        }
    }

    #[test]
    fn makes_bile_colors_likable() {
        for color in UNLIKABLE {
            let hct = Hct::from_argb(Argb::new(color));
            assert!(is_disliked(&hct));
            let likable = fix_if_disliked(hct);
            assert!(!is_disliked(&likable));
        }
    }

    #[test]
    fn likes_tone_67_colors() {
        let color = Hct::from_hct(100.0, 50.0, 67.0);
        assert!(!is_disliked(&color));
        assert_eq!(fix_if_disliked(color).to_argb(), color.to_argb());
    }
}
