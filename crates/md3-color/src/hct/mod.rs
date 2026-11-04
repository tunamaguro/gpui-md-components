// Ported from material-color-utilities
// (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/hct/hct.ts @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2021 Google LLC. Licensed under the Apache License, Version 2.0.

//! HCT 色空間 (色相・彩度・トーン) と、その基礎になる CAM16

mod cam16;
mod solver;
mod viewing_conditions;

pub use cam16::Cam16;
pub use viewing_conditions::ViewingConditions;

use crate::units::{Argb, Chroma, Hue, Tone};
use crate::utils::color;

/// HCT 色空間の色
///
/// 色相と彩度は CAM16、トーンは L*a*b* の L* に基づく。sRGB の色域内の色だけを表し、
/// 生成時に 8 bit の RGB へ丸めた色の色相・彩度・トーンを保持する
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hct {
    argb: u32,
    hue: f64,
    chroma: f64,
    tone: f64,
}

impl Hct {
    /// 色相・彩度・トーンから作る
    ///
    /// 指定の彩度が色域を超えるときは、色相とトーンを保ったまま彩度を下げる。
    /// 結果の色相・彩度は指定値と一致するとは限らない
    pub fn new(hue: Hue, chroma: Chroma, tone: Tone) -> Self {
        Self::from_hct_raw(hue.degrees(), chroma.value(), tone.value())
    }

    /// ARGB から作る。色相・彩度・トーンの計算では不透明度を無視し、[`Hct::to_argb`] は `argb` をそのまま返す
    pub fn from_argb(argb: Argb) -> Self {
        Self::from_int(argb.to_u32())
    }

    pub(crate) fn from_hct_raw(hue: f64, chroma: f64, tone: f64) -> Self {
        Self::from_int(solver::solve_to_int(hue, chroma, tone))
    }

    pub(crate) fn from_int(argb: u32) -> Self {
        let cam = Cam16::from_int(argb);
        Self {
            argb,
            hue: cam.hue(),
            chroma: cam.chroma(),
            tone: color::lstar_from_argb(argb),
        }
    }

    /// 色相
    pub fn hue(&self) -> Hue {
        Hue::new(self.hue)
    }

    /// 彩度
    pub fn chroma(&self) -> Chroma {
        Chroma::new(self.chroma)
    }

    /// トーン
    pub fn tone(&self) -> Tone {
        Tone::new(self.tone)
    }

    /// ARGB。[`Hct::from_argb`] で作った色は元の不透明度を保ち、それ以外は不透明
    pub fn to_argb(&self) -> Argb {
        Argb::new(self.argb)
    }

    /// 色相だけを変えた色。彩度とトーンは元の値を指定して作り直すため、色域によって変わることがある
    pub fn with_hue(&self, hue: Hue) -> Self {
        Self::from_hct_raw(hue.degrees(), self.chroma, self.tone)
    }

    /// 彩度だけを変えた色。色相とトーンは元の値を指定して作り直す
    pub fn with_chroma(&self, chroma: Chroma) -> Self {
        Self::from_hct_raw(self.hue, chroma.value(), self.tone)
    }

    /// トーンだけを変えた色。色相と彩度は元の値を指定して作り直す
    pub fn with_tone(&self, tone: Tone) -> Self {
        Self::from_hct_raw(self.hue, self.chroma, tone.value())
    }

    /// `viewing_conditions` で見たときの見えを、既定の観測条件で再現する色
    ///
    /// 例えば暗い環境で見る色を、既定の観測条件の画面で確認するときに使う
    pub fn in_viewing_conditions(&self, viewing_conditions: &ViewingConditions) -> Self {
        let cam = Cam16::from_int(self.argb);
        let viewed_in_vc = cam.xyz_in_viewing_conditions(viewing_conditions);
        let recast_in_vc = Cam16::from_xyz_in_viewing_conditions(
            viewed_in_vc[0],
            viewed_in_vc[1],
            viewed_in_vc[2],
            ViewingConditions::standard_ref(),
        );
        Self::from_hct_raw(
            recast_in_vc.hue(),
            recast_in_vc.chroma(),
            color::lstar_from_y(viewed_in_vc[1]),
        )
    }

    pub(crate) fn hue_raw(&self) -> f64 {
        self.hue
    }

    pub(crate) fn chroma_raw(&self) -> f64 {
        self.chroma
    }

    pub(crate) fn tone_raw(&self) -> f64 {
        self.tone
    }

    pub(crate) fn argb_raw(&self) -> u32 {
        self.argb
    }

    pub(crate) fn is_blue(hue: f64) -> bool {
        (250.0..270.0).contains(&hue)
    }

    pub(crate) fn is_yellow(hue: f64) -> bool {
        (105.0..125.0).contains(&hue)
    }

    pub(crate) fn is_cyan(hue: f64) -> bool {
        (170.0..207.0).contains(&hue)
    }
}
