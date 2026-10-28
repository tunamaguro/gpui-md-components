//! Material Design 3 の仕様に基づくデザイントークン。
//!
//! タイプスケール・角丸・高さ・モーション・状態レイヤー不透明度・密度を定数で提供する。
//! gpui に依存しない。寸法の単位は dp / sp（論理ピクセル）で、ピクセルへの変換はテーマで行う。
//!
//! すべての値の出典は `docs/TOKENS_SOURCE.md` にあり、`cargo xtask check-tokens` で
//! 定数と突き合わせている。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod component;
pub mod density;
pub mod elevation;
pub mod layout;
pub mod motion;
pub mod shape;
pub mod state;
pub mod typography;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_scale_has_15_styles() {
        assert_eq!(typography::TypeScale::ALL.len(), 15);
        let body_large = typography::TypeScale::BodyLarge.style();
        assert_eq!(body_large.size, 16.0);
        assert_eq!(body_large.line_height, 24.0);
        assert_eq!(body_large.weight, typography::WEIGHT_REGULAR);
    }

    #[test]
    fn elevation_levels_are_increasing() {
        use elevation::Elevation::*;
        let levels = [Level0, Level1, Level2, Level3, Level4, Level5];
        assert!(levels.windows(2).all(|w| w[0].dp() < w[1].dp()));
    }

    #[test]
    fn elevation_from_dp_round_trips() {
        for level in elevation::Elevation::ALL {
            assert_eq!(elevation::Elevation::from_dp(level.dp()), level);
        }
        assert_eq!(
            elevation::Elevation::from_dp(4.0),
            elevation::Elevation::Level2
        );
    }
}
