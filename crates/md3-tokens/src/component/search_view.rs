// Ported from Jetpack Compose: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/SearchViewTokens.kt @ b8256a52 (tokens VERSION v0_210). See NOTICE.

//! `SearchViewTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::elevation;
use crate::shape::Shape;
use crate::typography::TypeScale;
use md3_color::ColorRole;

/// `ContainerColor`
pub const CONTAINER_COLOR: ColorRole = ColorRole::SurfaceContainerHigh;
/// `ContainerElevation`
pub const CONTAINER_ELEVATION: f32 = elevation::LEVEL3;
/// `DividerColor`
pub const DIVIDER_COLOR: ColorRole = ColorRole::Outline;
/// `DockedContainerShape`
pub const DOCKED_CONTAINER_SHAPE: Shape = Shape::ExtraLarge;
/// `DockedHeaderContainerHeight`
pub const DOCKED_HEADER_CONTAINER_HEIGHT: f32 = 56.0;
/// `FullScreenContainerShape`
pub const FULL_SCREEN_CONTAINER_SHAPE: Shape = Shape::None;
/// `FullScreenHeaderContainerHeight`
pub const FULL_SCREEN_HEADER_CONTAINER_HEIGHT: f32 = 72.0;
/// `HeaderInputTextColor`
pub const HEADER_INPUT_TEXT_COLOR: ColorRole = ColorRole::OnSurface;
/// `HeaderInputTextFont`
pub const HEADER_INPUT_TEXT_FONT: TypeScale = TypeScale::BodyLarge;
/// `HeaderLeadingIconColor`
pub const HEADER_LEADING_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `HeaderSupportingTextColor`
pub const HEADER_SUPPORTING_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `HeaderSupportingTextFont`
pub const HEADER_SUPPORTING_TEXT_FONT: TypeScale = TypeScale::BodyLarge;
/// `HeaderTrailingIconColor`
pub const HEADER_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
