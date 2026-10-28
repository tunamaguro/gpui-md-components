//! 密度スケール（0、-1、-2）。
//!
//! 密度を 1 段下げるごとに、高さなどの寸法を [`INTERVAL`] ずつ小さくする
//! （material-components-web の `mdc-density` と同じ計算）。

// 値は docs/TOKENS_SOURCE.md に記載の一次資料から転記した。`cargo xtask check-tokens` で突き合わせる。

/// 密度 1 段あたりの寸法の変化量（dp）。
pub const INTERVAL: f32 = 4.0;

/// 密度スケール。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Density {
    /// 0（既定）。
    #[default]
    Default,
    /// -1。
    Minus1,
    /// -2。
    Minus2,
}

impl Density {
    /// すべての段階（既定から順に）。
    pub const ALL: [Density; 3] = [Density::Default, Density::Minus1, Density::Minus2];

    /// スケールの値（0、-1、-2）。
    pub const fn scale(self) -> i8 {
        match self {
            Density::Default => 0,
            Density::Minus1 => -1,
            Density::Minus2 => -2,
        }
    }

    /// 寸法の変化量（dp）。`scale × INTERVAL` で、0 以下になる。
    pub const fn offset(self) -> f32 {
        self.scale() as f32 * INTERVAL
    }

    /// 既定の寸法 `default` にこの密度を適用した値（dp）。
    pub const fn apply(self, default: f32) -> f32 {
        default + self.offset()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offset_decreases_by_interval() {
        assert_eq!(Density::Default.apply(40.0), 40.0);
        assert_eq!(Density::Minus1.apply(40.0), 36.0);
        assert_eq!(Density::Minus2.apply(40.0), 32.0);
    }
}
