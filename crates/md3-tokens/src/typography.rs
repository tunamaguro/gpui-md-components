//! 書体
//!
//! コンポーネントのトークンは書体を [`TypeScale`] で参照する。基本の 15 種と、
//! Expressive 用の `*Emphasized` 15 種は別のキーで、基本のキーが Emphasized に置き換わることはない
//!
//! [`TypeStyle`] の組み合わせは Jetpack Compose Material3 の `TypographyTokens`
//! (`compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/TypographyTokens.kt`
//! @ 2a941dd11aabd9bb7d2016ed159764b220117c20 (tokens VERSION 31.0.11)) に従う。
//! Compose が `DefaultTextStyle` から引き継ぐ行高の配置などの描画設定は含まない

use crate::Sp;
use crate::generated::type_scale;

pub use crate::generated::typeface::*;
pub use crate::generated::typography_key::TypeScale;

/// 総称的なフォントファミリー
///
/// 具体的なフォントは利用する環境やテーマが決める
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FontFamily {
    /// サンセリフ体
    SansSerif,
}

/// 書体の系統。Compose の `TypefaceTokens.Brand` / `TypefaceTokens.Plain` に当たる
///
/// テーマは系統ごとに別のフォントに置き換えられる
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Typeface {
    /// ブランドの書体
    Brand,
    /// 本文の書体
    Plain,
}

impl Typeface {
    /// この系統の既定のフォントファミリー
    pub const fn family(self) -> FontFamily {
        match self {
            Self::Brand => BRAND,
            Self::Plain => PLAIN,
        }
    }
}

/// フォントの太さ (1〜1000)
///
/// 定数の値は Compose の `FontWeight`
/// (`compose/ui/ui-text/src/commonMain/kotlin/androidx/compose/ui/text/font/FontWeight.kt`
/// @ 2a941dd11aabd9bb7d2016ed159764b220117c20) と同じ
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FontWeight(pub u16);

impl FontWeight {
    /// `FontWeight.Normal` (`W400`)
    pub const NORMAL: Self = Self(400);
    /// `FontWeight.Medium` (`W500`)
    pub const MEDIUM: Self = Self(500);
    /// `FontWeight.Bold` (`W700`)
    pub const BOLD: Self = Self(700);
}

/// 1 つの書体の値
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeStyle {
    /// 書体の系統
    pub font: Typeface,
    /// 太さ
    pub weight: FontWeight,
    /// 文字の大きさ
    pub size: Sp,
    /// 行の高さ
    pub line_height: Sp,
    /// 字間 (tracking)。文字ごとに加える間隔
    pub tracking: Sp,
}

impl TypeScale {
    /// このキーの既定の書体
    pub const fn style(self) -> TypeStyle {
        match self {
            Self::BodyLarge => type_scale::BODY_LARGE,
            Self::BodyMedium => type_scale::BODY_MEDIUM,
            Self::BodySmall => type_scale::BODY_SMALL,
            Self::DisplayLarge => type_scale::DISPLAY_LARGE,
            Self::DisplayMedium => type_scale::DISPLAY_MEDIUM,
            Self::DisplaySmall => type_scale::DISPLAY_SMALL,
            Self::HeadlineLarge => type_scale::HEADLINE_LARGE,
            Self::HeadlineMedium => type_scale::HEADLINE_MEDIUM,
            Self::HeadlineSmall => type_scale::HEADLINE_SMALL,
            Self::LabelLarge => type_scale::LABEL_LARGE,
            Self::LabelMedium => type_scale::LABEL_MEDIUM,
            Self::LabelSmall => type_scale::LABEL_SMALL,
            Self::TitleLarge => type_scale::TITLE_LARGE,
            Self::TitleMedium => type_scale::TITLE_MEDIUM,
            Self::TitleSmall => type_scale::TITLE_SMALL,
            Self::BodyLargeEmphasized => type_scale::BODY_LARGE_EMPHASIZED,
            Self::BodyMediumEmphasized => type_scale::BODY_MEDIUM_EMPHASIZED,
            Self::BodySmallEmphasized => type_scale::BODY_SMALL_EMPHASIZED,
            Self::DisplayLargeEmphasized => type_scale::DISPLAY_LARGE_EMPHASIZED,
            Self::DisplayMediumEmphasized => type_scale::DISPLAY_MEDIUM_EMPHASIZED,
            Self::DisplaySmallEmphasized => type_scale::DISPLAY_SMALL_EMPHASIZED,
            Self::HeadlineLargeEmphasized => type_scale::HEADLINE_LARGE_EMPHASIZED,
            Self::HeadlineMediumEmphasized => type_scale::HEADLINE_MEDIUM_EMPHASIZED,
            Self::HeadlineSmallEmphasized => type_scale::HEADLINE_SMALL_EMPHASIZED,
            Self::LabelLargeEmphasized => type_scale::LABEL_LARGE_EMPHASIZED,
            Self::LabelMediumEmphasized => type_scale::LABEL_MEDIUM_EMPHASIZED,
            Self::LabelSmallEmphasized => type_scale::LABEL_SMALL_EMPHASIZED,
            Self::TitleLargeEmphasized => type_scale::TITLE_LARGE_EMPHASIZED,
            Self::TitleMediumEmphasized => type_scale::TITLE_MEDIUM_EMPHASIZED,
            Self::TitleSmallEmphasized => type_scale::TITLE_SMALL_EMPHASIZED,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 書体の系統は解決されずに残る() {
        // TypeScaleTokens.kt では DisplayLargeFont が Brand、BodyLargeFont が Plain
        assert_eq!(TypeScale::DisplayLarge.style().font, Typeface::Brand);
        assert_eq!(TypeScale::BodyLarge.style().font, Typeface::Plain);
        assert_eq!(Typeface::Brand.family(), FontFamily::SansSerif);
    }
}
