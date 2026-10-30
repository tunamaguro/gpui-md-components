// Ported from Jetpack Compose: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/NavigationDrawerTokens.kt @ b8256a52 (tokens VERSION v0_210). See NOTICE.

//! `NavigationDrawerTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::elevation;
use crate::shape::Shape;
use crate::typography::TypeScale;
use md3_color::ColorRole;

/// `ActiveFocusIconColor`
pub const ACTIVE_FOCUS_ICON_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `ActiveFocusLabelTextColor`
pub const ACTIVE_FOCUS_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `ActiveHoverIconColor`
pub const ACTIVE_HOVER_ICON_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `ActiveHoverLabelTextColor`
pub const ACTIVE_HOVER_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `ActiveIconColor`
pub const ACTIVE_ICON_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `ActiveIndicatorColor`
pub const ACTIVE_INDICATOR_COLOR: ColorRole = ColorRole::SecondaryContainer;
/// `ActiveIndicatorHeight`
pub const ACTIVE_INDICATOR_HEIGHT: f32 = 56.0;
/// `ActiveIndicatorShape`
pub const ACTIVE_INDICATOR_SHAPE: Shape = Shape::Full;
/// `ActiveIndicatorWidth`
pub const ACTIVE_INDICATOR_WIDTH: f32 = 336.0;
/// `ActiveLabelTextColor`
pub const ACTIVE_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `ActivePressedIconColor`
pub const ACTIVE_PRESSED_ICON_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `ActivePressedLabelTextColor`
pub const ACTIVE_PRESSED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `BottomContainerShape`（上側の 2 つの角だけに適用する）
pub const BOTTOM_CONTAINER_SHAPE: Shape = Shape::Large;
/// `ContainerHeightPercent`
pub const CONTAINER_HEIGHT_PERCENT: f32 = 100.0;
/// `ContainerShape`（終端側（左から右に書く言語では右）の 2 つの角だけに適用する）
pub const CONTAINER_SHAPE: Shape = Shape::Large;
/// `ContainerWidth`
pub const CONTAINER_WIDTH: f32 = 360.0;
/// `FocusIndicatorColor`
pub const FOCUS_INDICATOR_COLOR: ColorRole = ColorRole::Secondary;
/// `HeadlineColor`
pub const HEADLINE_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `HeadlineFont`
pub const HEADLINE_FONT: TypeScale = TypeScale::TitleSmall;
/// `IconSize`
pub const ICON_SIZE: f32 = 24.0;
/// `InactiveFocusIconColor`
pub const INACTIVE_FOCUS_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `InactiveFocusLabelTextColor`
pub const INACTIVE_FOCUS_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurface;
/// `InactiveHoverIconColor`
pub const INACTIVE_HOVER_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `InactiveHoverLabelTextColor`
pub const INACTIVE_HOVER_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurface;
/// `InactiveIconColor`
pub const INACTIVE_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `InactiveLabelTextColor`
pub const INACTIVE_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `InactivePressedIconColor`
pub const INACTIVE_PRESSED_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `InactivePressedLabelTextColor`
pub const INACTIVE_PRESSED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurface;
/// `LabelTextFont`
pub const LABEL_TEXT_FONT: TypeScale = TypeScale::LabelLarge;
/// `LargeBadgeLabelColor`
pub const LARGE_BADGE_LABEL_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `LargeBadgeLabelFont`
pub const LARGE_BADGE_LABEL_FONT: TypeScale = TypeScale::LabelLarge;
/// `ModalContainerColor`
pub const MODAL_CONTAINER_COLOR: ColorRole = ColorRole::SurfaceContainerLow;
/// `ModalContainerElevation`
pub const MODAL_CONTAINER_ELEVATION: f32 = elevation::LEVEL1;
/// `StandardContainerColor`
pub const STANDARD_CONTAINER_COLOR: ColorRole = ColorRole::Surface;
/// `StandardContainerElevation`
pub const STANDARD_CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
