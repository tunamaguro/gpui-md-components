// Ported from Jetpack Compose: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/ExtendedFabPrimaryTokens.kt @ b8256a52 (tokens VERSION v0_103). See NOTICE.

//! `ExtendedFabPrimaryTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::elevation;
use crate::shape::Shape;
use crate::typography::TypeScale;
use md3_color::ColorRole;

/// `ContainerColor`
pub const CONTAINER_COLOR: ColorRole = ColorRole::PrimaryContainer;
/// `ContainerElevation`
pub const CONTAINER_ELEVATION: f32 = elevation::LEVEL3;
/// `ContainerHeight`
pub const CONTAINER_HEIGHT: f32 = 56.0;
/// `ContainerShape`
pub const CONTAINER_SHAPE: Shape = Shape::Large;
/// `FocusContainerElevation`
pub const FOCUS_CONTAINER_ELEVATION: f32 = elevation::LEVEL3;
/// `FocusIconColor`
pub const FOCUS_ICON_COLOR: ColorRole = ColorRole::OnPrimaryContainer;
/// `FocusLabelTextColor`
pub const FOCUS_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnPrimaryContainer;
/// `HoverContainerElevation`
pub const HOVER_CONTAINER_ELEVATION: f32 = elevation::LEVEL4;
/// `HoverIconColor`
pub const HOVER_ICON_COLOR: ColorRole = ColorRole::OnPrimaryContainer;
/// `HoverLabelTextColor`
pub const HOVER_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnPrimaryContainer;
/// `IconColor`
pub const ICON_COLOR: ColorRole = ColorRole::OnPrimaryContainer;
/// `IconSize`
pub const ICON_SIZE: f32 = 24.0;
/// `LabelTextColor`
pub const LABEL_TEXT_COLOR: ColorRole = ColorRole::OnPrimaryContainer;
/// `LabelTextFont`
pub const LABEL_TEXT_FONT: TypeScale = TypeScale::LabelLarge;
/// `LoweredContainerElevation`
pub const LOWERED_CONTAINER_ELEVATION: f32 = elevation::LEVEL1;
/// `LoweredFocusContainerElevation`
pub const LOWERED_FOCUS_CONTAINER_ELEVATION: f32 = elevation::LEVEL1;
/// `LoweredHoverContainerElevation`
pub const LOWERED_HOVER_CONTAINER_ELEVATION: f32 = elevation::LEVEL2;
/// `LoweredPressedContainerElevation`
pub const LOWERED_PRESSED_CONTAINER_ELEVATION: f32 = elevation::LEVEL1;
/// `PressedContainerElevation`
pub const PRESSED_CONTAINER_ELEVATION: f32 = elevation::LEVEL3;
/// `PressedIconColor`
pub const PRESSED_ICON_COLOR: ColorRole = ColorRole::OnPrimaryContainer;
/// `PressedLabelTextColor`
pub const PRESSED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnPrimaryContainer;
