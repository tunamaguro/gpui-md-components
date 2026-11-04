//! 角丸の形
//!
//! コンポーネントのトークンは形を [`ShapeKey`] で参照する。キーは [`Shapes`] の段階から解決され、
//! テーマが段階を上書きすると、そのキーを使うすべての部品に反映される
//!
//! [`ShapeKey::resolve`] は Jetpack Compose Material3 の `Shapes.fromToken`
//! (`compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Shapes.kt`
//! @ 2a941dd11aabd9bb7d2016ed159764b220117c20) と同じ対応で解決する

use crate::Dp;

pub use crate::generated::shape::*;
pub use crate::generated::shape_key::ShapeKey;

/// 1 つの角の丸みの大きさ
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CornerSize {
    /// 半径
    Dp(Dp),
    /// 形の短辺に対する割合 (0〜100)。50 で短辺の半分の半径になる
    Percent(f32),
}

impl CornerSize {
    /// 丸みのない角
    pub const ZERO: Self = Self::Dp(Dp(0.0));
}

/// 4 つの角の丸みを持つ形
///
/// start / end は書字方向に対する論理的な向きで、右から左へ書く言語では左右が入れ替わる
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CornerShape {
    /// 上の start 側の角
    pub top_start: CornerSize,
    /// 上の end 側の角
    pub top_end: CornerSize,
    /// 下の end 側の角
    pub bottom_end: CornerSize,
    /// 下の start 側の角
    pub bottom_start: CornerSize,
}

impl CornerShape {
    /// 両端が半円になる形。Compose の `CircleShape` (`RoundedCornerShape(50)`) に当たる
    pub const CIRCLE: Self = Self::all(CornerSize::Percent(50.0));

    /// 角の丸みのない矩形。Compose の `RectangleShape` に当たる
    pub const RECTANGLE: Self = Self::all(CornerSize::ZERO);

    /// 4 つの角を同じ大きさにした形
    pub const fn all(size: CornerSize) -> Self {
        Self {
            top_start: size,
            top_end: size,
            bottom_end: size,
            bottom_start: size,
        }
    }

    /// 上の 2 つの角だけを残した形。Compose の `CornerBasedShape.top()` に当たる
    pub const fn top(self) -> Self {
        Self {
            bottom_end: CornerSize::ZERO,
            bottom_start: CornerSize::ZERO,
            ..self
        }
    }

    /// start 側の 2 つの角だけを残した形。Compose の `CornerBasedShape.start()` に当たる
    pub const fn start(self) -> Self {
        Self {
            top_end: CornerSize::ZERO,
            bottom_end: CornerSize::ZERO,
            ..self
        }
    }

    /// end 側の 2 つの角だけを残した形。Compose の `CornerBasedShape.end()` に当たる
    pub const fn end(self) -> Self {
        Self {
            top_start: CornerSize::ZERO,
            bottom_start: CornerSize::ZERO,
            ..self
        }
    }
}

/// 角丸の段階。Compose の `Shapes` に当たる
///
/// [`Default`] は Compose の `ShapeDefaults` と同じく、`ShapeTokens` の値 (このモジュールの `CORNER_*`) を使う
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shapes {
    /// extra small
    pub extra_small: CornerShape,
    /// small
    pub small: CornerShape,
    /// medium
    pub medium: CornerShape,
    /// large
    pub large: CornerShape,
    /// large increased
    pub large_increased: CornerShape,
    /// extra large
    pub extra_large: CornerShape,
    /// extra large increased
    pub extra_large_increased: CornerShape,
    /// extra extra large
    pub extra_extra_large: CornerShape,
}

impl Default for Shapes {
    fn default() -> Self {
        Self {
            extra_small: CORNER_EXTRA_SMALL,
            small: CORNER_SMALL,
            medium: CORNER_MEDIUM,
            large: CORNER_LARGE,
            large_increased: CORNER_LARGE_INCREASED,
            extra_large: CORNER_EXTRA_LARGE,
            extra_large_increased: CORNER_EXTRA_LARGE_INCREASED,
            extra_extra_large: CORNER_EXTRA_EXTRA_LARGE,
        }
    }
}

impl ShapeKey {
    /// `shapes` の段階からこのキーの形を求める
    ///
    /// `CornerFull` と `CornerNone` は段階によらず、それぞれ [`CornerShape::CIRCLE`] と
    /// [`CornerShape::RECTANGLE`] になる
    pub fn resolve(self, shapes: &Shapes) -> CornerShape {
        match self {
            Self::CornerExtraExtraLarge => shapes.extra_extra_large,
            Self::CornerExtraLarge => shapes.extra_large,
            Self::CornerExtraLargeIncreased => shapes.extra_large_increased,
            Self::CornerExtraLargeTop => shapes.extra_large.top(),
            Self::CornerExtraSmall => shapes.extra_small,
            Self::CornerExtraSmallTop => shapes.extra_small.top(),
            Self::CornerFull => CornerShape::CIRCLE,
            Self::CornerLarge => shapes.large,
            Self::CornerLargeEnd => shapes.large.end(),
            Self::CornerLargeIncreased => shapes.large_increased,
            Self::CornerLargeStart => shapes.large.start(),
            Self::CornerLargeTop => shapes.large.top(),
            Self::CornerMedium => shapes.medium,
            Self::CornerNone => CornerShape::RECTANGLE,
            Self::CornerSmall => shapes.small,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 既定の段階から解決した形は_shape_tokens_の同名の形と一致する() {
        // ShapeTokens には部分的な角丸も転記されているため、Shapes.fromToken の解決結果と照合できる
        let shapes = Shapes::default();
        let cases = [
            (ShapeKey::CornerExtraExtraLarge, CORNER_EXTRA_EXTRA_LARGE),
            (ShapeKey::CornerExtraLarge, CORNER_EXTRA_LARGE),
            (
                ShapeKey::CornerExtraLargeIncreased,
                CORNER_EXTRA_LARGE_INCREASED,
            ),
            (ShapeKey::CornerExtraLargeTop, CORNER_EXTRA_LARGE_TOP),
            (ShapeKey::CornerExtraSmall, CORNER_EXTRA_SMALL),
            (ShapeKey::CornerExtraSmallTop, CORNER_EXTRA_SMALL_TOP),
            (ShapeKey::CornerFull, CORNER_FULL),
            (ShapeKey::CornerLarge, CORNER_LARGE),
            (ShapeKey::CornerLargeEnd, CORNER_LARGE_END),
            (ShapeKey::CornerLargeIncreased, CORNER_LARGE_INCREASED),
            (ShapeKey::CornerLargeStart, CORNER_LARGE_START),
            (ShapeKey::CornerLargeTop, CORNER_LARGE_TOP),
            (ShapeKey::CornerMedium, CORNER_MEDIUM),
            (ShapeKey::CornerNone, CORNER_NONE),
            (ShapeKey::CornerSmall, CORNER_SMALL),
        ];
        assert_eq!(cases.len(), ShapeKey::ALL.len());
        for (key, expected) in cases {
            assert_eq!(key.resolve(&shapes), expected, "{key:?}");
        }
    }

    #[test]
    fn 段階を上書きすると部分的な角丸にも反映される() {
        let shapes = Shapes {
            large: CornerShape::all(CornerSize::Dp(Dp(10.0))),
            ..Shapes::default()
        };
        let start = ShapeKey::CornerLargeStart.resolve(&shapes);
        assert_eq!(start.top_start, CornerSize::Dp(Dp(10.0)));
        assert_eq!(start.bottom_start, CornerSize::Dp(Dp(10.0)));
        assert_eq!(start.top_end, CornerSize::ZERO);
        assert_eq!(start.bottom_end, CornerSize::ZERO);
    }
}
