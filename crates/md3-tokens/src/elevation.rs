//! 高さ（elevation）のレベル 0〜5。単位は dp。

// 値は docs/TOKENS_SOURCE.md に記載の一次資料から転記した。`cargo xtask check-tokens` で突き合わせる。

/// レベル 0 の高さ（dp）。
pub const LEVEL0: f32 = 0.0;

/// レベル 1 の高さ（dp）。
pub const LEVEL1: f32 = 1.0;

/// レベル 2 の高さ（dp）。
pub const LEVEL2: f32 = 3.0;

/// レベル 3 の高さ（dp）。
pub const LEVEL3: f32 = 6.0;

/// レベル 4 の高さ（dp）。
pub const LEVEL4: f32 = 8.0;

/// レベル 5 の高さ（dp）。
pub const LEVEL5: f32 = 12.0;

/// 高さのレベル。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Elevation {
    /// レベル 0。
    Level0,
    /// レベル 1。
    Level1,
    /// レベル 2。
    Level2,
    /// レベル 3。
    Level3,
    /// レベル 4。
    Level4,
    /// レベル 5。
    Level5,
}

impl Elevation {
    /// 高さ（dp）。
    pub const fn dp(self) -> f32 {
        match self {
            Elevation::Level0 => LEVEL0,
            Elevation::Level1 => LEVEL1,
            Elevation::Level2 => LEVEL2,
            Elevation::Level3 => LEVEL3,
            Elevation::Level4 => LEVEL4,
            Elevation::Level5 => LEVEL5,
        }
    }
}
