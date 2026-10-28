//! ウィンドウの幅のクラス（compact / medium / expanded / large / extra-large）。
//!
//! 境界の値は androidx.window の `WindowSizeClass`（`md3_tokens::layout::WINDOW_WIDTH_*`）。

use gpui::{Pixels, Window, px};
use md3_theme::md3_tokens::layout;

/// ウィンドウの幅のクラス。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WindowSizeClass {
    /// 600dp 未満（スマートフォンの縦向きなど）。
    #[default]
    Compact,
    /// 600dp 以上 840dp 未満。
    Medium,
    /// 840dp 以上 1200dp 未満。
    Expanded,
    /// 1200dp 以上 1600dp 未満。
    Large,
    /// 1600dp 以上。
    ExtraLarge,
}

impl WindowSizeClass {
    /// 幅からクラスを決める。gpui の論理ピクセルを dp と同じに扱う。
    pub fn from_width(width: Pixels) -> Self {
        let width = f32::from(width);
        if width >= layout::WINDOW_WIDTH_EXTRA_LARGE {
            Self::ExtraLarge
        } else if width >= layout::WINDOW_WIDTH_LARGE {
            Self::Large
        } else if width >= layout::WINDOW_WIDTH_EXPANDED {
            Self::Expanded
        } else if width >= layout::WINDOW_WIDTH_MEDIUM {
            Self::Medium
        } else {
            Self::Compact
        }
    }

    /// ウィンドウの今の幅のクラス。
    pub fn of(window: &Window) -> Self {
        Self::from_width(window.viewport_size().width)
    }

    /// クラスの下限の幅。
    pub fn min_width(self) -> Pixels {
        px(match self {
            Self::Compact => 0.0,
            Self::Medium => layout::WINDOW_WIDTH_MEDIUM,
            Self::Expanded => layout::WINDOW_WIDTH_EXPANDED,
            Self::Large => layout::WINDOW_WIDTH_LARGE,
            Self::ExtraLarge => layout::WINDOW_WIDTH_EXTRA_LARGE,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breakpoints() {
        assert_eq!(
            WindowSizeClass::from_width(px(599.0)),
            WindowSizeClass::Compact
        );
        assert_eq!(
            WindowSizeClass::from_width(px(600.0)),
            WindowSizeClass::Medium
        );
        assert_eq!(
            WindowSizeClass::from_width(px(839.0)),
            WindowSizeClass::Medium
        );
        assert_eq!(
            WindowSizeClass::from_width(px(840.0)),
            WindowSizeClass::Expanded
        );
        assert_eq!(
            WindowSizeClass::from_width(px(1200.0)),
            WindowSizeClass::Large
        );
        assert_eq!(
            WindowSizeClass::from_width(px(1600.0)),
            WindowSizeClass::ExtraLarge
        );
    }

    #[test]
    fn min_width_is_the_lower_bound() {
        for class in [
            WindowSizeClass::Compact,
            WindowSizeClass::Medium,
            WindowSizeClass::Expanded,
            WindowSizeClass::Large,
            WindowSizeClass::ExtraLarge,
        ] {
            assert_eq!(WindowSizeClass::from_width(class.min_width()), class);
        }
    }
}
