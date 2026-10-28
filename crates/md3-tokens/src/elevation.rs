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

/// 影 1 つぶんの形（CSS の `box-shadow: x y blur spread`）。単位は dp。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    /// 横方向のずれ。
    pub x: f32,
    /// 縦方向のずれ。
    pub y: f32,
    /// ぼかしの半径。
    pub blur: f32,
    /// 広がり。
    pub spread: f32,
}

impl Shadow {
    /// 値から作る。
    pub const fn new(x: f32, y: f32, blur: f32, spread: f32) -> Self {
        Self { x, y, blur, spread }
    }
}

/// キーシャドウ（輪郭のはっきりした影）の不透明度。色は `shadow` ロール。
pub const KEY_SHADOW_OPACITY: f32 = 0.3;

/// アンビエントシャドウ（広がった影）の不透明度。色は `shadow` ロール。
pub const AMBIENT_SHADOW_OPACITY: f32 = 0.15;

/// レベル 0 のキーシャドウ。
pub const KEY_SHADOW_LEVEL0: Shadow = Shadow::new(0.0, 0.0, 0.0, 0.0);

/// レベル 1 のキーシャドウ。
pub const KEY_SHADOW_LEVEL1: Shadow = Shadow::new(0.0, 1.0, 2.0, 0.0);

/// レベル 2 のキーシャドウ。
pub const KEY_SHADOW_LEVEL2: Shadow = Shadow::new(0.0, 1.0, 2.0, 0.0);

/// レベル 3 のキーシャドウ。
pub const KEY_SHADOW_LEVEL3: Shadow = Shadow::new(0.0, 1.0, 3.0, 0.0);

/// レベル 4 のキーシャドウ。
pub const KEY_SHADOW_LEVEL4: Shadow = Shadow::new(0.0, 2.0, 3.0, 0.0);

/// レベル 5 のキーシャドウ。
pub const KEY_SHADOW_LEVEL5: Shadow = Shadow::new(0.0, 4.0, 4.0, 0.0);

/// レベル 0 のアンビエントシャドウ。
pub const AMBIENT_SHADOW_LEVEL0: Shadow = Shadow::new(0.0, 0.0, 0.0, 0.0);

/// レベル 1 のアンビエントシャドウ。
pub const AMBIENT_SHADOW_LEVEL1: Shadow = Shadow::new(0.0, 1.0, 3.0, 1.0);

/// レベル 2 のアンビエントシャドウ。
pub const AMBIENT_SHADOW_LEVEL2: Shadow = Shadow::new(0.0, 2.0, 6.0, 2.0);

/// レベル 3 のアンビエントシャドウ。
pub const AMBIENT_SHADOW_LEVEL3: Shadow = Shadow::new(0.0, 4.0, 8.0, 3.0);

/// レベル 4 のアンビエントシャドウ。
pub const AMBIENT_SHADOW_LEVEL4: Shadow = Shadow::new(0.0, 6.0, 10.0, 4.0);

/// レベル 5 のアンビエントシャドウ。
pub const AMBIENT_SHADOW_LEVEL5: Shadow = Shadow::new(0.0, 8.0, 12.0, 6.0);

impl Elevation {
    /// キーシャドウとアンビエントシャドウ。
    pub const fn shadows(self) -> (Shadow, Shadow) {
        match self {
            Elevation::Level0 => (KEY_SHADOW_LEVEL0, AMBIENT_SHADOW_LEVEL0),
            Elevation::Level1 => (KEY_SHADOW_LEVEL1, AMBIENT_SHADOW_LEVEL1),
            Elevation::Level2 => (KEY_SHADOW_LEVEL2, AMBIENT_SHADOW_LEVEL2),
            Elevation::Level3 => (KEY_SHADOW_LEVEL3, AMBIENT_SHADOW_LEVEL3),
            Elevation::Level4 => (KEY_SHADOW_LEVEL4, AMBIENT_SHADOW_LEVEL4),
            Elevation::Level5 => (KEY_SHADOW_LEVEL5, AMBIENT_SHADOW_LEVEL5),
        }
    }
}
