// Ported from material-color-utilities
// (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/dislike/dislike_analyzer.ts @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2023 Google LLC. Licensed under the Apache License, Version 2.0.

//! 好まれにくい色の判定と補正
//!
//! 暗い黄緑 (枯れ葉や嘔吐物を連想させる色) は好まれにくいという調査結果に基づく

use crate::hct::Hct;
use crate::utils::math::round_js;

/// 好まれにくい色なら `true`
///
/// 四捨五入した色相が 90〜111、彩度が 16 より大きく、トーンが 65 未満の色が該当する
pub fn is_disliked(hct: &Hct) -> bool {
    let hue_passes = round_js(hct.hue_raw()) >= 90.0 && round_js(hct.hue_raw()) <= 111.0;
    let chroma_passes = round_js(hct.chroma_raw()) > 16.0;
    let tone_passes = round_js(hct.tone_raw()) < 65.0;
    hue_passes && chroma_passes && tone_passes
}

/// 好まれにくい色ならトーンを 70 に上げた色、そうでなければ `hct` をそのまま返す
pub fn fix_if_disliked(hct: Hct) -> Hct {
    if is_disliked(&hct) {
        Hct::from_hct_raw(hct.hue_raw(), hct.chroma_raw(), 70.0)
    } else {
        hct
    }
}
