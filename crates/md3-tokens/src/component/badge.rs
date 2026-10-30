// Ported from Jetpack Compose: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/BadgeTokens.kt @ b8256a52 (tokens VERSION v0_103). See NOTICE.

//! `BadgeTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::shape::Shape;
use crate::typography::TypeScale;
use md3_color::ColorRole;

/// `Color`
pub const COLOR: ColorRole = ColorRole::Error;
/// `LargeColor`
pub const LARGE_COLOR: ColorRole = ColorRole::Error;
/// `LargeLabelTextColor`
pub const LARGE_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnError;
/// `LargeLabelTextFont`
pub const LARGE_LABEL_TEXT_FONT: TypeScale = TypeScale::LabelSmall;
/// `LargeShape`
pub const LARGE_SHAPE: Shape = Shape::Full;
/// `LargeSize`
pub const LARGE_SIZE: f32 = 16.0;
/// `Shape`
pub const SHAPE: Shape = Shape::Full;
/// `Size`
pub const SIZE: f32 = 6.0;
