// Ported from Jetpack Compose: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/FilledTonalButtonTokens.kt @ b8256a52 (tokens VERSION v0_103). See NOTICE.

//! `FilledTonalButtonTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::elevation;
use crate::shape::Shape;
use crate::typography::TypeScale;
use md3_color::ColorRole;

/// `ContainerColor`
pub const CONTAINER_COLOR: ColorRole = ColorRole::SecondaryContainer;
/// `ContainerElevation`
pub const CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `ContainerHeight`
pub const CONTAINER_HEIGHT: f32 = 40.0;
/// `ContainerShape`
pub const CONTAINER_SHAPE: Shape = Shape::Full;
/// `DisabledContainerColor`
pub const DISABLED_CONTAINER_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledContainerElevation`
pub const DISABLED_CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `DisabledContainerOpacity`
pub const DISABLED_CONTAINER_OPACITY: f32 = 0.12;
/// `DisabledLabelTextColor`
pub const DISABLED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledLabelTextOpacity`
pub const DISABLED_LABEL_TEXT_OPACITY: f32 = 0.38;
/// `FocusContainerElevation`
pub const FOCUS_CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `FocusLabelTextColor`
pub const FOCUS_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `HoverContainerElevation`
pub const HOVER_CONTAINER_ELEVATION: f32 = elevation::LEVEL1;
/// `HoverLabelTextColor`
pub const HOVER_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `LabelTextColor`
pub const LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `LabelTextFont`
pub const LABEL_TEXT_FONT: TypeScale = TypeScale::LabelLarge;
/// `PressedContainerElevation`
pub const PRESSED_CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `PressedLabelTextColor`
pub const PRESSED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `DisabledIconColor`
pub const DISABLED_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledIconOpacity`
pub const DISABLED_ICON_OPACITY: f32 = 0.38;
/// `FocusIconColor`
pub const FOCUS_ICON_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `HoverIconColor`
pub const HOVER_ICON_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `IconColor`
pub const ICON_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `IconSize`
pub const ICON_SIZE: f32 = 18.0;
/// `PressedIconColor`
pub const PRESSED_ICON_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
