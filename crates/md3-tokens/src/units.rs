//! 論理単位

/// 密度に依存しない長さ (dp)
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Dp(pub f32);

/// 文字の大きさの単位 (sp)。利用者の文字倍率を掛ける前の値
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Sp(pub f32);
