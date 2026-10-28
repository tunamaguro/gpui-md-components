// Ported from material-color-utilities (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/hct/hct.ts @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2021 Google LLC. Licensed under the Apache License, Version 2.0.

//! HCT（Hue / Chroma / Tone）色空間と、その土台の CAM16。

mod cam16;
pub(crate) mod solver;
mod viewing_conditions;

pub use cam16::Cam16;
pub use viewing_conditions::ViewingConditions;

use crate::units::{Argb, Chroma, Hue, Tone};
use crate::utils::color;

/// HCT 色空間の色。
///
/// 色相と彩度は CAM16、トーンは L*a*b* の L* を使う。
/// sRGB で表せない組み合わせを指定した場合は、色相とトーンを保ったまま
/// 彩度が最大の色になる。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hct {
    argb: Argb,
    hue: Hue,
    chroma: Chroma,
    tone: Tone,
}

impl Hct {
    /// 色相・彩度・トーンから作る。
    pub fn new(hue: Hue, chroma: Chroma, tone: Tone) -> Self {
        Self::from_hct(hue.degrees(), chroma.value(), tone.value())
    }

    /// 正規化前の値から作る。移植元の `Hct.from` と同じ。
    pub(crate) fn from_hct(hue: f64, chroma: f64, tone: f64) -> Self {
        Self::from_argb(Argb::new(solver::solve_to_int(hue, chroma, tone)))
    }

    /// ARGB から作る。
    pub fn from_argb(argb: Argb) -> Self {
        let cam = Cam16::from_argb(argb);
        Self {
            argb,
            hue: Hue::raw(cam.hue()),
            chroma: Chroma::raw(cam.chroma()),
            tone: Tone::raw(color::lstar_from_argb(argb.to_u32())),
        }
    }

    /// ARGB。
    pub fn to_argb(&self) -> Argb {
        self.argb
    }

    /// 色相。
    pub fn hue(&self) -> Hue {
        self.hue
    }

    /// 彩度。
    pub fn chroma(&self) -> Chroma {
        self.chroma
    }

    /// トーン。
    pub fn tone(&self) -> Tone {
        self.tone
    }

    /// 色相だけを変えた色。
    pub fn with_hue(&self, hue: Hue) -> Self {
        Self::from_hct(hue.degrees(), self.chroma.value(), self.tone.value())
    }

    /// 彩度だけを変えた色。
    pub fn with_chroma(&self, chroma: Chroma) -> Self {
        Self::from_hct(self.hue.degrees(), chroma.value(), self.tone.value())
    }

    /// トーンだけを変えた色。
    pub fn with_tone(&self, tone: Tone) -> Self {
        Self::from_hct(self.hue.degrees(), self.chroma.value(), tone.value())
    }

    /// 青系の色相か（250 以上 270 未満）。
    pub fn is_blue(hue: Hue) -> bool {
        (250.0..270.0).contains(&hue.degrees())
    }

    /// 黄系の色相か（105 以上 125 未満）。
    pub fn is_yellow(hue: Hue) -> bool {
        (105.0..125.0).contains(&hue.degrees())
    }

    /// シアン系の色相か（170 以上 207 未満）。
    pub fn is_cyan(hue: Hue) -> bool {
        (170.0..207.0).contains(&hue.degrees())
    }

    /// 指定の観察条件で見たときの見え方を、既定の観察条件で再現した色。
    pub fn in_viewing_conditions(&self, vc: &ViewingConditions) -> Self {
        // 1. Use CAM16 to find XYZ coordinates of color in specified VC.
        let cam = Cam16::from_argb(self.argb);
        let viewed_in_vc = cam.xyz_in_viewing_conditions(vc);

        // 2. Create CAM16 of those XYZ coordinates in default VC.
        let recast_in_vc = Cam16::from_xyz_in_viewing_conditions(
            viewed_in_vc[0],
            viewed_in_vc[1],
            viewed_in_vc[2],
            ViewingConditions::standard(),
        );

        // 3. Create HCT from:
        // - CAM16 using default VC with XYZ coordinates in specified VC.
        // - L* converted from Y in XYZ coordinates in specified VC.
        Self::from_hct(
            recast_in_vc.hue(),
            recast_in_vc.chroma(),
            color::lstar_from_y(viewed_in_vc[1]),
        )
    }
}

impl From<Argb> for Hct {
    fn from(argb: Argb) -> Self {
        Self::from_argb(argb)
    }
}

impl From<Hct> for Argb {
    fn from(hct: Hct) -> Self {
        hct.argb
    }
}

#[cfg(test)]
mod tests {
    // Ported from typescript/hct/hct_test.ts and hct_round_trip_test.ts

    use super::*;

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 0.005, "{actual} != {expected}");
    }

    #[test]
    fn argb_to_hct_green() {
        let hct = Hct::from_argb(Argb::new(0xff00_ff00));
        assert_close(hct.hue().degrees(), 142.139);
        assert_close(hct.chroma().value(), 108.410);
        assert_close(hct.tone().value(), 87.737);
    }

    #[test]
    fn argb_to_hct_blue() {
        let hct = Hct::from_argb(Argb::new(0xff00_00ff));
        assert_close(hct.hue().degrees(), 282.788);
        assert_close(hct.chroma().value(), 87.230);
        assert_close(hct.tone().value(), 32.302);
    }

    #[test]
    fn argb_to_hct_blue_tone_90() {
        let hct = Hct::from_hct(282.788, 87.230, 90.0);
        assert_close(hct.hue().degrees(), 282.239);
        assert_close(hct.chroma().value(), 19.144);
        assert_close(hct.tone().value(), 90.035);
    }

    fn color_is_on_boundary(argb: Argb) -> bool {
        [argb.red(), argb.green(), argb.blue()]
            .iter()
            .any(|&c| c == 0 || c == 255)
    }

    #[test]
    fn solver_returns_sufficiently_close_color() {
        for hue in (15..360).step_by(30) {
            for chroma in (0..=100).step_by(10) {
                for tone in (20..=80).step_by(10) {
                    let (hue, chroma, tone) = (f64::from(hue), f64::from(chroma), f64::from(tone));
                    let hct = Hct::from_hct(hue, chroma, tone);
                    if chroma > 0.0 {
                        assert!((hct.hue().degrees() - hue).abs() <= 4.0);
                    }
                    assert!(hct.chroma().value() >= 0.0);
                    assert!(hct.chroma().value() <= chroma + 2.5);
                    if hct.chroma().value() < chroma - 2.5 {
                        assert!(color_is_on_boundary(hct.to_argb()));
                    }
                    assert!((hct.tone().value() - tone).abs() <= 0.5);
                }
            }
        }
    }

    #[test]
    fn round_trip_preserves_original_color() {
        for r in (0..296u32).step_by(37) {
            for g in (0..296u32).step_by(37) {
                for b in (0..296u32).step_by(37) {
                    let channel = |v: u32| u8::try_from(v.min(255)).unwrap_or(u8::MAX);
                    let argb = Argb::from_rgb(channel(r), channel(g), channel(b));
                    let hct = Hct::from_argb(argb);
                    let reconstructed = Hct::from_hct(
                        hct.hue().degrees(),
                        hct.chroma().value(),
                        hct.tone().value(),
                    );
                    assert_eq!(reconstructed.to_argb(), argb);
                }
            }
        }
    }
}
