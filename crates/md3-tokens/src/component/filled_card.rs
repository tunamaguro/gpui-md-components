// Ported from Jetpack Compose: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/FilledCardTokens.kt @ b8256a52 (tokens VERSION v0_210). See NOTICE.

//! `FilledCardTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::elevation;
use crate::shape::Shape;
use md3_color::ColorRole;

/// `ContainerColor`
pub const CONTAINER_COLOR: ColorRole = ColorRole::SurfaceContainerHighest;
/// `ContainerElevation`
pub const CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `ContainerShape`
pub const CONTAINER_SHAPE: Shape = Shape::Medium;
/// `DisabledContainerColor`
pub const DISABLED_CONTAINER_COLOR: ColorRole = ColorRole::SurfaceVariant;
/// `DisabledContainerElevation`
pub const DISABLED_CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `DisabledContainerOpacity`
pub const DISABLED_CONTAINER_OPACITY: f32 = 0.38;
/// `DraggedContainerElevation`
pub const DRAGGED_CONTAINER_ELEVATION: f32 = elevation::LEVEL3;
/// `FocusContainerElevation`
pub const FOCUS_CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `FocusIndicatorColor`
pub const FOCUS_INDICATOR_COLOR: ColorRole = ColorRole::Secondary;
/// `HoverContainerElevation`
pub const HOVER_CONTAINER_ELEVATION: f32 = elevation::LEVEL1;
/// `IconColor`
pub const ICON_COLOR: ColorRole = ColorRole::Primary;
/// `IconSize`
pub const ICON_SIZE: f32 = 24.0;
/// `PressedContainerElevation`
pub const PRESSED_CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
