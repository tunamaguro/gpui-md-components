// Ported from material-color-utilities
// (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/dynamiccolor/tone_delta_pair.ts @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2023 Google LLC. Licensed under the Apache License, Version 2.0.

use super::ColorRole;

/// トーンの差を保つ 2 つの役割の組の、向きの指定
///
/// 移植元の `TonePolarity` のうち、どの役割も使わない `farther` は省いている
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TonePolarity {
    /// `role_a` を `role_b` より暗くする
    Darker,
    /// `role_a` を `role_b` より明るくする
    Lighter,
    /// `role_a` を `role_b` より背景のトーンに近づける
    Nearer,
    /// ライトでは暗く、ダークでは明るくする
    RelativeDarker,
    /// ライトでは明るく、ダークでは暗くする
    RelativeLighter,
}

/// トーンの差の制約 (2025 仕様以降の計算で使う)
///
/// 移植元の `DeltaConstraint` のうち、どの役割も使わない `nearer` は省いている
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DeltaConstraint {
    /// 差をちょうど `delta` にする
    Exact,
    /// 差を `delta` 以上にする
    Farther,
}

/// トーンの差を保つ 2 つの役割の組
///
/// 一方を調整したときにもう一方も動かし、区別できる差を保つ
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ToneDeltaPair {
    pub(crate) role_a: ColorRole,
    pub(crate) role_b: ColorRole,
    pub(crate) delta: f64,
    pub(crate) polarity: TonePolarity,
    /// 2021 仕様の計算で、トーン 50〜59 を避けるときに両方を同じ側へ動かすか
    pub(crate) stay_together: bool,
    pub(crate) constraint: DeltaConstraint,
}

impl ToneDeltaPair {
    /// 移植元のコンストラクタと同じく、`constraint` を省略したときは [`DeltaConstraint::Exact`]
    pub(crate) const fn new(
        role_a: ColorRole,
        role_b: ColorRole,
        delta: f64,
        polarity: TonePolarity,
        stay_together: bool,
    ) -> Self {
        Self {
            role_a,
            role_b,
            delta,
            polarity,
            stay_together,
            constraint: DeltaConstraint::Exact,
        }
    }

    pub(crate) const fn with_constraint(mut self, constraint: DeltaConstraint) -> Self {
        self.constraint = constraint;
        self
    }
}
