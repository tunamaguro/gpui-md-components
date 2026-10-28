//! 角丸のスケール。単位は dp（論理ピクセル）。

// 値は docs/TOKENS_SOURCE.md に記載の一次資料から転記した。`cargo xtask check-tokens` で突き合わせる。

/// 角丸の大きさ。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CornerSize {
    /// 固定の半径（dp）。
    Fixed(f32),
    /// 短辺の半分（完全な丸）。
    Full,
}

/// 角丸 なし の半径（dp）。
pub const CORNER_NONE: f32 = 0.0;

/// 角丸 extra-small の半径（dp）。
pub const CORNER_EXTRA_SMALL: f32 = 4.0;

/// 角丸 small の半径（dp）。
pub const CORNER_SMALL: f32 = 8.0;

/// 角丸 medium の半径（dp）。
pub const CORNER_MEDIUM: f32 = 12.0;

/// 角丸 large の半径（dp）。
pub const CORNER_LARGE: f32 = 16.0;

/// 角丸 extra-large の半径（dp）。
pub const CORNER_EXTRA_LARGE: f32 = 28.0;

/// 角丸 full（完全な丸）。
pub const CORNER_FULL: CornerSize = CornerSize::Full;

/// 角丸スケールの段階。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Shape {
    /// なし。
    None,
    /// extra-small。
    ExtraSmall,
    /// small。
    Small,
    /// medium。
    Medium,
    /// large。
    Large,
    /// extra-large。
    ExtraLarge,
    /// full。
    Full,
}

impl Shape {
    /// 角丸の大きさ。
    pub const fn corner(self) -> CornerSize {
        match self {
            Shape::None => CornerSize::Fixed(CORNER_NONE),
            Shape::ExtraSmall => CornerSize::Fixed(CORNER_EXTRA_SMALL),
            Shape::Small => CornerSize::Fixed(CORNER_SMALL),
            Shape::Medium => CornerSize::Fixed(CORNER_MEDIUM),
            Shape::Large => CornerSize::Fixed(CORNER_LARGE),
            Shape::ExtraLarge => CornerSize::Fixed(CORNER_EXTRA_LARGE),
            Shape::Full => CORNER_FULL,
        }
    }
}
