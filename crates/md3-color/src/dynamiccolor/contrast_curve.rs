// Ported from material-color-utilities
// (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/dynamiccolor/contrast_curve.ts @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2023 Google LLC. Licensed under the Apache License, Version 2.0.

use crate::utils::math;

/// コントラストレベルに応じたコントラスト比の目標
///
/// コントラストレベル -1、0、0.5、1 での値を持ち、その間は線形補間する
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ContrastCurve {
    low: f64,
    normal: f64,
    medium: f64,
    high: f64,
}

impl ContrastCurve {
    pub(crate) const fn new(low: f64, normal: f64, medium: f64, high: f64) -> Self {
        Self {
            low,
            normal,
            medium,
            high,
        }
    }

    /// `contrast_level` でのコントラスト比
    pub(crate) fn get(&self, contrast_level: f64) -> f64 {
        if contrast_level <= -1.0 {
            self.low
        } else if contrast_level < 0.0 {
            math::lerp(self.low, self.normal, (contrast_level - (-1.0)) / 1.0)
        } else if contrast_level < 0.5 {
            math::lerp(self.normal, self.medium, (contrast_level - 0.0) / 0.5)
        } else if contrast_level < 1.0 {
            math::lerp(self.medium, self.high, (contrast_level - 0.5) / 0.5)
        } else {
            self.high
        }
    }
}
