// Ported from material-color-utilities
// (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/blend/blend.ts @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2021 Google LLC. Licensed under the Apache License, Version 2.0.

//! 色の混合

use crate::hct::{Cam16, Hct};
use crate::units::Argb;
use crate::utils::{color, math};

/// `design_color` の色相を `source_color` に近づけて調和させた色
///
/// 回転量は色相の差の半分で、最大 15 度。彩度とトーンは `design_color` のまま保つ
pub fn harmonize(design_color: Argb, source_color: Argb) -> Argb {
    let from_hct = Hct::from_argb(design_color);
    let to_hct = Hct::from_argb(source_color);
    let difference_degrees = math::difference_degrees(from_hct.hue_raw(), to_hct.hue_raw());
    let rotation_degrees = math::min_js(difference_degrees * 0.5, 15.0);
    let output_hue = math::sanitize_degrees_double(
        from_hct.hue_raw()
            + rotation_degrees * math::rotation_direction(from_hct.hue_raw(), to_hct.hue_raw()),
    );
    Hct::from_hct_raw(output_hue, from_hct.chroma_raw(), from_hct.tone_raw()).to_argb()
}

/// `from` の色相を `to` に向けて `amount` (0〜1) だけ CAM16-UCS 上で混ぜた色
///
/// 彩度とトーンは `from` のまま保つ
pub fn hct_hue(from: Argb, to: Argb, amount: f64) -> Argb {
    let ucs = cam16_ucs(from, to, amount);
    let ucs_cam = Cam16::from_argb(ucs);
    let from_cam = Cam16::from_argb(from);
    Hct::from_hct_raw(
        ucs_cam.hue(),
        from_cam.chroma(),
        color::lstar_from_argb(from.to_u32()),
    )
    .to_argb()
}

/// `from` と `to` を CAM16-UCS 上で `amount` (0〜1) の割合で混ぜた色
pub fn cam16_ucs(from: Argb, to: Argb, amount: f64) -> Argb {
    let from_cam = Cam16::from_argb(from);
    let to_cam = Cam16::from_argb(to);
    let from_j = from_cam.jstar();
    let from_a = from_cam.astar();
    let from_b = from_cam.bstar();
    let to_j = to_cam.jstar();
    let to_a = to_cam.astar();
    let to_b = to_cam.bstar();
    let jstar = from_j + (to_j - from_j) * amount;
    let astar = from_a + (to_a - from_a) * amount;
    let bstar = from_b + (to_b - from_b) * amount;
    Cam16::from_ucs(jstar, astar, bstar).to_argb()
}
