// Ported from material-color-utilities
// (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/palettes/tonal_palette.ts (TonalPalette, KeyColor)
//         @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2021 Google LLC. Licensed under the Apache License, Version 2.0.

//! 色相と彩度を固定し、トーンだけを変えた色の集まり

use crate::hct::Hct;
use crate::units::{Argb, Chroma, Hue, Tone};
use crate::utils::math::round_js;

/// 色相と彩度が一定で、トーンだけが異なる色のパレット
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TonalPalette {
    hue: f64,
    chroma: f64,
    key_color: Hct,
}

impl TonalPalette {
    /// ARGB の色相と彩度を持つパレット。キーカラーは `argb` の色になる
    pub fn from_argb(argb: Argb) -> Self {
        Self::from_hct(Hct::from_argb(argb))
    }

    /// `hct` の色相と彩度を持つパレット。キーカラーは `hct` になる
    pub fn from_hct(hct: Hct) -> Self {
        Self {
            hue: hct.hue_raw(),
            chroma: hct.chroma_raw(),
            key_color: hct,
        }
    }

    /// 色相と彩度から作る
    ///
    /// キーカラーは、指定の彩度を出せるトーンのうちトーン 50 に最も近い色になる。
    /// どのトーンでも出せないときは、最も大きな彩度を出せるトーンの色になる
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

    /// パレットの色相
    pub fn hue(&self) -> Hue {
        Hue::new(self.hue)
    }

    /// パレットの彩度。色域の限界ではなく、パレットを作るときに指定した値
    pub fn chroma(&self) -> Chroma {
        Chroma::new(self.chroma)
    }

    /// パレットを代表するキーカラー
    pub fn key_color(&self) -> Hct {
        self.key_color
    }

    /// `tone` の色
    ///
    /// 黄色の色相 (105〜125 度) のトーン 99 は、トーン 98 と 100 の RGB の平均にする
    pub fn tone(&self, tone: Tone) -> Argb {
        Argb::new(self.tone_raw(tone.value()))
    }

    /// `tone` の色を HCT で返す
    pub fn hct(&self, tone: Tone) -> Hct {
        self.hct_raw(tone.value())
    }

    pub(crate) fn hue_raw(&self) -> f64 {
        self.hue
    }

    pub(crate) fn chroma_raw(&self) -> f64 {
        self.chroma
    }

    pub(crate) fn tone_raw(&self, tone: f64) -> u32 {
        if tone == 99.0 && Hct::is_yellow(self.hue) {
            average_argb(self.tone_raw(98.0), self.tone_raw(100.0))
        } else {
            Hct::from_hct_raw(self.hue, self.chroma, tone).argb_raw()
        }
    }

    pub(crate) fn hct_raw(&self, tone: f64) -> Hct {
        Hct::from_int(self.tone_raw(tone))
    }
}

fn average_argb(argb1: u32, argb2: u32) -> u32 {
    let red1 = (argb1 >> 16) & 0xff;
    let green1 = (argb1 >> 8) & 0xff;
    let blue1 = argb1 & 0xff;
    let red2 = (argb2 >> 16) & 0xff;
    let green2 = (argb2 >> 8) & 0xff;
    let blue2 = argb2 & 0xff;
    // 0〜255 の平均なので `u32` に戻しても範囲を外れない
    let red = round_js(f64::from(red1 + red2) / 2.0) as u32;
    let green = round_js(f64::from(green1 + green2) / 2.0) as u32;
    let blue = round_js(f64::from(blue1 + blue2) / 2.0) as u32;
    (255 << 24) | ((red & 255) << 16) | ((green & 255) << 8) | (blue & 255)
}

/// 色相と彩度に対するキーカラーを求める (移植元の `KeyColor.create`)
///
/// 指定の彩度を出せるトーンを、トーン 50 から近い順に二分探索で探す
fn create_key_color(hue: f64, requested_chroma: f64) -> Hct {
    // 移植元は `maxChroma` の結果をトーンごとにキャッシュする。探索は高々 100 トーンなので計算し直す
    let max_chroma = |tone: i64| Hct::from_hct_raw(hue, 200.0, tone as f64).chroma_raw();

    // 平均して最も彩度を出しやすいトーン 50 を起点にする
    let pivot_tone = 50;
    let tone_step_size = 1;
    // 彩度の誤差の許容値
    let epsilon = 0.01;

    // 二分探索の範囲
    let mut lower_tone = 0;
    let mut upper_tone = 100;
    while lower_tone < upper_tone {
        let mid_tone = (lower_tone + upper_tone) / 2;
        let is_ascending = max_chroma(mid_tone) < max_chroma(mid_tone + tone_step_size);
        let sufficient_chroma = max_chroma(mid_tone) >= requested_chroma - epsilon;

        if sufficient_chroma {
            // 範囲の端のうちピボットから遠い方を縮める
            if (lower_tone - pivot_tone).abs() < (upper_tone - pivot_tone).abs() {
                upper_tone = mid_tone;
            } else {
                if lower_tone == mid_tone {
                    return Hct::from_hct_raw(hue, requested_chroma, lower_tone as f64);
                }
                lower_tone = mid_tone;
            }
        } else if is_ascending {
            // 最大彩度の峰に向かって範囲を縮める
            lower_tone = mid_tone + tone_step_size;
        } else {
            // mid_tone が彩度の峰の可能性があるので残す
            upper_tone = mid_tone;
        }
    }

    Hct::from_hct_raw(hue, requested_chroma, lower_tone as f64)
}
