//! 高さ (elevation) の段階
//!
//! コンポーネントのトークンは高さを [`Elevation`] の段階で参照する。影の描画パラメータは [`shadow`](crate::shadow) にある

use crate::Dp;

pub use crate::generated::elevation::*;

/// 高さの段階。Compose の `ElevationTokens.Level0`〜`Level5` に当たる
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Elevation {
    /// 段階 0
    #[default]
    Level0,
    /// 段階 1
    Level1,
    /// 段階 2
    Level2,
    /// 段階 3
    Level3,
    /// 段階 4
    Level4,
    /// 段階 5
    Level5,
}

impl Elevation {
    /// すべての段階。低い順
    pub const ALL: [Self; 6] = [
        Self::Level0,
        Self::Level1,
        Self::Level2,
        Self::Level3,
        Self::Level4,
        Self::Level5,
    ];

    /// 段階の番号 (0〜5)。[`shadow::shadow`](crate::shadow::shadow) に渡す値
    pub const fn level(self) -> u8 {
        self as u8
    }

    /// 面の高さ
    pub const fn dp(self) -> Dp {
        match self {
            Self::Level0 => LEVEL0,
            Self::Level1 => LEVEL1,
            Self::Level2 => LEVEL2,
            Self::Level3 => LEVEL3,
            Self::Level4 => LEVEL4,
            Self::Level5 => LEVEL5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::filled_button;

    #[test]
    fn 段階の番号と高さが_elevation_tokens_に対応する() {
        // ElevationTokens.kt の Level0〜Level5 は 0, 1, 3, 6, 8, 12 dp
        let dps: Vec<f32> = Elevation::ALL.iter().map(|e| e.dp().0).collect();
        assert_eq!(dps, [0.0, 1.0, 3.0, 6.0, 8.0, 12.0]);
        let levels: Vec<u8> = Elevation::ALL.iter().map(|e| e.level()).collect();
        assert_eq!(levels, [0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn コンポーネントのトークンは段階で参照する() {
        // FilledButtonTokens.kt の HoveredContainerElevation は ElevationTokens.Level1
        assert_eq!(
            filled_button::HOVERED_CONTAINER_ELEVATION,
            Elevation::Level1
        );
    }
}
