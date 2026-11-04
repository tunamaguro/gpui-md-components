// Ported from material-color-utilities
// (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/dynamiccolor/variant.ts (Variant),
//         typescript/dynamiccolor/color_spec.ts (SpecVersion),
//         typescript/dynamiccolor/dynamic_scheme.ts (Platform)
//         @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2022 Google LLC. Licensed under the Apache License, Version 2.0.
// color_spec.ts: Copyright 2025 Google LLC. Licensed under the Apache License, Version 2.0.

/// 配色の種類。元の色から各パレットの色相と彩度を決める規則
///
/// 説明は移植元の各 `Scheme*` クラスの doc に基づく
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Variant {
    /// 無彩色の配色
    Monochrome,
    /// 無彩色に近い配色
    Neutral,
    /// 彩度が低めから中程度で、第三色の色相が元の色に関連する配色。Android 12 / 13 の既定
    TonalSpot,
    /// 主色のパレットで彩度を最大にする配色
    Vibrant,
    /// 意図的に元の色から離した配色
    Expressive,
    /// 元の色を主色のコンテナに置き、第三色のコンテナに補色を置く配色
    Fidelity,
    /// 元の色を主色のコンテナに置き、第三色に類似色を使う配色
    Content,
    /// 元の色の色相が配色に現れない、遊びのある配色
    Rainbow,
    /// 元の色の色相が配色に現れない、遊びのある配色
    FruitSalad,
    /// 2 つの元の色から作る配色 (Color Match Fidelity)。2026 仕様でだけ使える
    Cmf,
}

/// 配色の仕様の版
///
/// 版によって各色の役割のトーン・彩度の規則が変わる。版の指定に対応しない [`Variant`] では、
/// 対応する最も新しい版に読み替える
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SpecVersion {
    /// 2021 仕様 (MD3 の初版)
    Spec2021,
    /// 2025 仕様
    Spec2025,
    /// 2026 仕様
    Spec2026,
}

impl Default for SpecVersion {
    /// このクレートの既定は 2026 仕様。移植元の `DynamicScheme.DEFAULT_SPEC_VERSION` は 2021 仕様
    fn default() -> Self {
        Self::Spec2026
    }
}

/// 配色を表示する機器の種類
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Platform {
    /// スマートフォンなどの一般的な画面。移植元の既定
    #[default]
    Phone,
    /// 腕時計の画面
    Watch,
}
