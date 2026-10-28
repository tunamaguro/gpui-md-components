//! タイプスケール（15 種）。
//!
//! 寸法の単位は sp（論理ピクセル）。トラッキング（字間）も sp。

// 値は docs/TOKENS_SOURCE.md に記載の一次資料から転記した。`cargo xtask check-tokens` で突き合わせる。

/// 書体の役割。具体的なフォントはテーマで決める（移植元はどちらも `FontFamily.SansSerif`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Typeface {
    /// 見出し向け（Display / Headline / Title Large）。
    Brand,
    /// 本文向け。
    Plain,
}

/// 1 つの文字スタイル。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeStyle {
    /// 書体の役割。
    pub typeface: Typeface,
    /// 太さ（100〜900）。
    pub weight: u16,
    /// 文字の大きさ（sp）。
    pub size: f32,
    /// 行の高さ（sp）。
    pub line_height: f32,
    /// 字間（sp）。
    pub tracking: f32,
}

/// 標準の太さ（Normal）。
pub const WEIGHT_REGULAR: u16 = 400;

/// 中くらいの太さ（Medium）。
pub const WEIGHT_MEDIUM: u16 = 500;

/// 太字（Bold）。
pub const WEIGHT_BOLD: u16 = 700;

/// Body Large の書体。
pub const BODY_LARGE_TYPEFACE: Typeface = Typeface::Plain;
/// Body Large の太さ。
pub const BODY_LARGE_WEIGHT: u16 = WEIGHT_REGULAR;
/// Body Large の文字の大きさ（sp）。
pub const BODY_LARGE_SIZE: f32 = 16.0;
/// Body Large の行の高さ（sp）。
pub const BODY_LARGE_LINE_HEIGHT: f32 = 24.0;
/// Body Large の字間（sp）。
pub const BODY_LARGE_TRACKING: f32 = 0.5;
/// Body Large の文字スタイル。
pub const BODY_LARGE: TypeStyle = TypeStyle {
    typeface: BODY_LARGE_TYPEFACE,
    weight: BODY_LARGE_WEIGHT,
    size: BODY_LARGE_SIZE,
    line_height: BODY_LARGE_LINE_HEIGHT,
    tracking: BODY_LARGE_TRACKING,
};

/// Body Medium の書体。
pub const BODY_MEDIUM_TYPEFACE: Typeface = Typeface::Plain;
/// Body Medium の太さ。
pub const BODY_MEDIUM_WEIGHT: u16 = WEIGHT_REGULAR;
/// Body Medium の文字の大きさ（sp）。
pub const BODY_MEDIUM_SIZE: f32 = 14.0;
/// Body Medium の行の高さ（sp）。
pub const BODY_MEDIUM_LINE_HEIGHT: f32 = 20.0;
/// Body Medium の字間（sp）。
pub const BODY_MEDIUM_TRACKING: f32 = 0.2;
/// Body Medium の文字スタイル。
pub const BODY_MEDIUM: TypeStyle = TypeStyle {
    typeface: BODY_MEDIUM_TYPEFACE,
    weight: BODY_MEDIUM_WEIGHT,
    size: BODY_MEDIUM_SIZE,
    line_height: BODY_MEDIUM_LINE_HEIGHT,
    tracking: BODY_MEDIUM_TRACKING,
};

/// Body Small の書体。
pub const BODY_SMALL_TYPEFACE: Typeface = Typeface::Plain;
/// Body Small の太さ。
pub const BODY_SMALL_WEIGHT: u16 = WEIGHT_REGULAR;
/// Body Small の文字の大きさ（sp）。
pub const BODY_SMALL_SIZE: f32 = 12.0;
/// Body Small の行の高さ（sp）。
pub const BODY_SMALL_LINE_HEIGHT: f32 = 16.0;
/// Body Small の字間（sp）。
pub const BODY_SMALL_TRACKING: f32 = 0.4;
/// Body Small の文字スタイル。
pub const BODY_SMALL: TypeStyle = TypeStyle {
    typeface: BODY_SMALL_TYPEFACE,
    weight: BODY_SMALL_WEIGHT,
    size: BODY_SMALL_SIZE,
    line_height: BODY_SMALL_LINE_HEIGHT,
    tracking: BODY_SMALL_TRACKING,
};

/// Display Large の書体。
pub const DISPLAY_LARGE_TYPEFACE: Typeface = Typeface::Brand;
/// Display Large の太さ。
pub const DISPLAY_LARGE_WEIGHT: u16 = WEIGHT_REGULAR;
/// Display Large の文字の大きさ（sp）。
pub const DISPLAY_LARGE_SIZE: f32 = 57.0;
/// Display Large の行の高さ（sp）。
pub const DISPLAY_LARGE_LINE_HEIGHT: f32 = 64.0;
/// Display Large の字間（sp）。
pub const DISPLAY_LARGE_TRACKING: f32 = -0.2;
/// Display Large の文字スタイル。
pub const DISPLAY_LARGE: TypeStyle = TypeStyle {
    typeface: DISPLAY_LARGE_TYPEFACE,
    weight: DISPLAY_LARGE_WEIGHT,
    size: DISPLAY_LARGE_SIZE,
    line_height: DISPLAY_LARGE_LINE_HEIGHT,
    tracking: DISPLAY_LARGE_TRACKING,
};

/// Display Medium の書体。
pub const DISPLAY_MEDIUM_TYPEFACE: Typeface = Typeface::Brand;
/// Display Medium の太さ。
pub const DISPLAY_MEDIUM_WEIGHT: u16 = WEIGHT_REGULAR;
/// Display Medium の文字の大きさ（sp）。
pub const DISPLAY_MEDIUM_SIZE: f32 = 45.0;
/// Display Medium の行の高さ（sp）。
pub const DISPLAY_MEDIUM_LINE_HEIGHT: f32 = 52.0;
/// Display Medium の字間（sp）。
pub const DISPLAY_MEDIUM_TRACKING: f32 = 0.0;
/// Display Medium の文字スタイル。
pub const DISPLAY_MEDIUM: TypeStyle = TypeStyle {
    typeface: DISPLAY_MEDIUM_TYPEFACE,
    weight: DISPLAY_MEDIUM_WEIGHT,
    size: DISPLAY_MEDIUM_SIZE,
    line_height: DISPLAY_MEDIUM_LINE_HEIGHT,
    tracking: DISPLAY_MEDIUM_TRACKING,
};

/// Display Small の書体。
pub const DISPLAY_SMALL_TYPEFACE: Typeface = Typeface::Brand;
/// Display Small の太さ。
pub const DISPLAY_SMALL_WEIGHT: u16 = WEIGHT_REGULAR;
/// Display Small の文字の大きさ（sp）。
pub const DISPLAY_SMALL_SIZE: f32 = 36.0;
/// Display Small の行の高さ（sp）。
pub const DISPLAY_SMALL_LINE_HEIGHT: f32 = 44.0;
/// Display Small の字間（sp）。
pub const DISPLAY_SMALL_TRACKING: f32 = 0.0;
/// Display Small の文字スタイル。
pub const DISPLAY_SMALL: TypeStyle = TypeStyle {
    typeface: DISPLAY_SMALL_TYPEFACE,
    weight: DISPLAY_SMALL_WEIGHT,
    size: DISPLAY_SMALL_SIZE,
    line_height: DISPLAY_SMALL_LINE_HEIGHT,
    tracking: DISPLAY_SMALL_TRACKING,
};

/// Headline Large の書体。
pub const HEADLINE_LARGE_TYPEFACE: Typeface = Typeface::Brand;
/// Headline Large の太さ。
pub const HEADLINE_LARGE_WEIGHT: u16 = WEIGHT_REGULAR;
/// Headline Large の文字の大きさ（sp）。
pub const HEADLINE_LARGE_SIZE: f32 = 32.0;
/// Headline Large の行の高さ（sp）。
pub const HEADLINE_LARGE_LINE_HEIGHT: f32 = 40.0;
/// Headline Large の字間（sp）。
pub const HEADLINE_LARGE_TRACKING: f32 = 0.0;
/// Headline Large の文字スタイル。
pub const HEADLINE_LARGE: TypeStyle = TypeStyle {
    typeface: HEADLINE_LARGE_TYPEFACE,
    weight: HEADLINE_LARGE_WEIGHT,
    size: HEADLINE_LARGE_SIZE,
    line_height: HEADLINE_LARGE_LINE_HEIGHT,
    tracking: HEADLINE_LARGE_TRACKING,
};

/// Headline Medium の書体。
pub const HEADLINE_MEDIUM_TYPEFACE: Typeface = Typeface::Brand;
/// Headline Medium の太さ。
pub const HEADLINE_MEDIUM_WEIGHT: u16 = WEIGHT_REGULAR;
/// Headline Medium の文字の大きさ（sp）。
pub const HEADLINE_MEDIUM_SIZE: f32 = 28.0;
/// Headline Medium の行の高さ（sp）。
pub const HEADLINE_MEDIUM_LINE_HEIGHT: f32 = 36.0;
/// Headline Medium の字間（sp）。
pub const HEADLINE_MEDIUM_TRACKING: f32 = 0.0;
/// Headline Medium の文字スタイル。
pub const HEADLINE_MEDIUM: TypeStyle = TypeStyle {
    typeface: HEADLINE_MEDIUM_TYPEFACE,
    weight: HEADLINE_MEDIUM_WEIGHT,
    size: HEADLINE_MEDIUM_SIZE,
    line_height: HEADLINE_MEDIUM_LINE_HEIGHT,
    tracking: HEADLINE_MEDIUM_TRACKING,
};

/// Headline Small の書体。
pub const HEADLINE_SMALL_TYPEFACE: Typeface = Typeface::Brand;
/// Headline Small の太さ。
pub const HEADLINE_SMALL_WEIGHT: u16 = WEIGHT_REGULAR;
/// Headline Small の文字の大きさ（sp）。
pub const HEADLINE_SMALL_SIZE: f32 = 24.0;
/// Headline Small の行の高さ（sp）。
pub const HEADLINE_SMALL_LINE_HEIGHT: f32 = 32.0;
/// Headline Small の字間（sp）。
pub const HEADLINE_SMALL_TRACKING: f32 = 0.0;
/// Headline Small の文字スタイル。
pub const HEADLINE_SMALL: TypeStyle = TypeStyle {
    typeface: HEADLINE_SMALL_TYPEFACE,
    weight: HEADLINE_SMALL_WEIGHT,
    size: HEADLINE_SMALL_SIZE,
    line_height: HEADLINE_SMALL_LINE_HEIGHT,
    tracking: HEADLINE_SMALL_TRACKING,
};

/// Label Large の書体。
pub const LABEL_LARGE_TYPEFACE: Typeface = Typeface::Plain;
/// Label Large の太さ。
pub const LABEL_LARGE_WEIGHT: u16 = WEIGHT_MEDIUM;
/// Label Large の文字の大きさ（sp）。
pub const LABEL_LARGE_SIZE: f32 = 14.0;
/// Label Large の行の高さ（sp）。
pub const LABEL_LARGE_LINE_HEIGHT: f32 = 20.0;
/// Label Large の字間（sp）。
pub const LABEL_LARGE_TRACKING: f32 = 0.1;
/// Label Large の文字スタイル。
pub const LABEL_LARGE: TypeStyle = TypeStyle {
    typeface: LABEL_LARGE_TYPEFACE,
    weight: LABEL_LARGE_WEIGHT,
    size: LABEL_LARGE_SIZE,
    line_height: LABEL_LARGE_LINE_HEIGHT,
    tracking: LABEL_LARGE_TRACKING,
};

/// Label Medium の書体。
pub const LABEL_MEDIUM_TYPEFACE: Typeface = Typeface::Plain;
/// Label Medium の太さ。
pub const LABEL_MEDIUM_WEIGHT: u16 = WEIGHT_MEDIUM;
/// Label Medium の文字の大きさ（sp）。
pub const LABEL_MEDIUM_SIZE: f32 = 12.0;
/// Label Medium の行の高さ（sp）。
pub const LABEL_MEDIUM_LINE_HEIGHT: f32 = 16.0;
/// Label Medium の字間（sp）。
pub const LABEL_MEDIUM_TRACKING: f32 = 0.5;
/// Label Medium の文字スタイル。
pub const LABEL_MEDIUM: TypeStyle = TypeStyle {
    typeface: LABEL_MEDIUM_TYPEFACE,
    weight: LABEL_MEDIUM_WEIGHT,
    size: LABEL_MEDIUM_SIZE,
    line_height: LABEL_MEDIUM_LINE_HEIGHT,
    tracking: LABEL_MEDIUM_TRACKING,
};

/// Label Small の書体。
pub const LABEL_SMALL_TYPEFACE: Typeface = Typeface::Plain;
/// Label Small の太さ。
pub const LABEL_SMALL_WEIGHT: u16 = WEIGHT_MEDIUM;
/// Label Small の文字の大きさ（sp）。
pub const LABEL_SMALL_SIZE: f32 = 11.0;
/// Label Small の行の高さ（sp）。
pub const LABEL_SMALL_LINE_HEIGHT: f32 = 16.0;
/// Label Small の字間（sp）。
pub const LABEL_SMALL_TRACKING: f32 = 0.5;
/// Label Small の文字スタイル。
pub const LABEL_SMALL: TypeStyle = TypeStyle {
    typeface: LABEL_SMALL_TYPEFACE,
    weight: LABEL_SMALL_WEIGHT,
    size: LABEL_SMALL_SIZE,
    line_height: LABEL_SMALL_LINE_HEIGHT,
    tracking: LABEL_SMALL_TRACKING,
};

/// Title Large の書体。
pub const TITLE_LARGE_TYPEFACE: Typeface = Typeface::Brand;
/// Title Large の太さ。
pub const TITLE_LARGE_WEIGHT: u16 = WEIGHT_REGULAR;
/// Title Large の文字の大きさ（sp）。
pub const TITLE_LARGE_SIZE: f32 = 22.0;
/// Title Large の行の高さ（sp）。
pub const TITLE_LARGE_LINE_HEIGHT: f32 = 28.0;
/// Title Large の字間（sp）。
pub const TITLE_LARGE_TRACKING: f32 = 0.0;
/// Title Large の文字スタイル。
pub const TITLE_LARGE: TypeStyle = TypeStyle {
    typeface: TITLE_LARGE_TYPEFACE,
    weight: TITLE_LARGE_WEIGHT,
    size: TITLE_LARGE_SIZE,
    line_height: TITLE_LARGE_LINE_HEIGHT,
    tracking: TITLE_LARGE_TRACKING,
};

/// Title Medium の書体。
pub const TITLE_MEDIUM_TYPEFACE: Typeface = Typeface::Plain;
/// Title Medium の太さ。
pub const TITLE_MEDIUM_WEIGHT: u16 = WEIGHT_MEDIUM;
/// Title Medium の文字の大きさ（sp）。
pub const TITLE_MEDIUM_SIZE: f32 = 16.0;
/// Title Medium の行の高さ（sp）。
pub const TITLE_MEDIUM_LINE_HEIGHT: f32 = 24.0;
/// Title Medium の字間（sp）。
pub const TITLE_MEDIUM_TRACKING: f32 = 0.2;
/// Title Medium の文字スタイル。
pub const TITLE_MEDIUM: TypeStyle = TypeStyle {
    typeface: TITLE_MEDIUM_TYPEFACE,
    weight: TITLE_MEDIUM_WEIGHT,
    size: TITLE_MEDIUM_SIZE,
    line_height: TITLE_MEDIUM_LINE_HEIGHT,
    tracking: TITLE_MEDIUM_TRACKING,
};

/// Title Small の書体。
pub const TITLE_SMALL_TYPEFACE: Typeface = Typeface::Plain;
/// Title Small の太さ。
pub const TITLE_SMALL_WEIGHT: u16 = WEIGHT_MEDIUM;
/// Title Small の文字の大きさ（sp）。
pub const TITLE_SMALL_SIZE: f32 = 14.0;
/// Title Small の行の高さ（sp）。
pub const TITLE_SMALL_LINE_HEIGHT: f32 = 20.0;
/// Title Small の字間（sp）。
pub const TITLE_SMALL_TRACKING: f32 = 0.1;
/// Title Small の文字スタイル。
pub const TITLE_SMALL: TypeStyle = TypeStyle {
    typeface: TITLE_SMALL_TYPEFACE,
    weight: TITLE_SMALL_WEIGHT,
    size: TITLE_SMALL_SIZE,
    line_height: TITLE_SMALL_LINE_HEIGHT,
    tracking: TITLE_SMALL_TRACKING,
};

/// タイプスケールの 15 種。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TypeScale {
    /// Display Large。
    DisplayLarge,
    /// Display Medium。
    DisplayMedium,
    /// Display Small。
    DisplaySmall,
    /// Headline Large。
    HeadlineLarge,
    /// Headline Medium。
    HeadlineMedium,
    /// Headline Small。
    HeadlineSmall,
    /// Title Large。
    TitleLarge,
    /// Title Medium。
    TitleMedium,
    /// Title Small。
    TitleSmall,
    /// Body Large。
    BodyLarge,
    /// Body Medium。
    BodyMedium,
    /// Body Small。
    BodySmall,
    /// Label Large。
    LabelLarge,
    /// Label Medium。
    LabelMedium,
    /// Label Small。
    LabelSmall,
}

impl TypeScale {
    /// すべての種類（大きい順）。
    pub const ALL: [TypeScale; 15] = [
        TypeScale::DisplayLarge,
        TypeScale::DisplayMedium,
        TypeScale::DisplaySmall,
        TypeScale::HeadlineLarge,
        TypeScale::HeadlineMedium,
        TypeScale::HeadlineSmall,
        TypeScale::TitleLarge,
        TypeScale::TitleMedium,
        TypeScale::TitleSmall,
        TypeScale::BodyLarge,
        TypeScale::BodyMedium,
        TypeScale::BodySmall,
        TypeScale::LabelLarge,
        TypeScale::LabelMedium,
        TypeScale::LabelSmall,
    ];

    /// 文字スタイル。
    pub const fn style(self) -> TypeStyle {
        match self {
            TypeScale::DisplayLarge => DISPLAY_LARGE,
            TypeScale::DisplayMedium => DISPLAY_MEDIUM,
            TypeScale::DisplaySmall => DISPLAY_SMALL,
            TypeScale::HeadlineLarge => HEADLINE_LARGE,
            TypeScale::HeadlineMedium => HEADLINE_MEDIUM,
            TypeScale::HeadlineSmall => HEADLINE_SMALL,
            TypeScale::TitleLarge => TITLE_LARGE,
            TypeScale::TitleMedium => TITLE_MEDIUM,
            TypeScale::TitleSmall => TITLE_SMALL,
            TypeScale::BodyLarge => BODY_LARGE,
            TypeScale::BodyMedium => BODY_MEDIUM,
            TypeScale::BodySmall => BODY_SMALL,
            TypeScale::LabelLarge => LABEL_LARGE,
            TypeScale::LabelMedium => LABEL_MEDIUM,
            TypeScale::LabelSmall => LABEL_SMALL,
        }
    }
}
