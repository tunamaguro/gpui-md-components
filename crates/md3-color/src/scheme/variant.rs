// Ported from material-color-utilities (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/dynamiccolor/variant.ts @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2021 Google LLC. Licensed under the Apache License, Version 2.0.

/// 配色の種類。MD3 ベースライン（2021 仕様）の 7 種。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Variant {
    /// 無彩色のみ。
    Monochrome,
    /// ほぼ無彩色。
    Neutral,
    /// 落ち着いた配色。MD3 の既定。
    TonalSpot,
    /// 高彩度の配色。
    Vibrant,
    /// 元の色から色相をずらした、表現豊かな配色。
    Expressive,
    /// 元の色をなるべく忠実に保つ配色。
    Fidelity,
    /// コンテンツ（画像など）の色に合わせた配色。
    Content,
}

impl Variant {
    /// すべての種類。
    pub const ALL: [Variant; 7] = [
        Variant::Monochrome,
        Variant::Neutral,
        Variant::TonalSpot,
        Variant::Vibrant,
        Variant::Expressive,
        Variant::Fidelity,
        Variant::Content,
    ];
}
