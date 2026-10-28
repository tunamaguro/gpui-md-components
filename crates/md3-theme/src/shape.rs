//! 角丸の gpui 向け表現。

use gpui::{Pixels, px};
use md3_tokens::shape::{self as tokens, Shape};

/// 角丸の半径。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CornerRadius {
    /// 固定の半径。
    Fixed(Pixels),
    /// 短辺の半分（完全な丸）。
    Full,
}

impl CornerRadius {
    /// 外側に `offset` だけ広げた角丸（フォーカスリングなど）。
    pub fn grow(self, offset: Pixels) -> Self {
        match self {
            CornerRadius::Fixed(radius) => CornerRadius::Fixed(radius + offset),
            CornerRadius::Full => CornerRadius::Full,
        }
    }
}

/// 角丸スケールの各段階の半径。利用者が上書きできる。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shapes {
    /// extra-small。
    pub extra_small: Pixels,
    /// small。
    pub small: Pixels,
    /// medium。
    pub medium: Pixels,
    /// large。
    pub large: Pixels,
    /// extra-large。
    pub extra_large: Pixels,
}

impl Default for Shapes {
    fn default() -> Self {
        Self {
            extra_small: px(tokens::CORNER_EXTRA_SMALL),
            small: px(tokens::CORNER_SMALL),
            medium: px(tokens::CORNER_MEDIUM),
            large: px(tokens::CORNER_LARGE),
            extra_large: px(tokens::CORNER_EXTRA_LARGE),
        }
    }
}

impl Shapes {
    /// 段階に対応する半径。
    pub fn corner(&self, shape: Shape) -> CornerRadius {
        match shape {
            Shape::None => CornerRadius::Fixed(px(tokens::CORNER_NONE)),
            Shape::ExtraSmall => CornerRadius::Fixed(self.extra_small),
            Shape::Small => CornerRadius::Fixed(self.small),
            Shape::Medium => CornerRadius::Fixed(self.medium),
            Shape::Large => CornerRadius::Fixed(self.large),
            Shape::ExtraLarge => CornerRadius::Fixed(self.extra_large),
            Shape::Full => CornerRadius::Full,
        }
    }
}
