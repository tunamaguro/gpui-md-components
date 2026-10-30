// Ported from Jetpack Compose: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/SecondaryNavigationTabTokens.kt @ b8256a52 (tokens VERSION v0_162). See NOTICE.

//! `SecondaryNavigationTabTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::elevation;
use crate::shape::Shape;
use crate::typography::TypeScale;
use md3_color::ColorRole;

/// `ActiveLabelTextColor`
pub const ACTIVE_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurface;
/// `ContainerColor`
pub const CONTAINER_COLOR: ColorRole = ColorRole::Surface;
/// `ContainerElevation`
pub const CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `ContainerHeight`
pub const CONTAINER_HEIGHT: f32 = 48.0;
/// `ContainerShape`
pub const CONTAINER_SHAPE: Shape = Shape::None;
/// `DividerColor`
pub const DIVIDER_COLOR: ColorRole = ColorRole::SurfaceVariant;
/// `DividerHeight`
pub const DIVIDER_HEIGHT: f32 = 1.0;
/// `FocusLabelTextColor`
pub const FOCUS_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurface;
/// `HoverLabelTextColor`
pub const HOVER_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurface;
/// `InactiveLabelTextColor`
pub const INACTIVE_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `LabelTextFont`
pub const LABEL_TEXT_FONT: TypeScale = TypeScale::TitleSmall;
/// `PressedLabelTextColor`
pub const PRESSED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurface;
/// `ActiveIconColor`
pub const ACTIVE_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `FocusIconColor`
pub const FOCUS_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `HoverIconColor`
pub const HOVER_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `IconSize`
pub const ICON_SIZE: f32 = 24.0;
/// `InactiveIconColor`
pub const INACTIVE_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `PressedIconColor`
pub const PRESSED_ICON_COLOR: ColorRole = ColorRole::OnSurface;
