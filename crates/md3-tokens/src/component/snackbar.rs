// Ported from Jetpack Compose: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/SnackbarTokens.kt @ b8256a52 (tokens VERSION v0_103). See NOTICE.

//! `SnackbarTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::elevation;
use crate::shape::Shape;
use crate::typography::TypeScale;
use md3_color::ColorRole;

/// `ActionFocusLabelTextColor`
pub const ACTION_FOCUS_LABEL_TEXT_COLOR: ColorRole = ColorRole::InversePrimary;
/// `ActionHoverLabelTextColor`
pub const ACTION_HOVER_LABEL_TEXT_COLOR: ColorRole = ColorRole::InversePrimary;
/// `ActionLabelTextColor`
pub const ACTION_LABEL_TEXT_COLOR: ColorRole = ColorRole::InversePrimary;
/// `ActionLabelTextFont`
pub const ACTION_LABEL_TEXT_FONT: TypeScale = TypeScale::LabelLarge;
/// `ActionPressedLabelTextColor`
pub const ACTION_PRESSED_LABEL_TEXT_COLOR: ColorRole = ColorRole::InversePrimary;
/// `ContainerColor`
pub const CONTAINER_COLOR: ColorRole = ColorRole::InverseSurface;
/// `ContainerElevation`
pub const CONTAINER_ELEVATION: f32 = elevation::LEVEL3;
/// `ContainerShape`
pub const CONTAINER_SHAPE: Shape = Shape::ExtraSmall;
/// `IconColor`
pub const ICON_COLOR: ColorRole = ColorRole::InverseOnSurface;
/// `FocusIconColor`
pub const FOCUS_ICON_COLOR: ColorRole = ColorRole::InverseOnSurface;
/// `HoverIconColor`
pub const HOVER_ICON_COLOR: ColorRole = ColorRole::InverseOnSurface;
/// `PressedIconColor`
pub const PRESSED_ICON_COLOR: ColorRole = ColorRole::InverseOnSurface;
/// `IconSize`
pub const ICON_SIZE: f32 = 24.0;
/// `SupportingTextColor`
pub const SUPPORTING_TEXT_COLOR: ColorRole = ColorRole::InverseOnSurface;
/// `SupportingTextFont`
pub const SUPPORTING_TEXT_FONT: TypeScale = TypeScale::BodyMedium;
/// `SingleLineContainerHeight`
pub const SINGLE_LINE_CONTAINER_HEIGHT: f32 = 48.0;
/// `TwoLinesContainerHeight`
pub const TWO_LINES_CONTAINER_HEIGHT: f32 = 68.0;
