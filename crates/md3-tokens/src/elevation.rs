//! 高さ (elevation) の段階
//!
//! 値は面の高さを表す [`Dp`] で、影の描画パラメータは [`shadow`](crate::shadow) にある

use crate::Dp;

pub use crate::generated::elevation::*;

/// 段階の番号ごとの高さ。添字が段階の番号 (0〜5) で、LEVEL0〜LEVEL5 と同じ値
pub const LEVELS: [Dp; 6] = [LEVEL0, LEVEL1, LEVEL2, LEVEL3, LEVEL4, LEVEL5];

/// 高さ `elevation` に当たる段階の番号
///
/// コンポーネントのトークンの高さを [`shadow::shadow`](crate::shadow::shadow) に渡す番号に変える。
/// [`LEVELS`] のどれとも一致しなければ `None`
pub fn level_of(elevation: Dp) -> Option<u8> {
    LEVELS
        .iter()
        .position(|level| *level == elevation)
        .and_then(|index| u8::try_from(index).ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::filled_button;

    #[test]
    fn コンポーネントの高さから段階の番号を求める() {
        // FilledButtonTokens.kt の HoveredContainerElevation は ElevationTokens.Level1
        assert_eq!(
            level_of(filled_button::HOVERED_CONTAINER_ELEVATION),
            Some(1)
        );
        assert_eq!(level_of(Dp(6.0)), Some(3));
        assert_eq!(level_of(Dp(2.0)), None);
    }
}
