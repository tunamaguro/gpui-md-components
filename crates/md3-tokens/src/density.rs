//! 部品の密度
//!
//! Material Design 3 の仕様 <https://m3.material.io/foundations/layout/grids-spacing/density>
//! (2026-11-04 に参照) に従う。密度は既定の 0 から -1、-2、-3 と負の向きに上がり、
//! 上がるほど部品の上下の余白か高さを小さくする。横方向の余白は変えない。
//! 1 段あたりの変化量 4dp は Material Design の記事
//! <https://m3.material.io/blog/material-density-web> の「1 段ごとに高さを 4px 小さくする」による。
//! Compose と Material Web には密度のトークンがない
//!
//! どの部品がどの段階まで対応するかは部品ごとに決まる。
//! 仕様は、密度を利用者が選べるようにし、操作できる範囲を既定で 48x48dp 未満にしないことを求めている

use crate::Dp;

/// 密度 1 段あたりの寸法の変化量
pub const INTERVAL: Dp = Dp(4.0);

/// 密度の段階
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Density {
    /// 0 (既定)
    #[default]
    Default,
    /// -1
    Minus1,
    /// -2
    Minus2,
    /// -3 (最も密)
    Minus3,
}

impl Density {
    /// すべての段階。既定から密になる順
    pub const ALL: [Self; 4] = [Self::Default, Self::Minus1, Self::Minus2, Self::Minus3];

    /// 段階の番号 (0〜-3)
    pub const fn scale(self) -> i8 {
        match self {
            Self::Default => 0,
            Self::Minus1 => -1,
            Self::Minus2 => -2,
            Self::Minus3 => -3,
        }
    }

    /// 既定の寸法 `size` にこの密度を適用した寸法。`size + scale × INTERVAL`
    ///
    /// 結果が 0 未満になっても丸めない。部品の最小寸法や操作領域の確保は呼び出し側が行う
    pub const fn apply(self, size: Dp) -> Dp {
        Dp(size.0 + self.scale() as f32 * INTERVAL.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 一段ごとに_4dp_小さくなる() {
        let heights: Vec<f32> = Density::ALL.iter().map(|d| d.apply(Dp(40.0)).0).collect();
        assert_eq!(heights, [40.0, 36.0, 32.0, 28.0]);
    }
}
