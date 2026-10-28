//! 文字スタイル（タイプスケール）の gpui 向け表現。

use gpui::{FontWeight, Pixels, SharedString, px};
use md3_tokens::typography::{self as tokens, TypeScale, TypeStyle, Typeface};

/// 1 つの文字スタイル。値は gpui の単位で持つ。
#[derive(Clone, Debug, PartialEq)]
pub struct TypeScaleStyle {
    /// フォントファミリ。`None` ならテーマのフォント（未指定ならシステムフォント）を使う。
    pub font_family: Option<SharedString>,
    /// 書体の役割（見出し向けか本文向けか）。
    pub typeface: Typeface,
    /// 太さ。
    pub font_weight: FontWeight,
    /// 文字の大きさ。
    pub font_size: Pixels,
    /// 行の高さ。
    pub line_height: Pixels,
    /// 字間。gpui は字間の指定に対応していないため、現在は描画に反映されない。
    pub tracking: Pixels,
}

impl TypeScaleStyle {
    /// トークンの値から作る（dp / sp は gpui の論理ピクセルと 1:1）。
    pub fn from_token(style: TypeStyle) -> Self {
        Self {
            font_family: None,
            typeface: style.typeface,
            font_weight: FontWeight(f32::from(style.weight)),
            font_size: px(style.size),
            line_height: px(style.line_height),
            tracking: px(style.tracking),
        }
    }

    /// フォントファミリを変える。
    pub fn font_family(mut self, family: impl Into<SharedString>) -> Self {
        self.font_family = Some(family.into());
        self
    }

    /// 太さを変える。
    pub fn weight(mut self, weight: FontWeight) -> Self {
        self.font_weight = weight;
        self
    }

    /// 文字の大きさを変える。
    pub fn size(mut self, size: Pixels) -> Self {
        self.font_size = size;
        self
    }

    /// 行の高さを変える。
    pub fn line_height(mut self, line_height: Pixels) -> Self {
        self.line_height = line_height;
        self
    }
}

/// タイプスケール 15 種の文字スタイル。利用者が個別に上書きできる。
#[derive(Clone, Debug, PartialEq)]
pub struct TypeScaleStyles {
    /// Display Large。
    pub display_large: TypeScaleStyle,
    /// Display Medium。
    pub display_medium: TypeScaleStyle,
    /// Display Small。
    pub display_small: TypeScaleStyle,
    /// Headline Large。
    pub headline_large: TypeScaleStyle,
    /// Headline Medium。
    pub headline_medium: TypeScaleStyle,
    /// Headline Small。
    pub headline_small: TypeScaleStyle,
    /// Title Large。
    pub title_large: TypeScaleStyle,
    /// Title Medium。
    pub title_medium: TypeScaleStyle,
    /// Title Small。
    pub title_small: TypeScaleStyle,
    /// Body Large。
    pub body_large: TypeScaleStyle,
    /// Body Medium。
    pub body_medium: TypeScaleStyle,
    /// Body Small。
    pub body_small: TypeScaleStyle,
    /// Label Large。
    pub label_large: TypeScaleStyle,
    /// Label Medium。
    pub label_medium: TypeScaleStyle,
    /// Label Small。
    pub label_small: TypeScaleStyle,
}

impl Default for TypeScaleStyles {
    fn default() -> Self {
        let s = TypeScaleStyle::from_token;
        Self {
            display_large: s(tokens::DISPLAY_LARGE),
            display_medium: s(tokens::DISPLAY_MEDIUM),
            display_small: s(tokens::DISPLAY_SMALL),
            headline_large: s(tokens::HEADLINE_LARGE),
            headline_medium: s(tokens::HEADLINE_MEDIUM),
            headline_small: s(tokens::HEADLINE_SMALL),
            title_large: s(tokens::TITLE_LARGE),
            title_medium: s(tokens::TITLE_MEDIUM),
            title_small: s(tokens::TITLE_SMALL),
            body_large: s(tokens::BODY_LARGE),
            body_medium: s(tokens::BODY_MEDIUM),
            body_small: s(tokens::BODY_SMALL),
            label_large: s(tokens::LABEL_LARGE),
            label_medium: s(tokens::LABEL_MEDIUM),
            label_small: s(tokens::LABEL_SMALL),
        }
    }
}

impl TypeScaleStyles {
    /// 種類に対応する文字スタイル。
    pub fn get(&self, scale: TypeScale) -> &TypeScaleStyle {
        match scale {
            TypeScale::DisplayLarge => &self.display_large,
            TypeScale::DisplayMedium => &self.display_medium,
            TypeScale::DisplaySmall => &self.display_small,
            TypeScale::HeadlineLarge => &self.headline_large,
            TypeScale::HeadlineMedium => &self.headline_medium,
            TypeScale::HeadlineSmall => &self.headline_small,
            TypeScale::TitleLarge => &self.title_large,
            TypeScale::TitleMedium => &self.title_medium,
            TypeScale::TitleSmall => &self.title_small,
            TypeScale::BodyLarge => &self.body_large,
            TypeScale::BodyMedium => &self.body_medium,
            TypeScale::BodySmall => &self.body_small,
            TypeScale::LabelLarge => &self.label_large,
            TypeScale::LabelMedium => &self.label_medium,
            TypeScale::LabelSmall => &self.label_small,
        }
    }

    /// 種類に対応する文字スタイル（書き換え用）。
    pub fn get_mut(&mut self, scale: TypeScale) -> &mut TypeScaleStyle {
        match scale {
            TypeScale::DisplayLarge => &mut self.display_large,
            TypeScale::DisplayMedium => &mut self.display_medium,
            TypeScale::DisplaySmall => &mut self.display_small,
            TypeScale::HeadlineLarge => &mut self.headline_large,
            TypeScale::HeadlineMedium => &mut self.headline_medium,
            TypeScale::HeadlineSmall => &mut self.headline_small,
            TypeScale::TitleLarge => &mut self.title_large,
            TypeScale::TitleMedium => &mut self.title_medium,
            TypeScale::TitleSmall => &mut self.title_small,
            TypeScale::BodyLarge => &mut self.body_large,
            TypeScale::BodyMedium => &mut self.body_medium,
            TypeScale::BodySmall => &mut self.body_small,
            TypeScale::LabelLarge => &mut self.label_large,
            TypeScale::LabelMedium => &mut self.label_medium,
            TypeScale::LabelSmall => &mut self.label_small,
        }
    }
}
