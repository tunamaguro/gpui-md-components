// Ported from material-color-utilities: typescript/dynamiccolor/contrast_curve.ts @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3. See NOTICE.

use crate::utils::math;

/// コントラストレベルに応じたコントラスト比の目標値。
///
/// レベル -1 / 0 / 0.5 / 1 の値を持ち、間は線形補間する。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContrastCurve {
    /// コントラストレベル -1 のときの値。
    pub low: f64,
    /// コントラストレベル 0 のときの値。
    pub normal: f64,
    /// コントラストレベル 0.5 のときの値。
    pub medium: f64,
    /// コントラストレベル 1 のときの値。
    pub high: f64,
}

impl ContrastCurve {
    /// 各レベルの値から作る。
    pub const fn new(low: f64, normal: f64, medium: f64, high: f64) -> Self {
        Self {
            low,
            normal,
            medium,
            high,
        }
    }

    /// コントラストレベル（-1〜1）に対する値。
    pub fn get(&self, contrast_level: f64) -> f64 {
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
