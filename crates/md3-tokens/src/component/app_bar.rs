// Ported from Jetpack Compose: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/AppBarTokens.kt @ b8256a52 (tokens VERSION 14_0_0). See NOTICE.

//! `AppBarTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::elevation;
use crate::shape::Shape;
use md3_color::ColorRole;

/// `AvatarSize`
pub const AVATAR_SIZE: f32 = 32.0;
/// `ContainerColor`
pub const CONTAINER_COLOR: ColorRole = ColorRole::Surface;
/// `ContainerElevation`
pub const CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `ContainerShape`
pub const CONTAINER_SHAPE: Shape = Shape::None;
/// `IconButtonSpace`
pub const ICON_BUTTON_SPACE: f32 = 0.0;
/// `IconSize`
pub const ICON_SIZE: f32 = 24.0;
/// `LeadingIconColor`
pub const LEADING_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `LeadingSpace`
pub const LEADING_SPACE: f32 = 4.0;
/// `OnScrollContainerColor`
pub const ON_SCROLL_CONTAINER_COLOR: ColorRole = ColorRole::SurfaceContainer;
/// `OnScrollContainerElevation`
pub const ON_SCROLL_CONTAINER_ELEVATION: f32 = elevation::LEVEL2;
/// `SubtitleColor`
pub const SUBTITLE_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `TitleColor`
pub const TITLE_COLOR: ColorRole = ColorRole::OnSurface;
/// `TrailingIconColor`
pub const TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `TrailingSpace`
pub const TRAILING_SPACE: f32 = 4.0;
