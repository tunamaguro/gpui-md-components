//! テーマの値を gpui のスタイルに当てる補助。

use gpui::Styled;
use md3_theme::{CornerRadius, TypeScaleStyle};

/// テーマの値をスタイルに当てる拡張トレイト。
pub trait StyledExt: Styled + Sized {
    /// 角丸を当てる。
    fn corner_radius(self, radius: CornerRadius) -> Self {
        match radius {
            CornerRadius::Fixed(radius) => self.rounded(radius),
            CornerRadius::Full => self.rounded_full(),
        }
    }

    /// 文字スタイル（大きさ・行の高さ・太さ・フォント）を当てる。
    ///
    /// gpui は字間の指定に対応していないため、`tracking` は当てない。
    fn type_style(self, style: &TypeScaleStyle) -> Self {
        let this = self
            .text_size(style.font_size)
            .line_height(style.line_height)
            .font_weight(style.font_weight);
        match &style.font_family {
            Some(family) => this.font_family(family.clone()),
            None => this,
        }
    }
}

impl<E: Styled> StyledExt for E {}
