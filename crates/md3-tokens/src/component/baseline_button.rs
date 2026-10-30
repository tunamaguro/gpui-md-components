// Ported from Jetpack Compose: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/BaselineButtonTokens.kt @ b8256a52 (tokens VERSION v0_11_0). See NOTICE.

//! `BaselineButtonTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::elevation;
use crate::shape::Shape;
use md3_color::ColorRole;

/// `ContainerColor`
pub const CONTAINER_COLOR: ColorRole = ColorRole::Primary;
/// `ContainerElevation`
pub const CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `ContainerHeight`
pub const CONTAINER_HEIGHT: f32 = 40.0;
/// `ContainerShapeRound`
pub const CONTAINER_SHAPE_ROUND: Shape = Shape::Full;
/// `ContainerShapeSquare`
pub const CONTAINER_SHAPE_SQUARE: Shape = Shape::Medium;
/// `DisabledContainerColor`
pub const DISABLED_CONTAINER_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledContainerElevation`
pub const DISABLED_CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `DisabledContainerOpacity`
pub const DISABLED_CONTAINER_OPACITY: f32 = 0.1;
/// `DisabledIconColor`
pub const DISABLED_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `DisabledIconOpacity`
pub const DISABLED_ICON_OPACITY: f32 = 0.38;
/// `DisabledLabelTextColor`
pub const DISABLED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `DisabledLabelTextOpacity`
pub const DISABLED_LABEL_TEXT_OPACITY: f32 = 0.38;
/// `FocusedContainerElevation`
pub const FOCUSED_CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `FocusedIconColor`
pub const FOCUSED_ICON_COLOR: ColorRole = ColorRole::OnPrimary;
/// `FocusedLabelTextColor`
pub const FOCUSED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnPrimary;
/// `HoveredContainerElevation`
pub const HOVERED_CONTAINER_ELEVATION: f32 = elevation::LEVEL1;
/// `HoveredIconColor`
pub const HOVERED_ICON_COLOR: ColorRole = ColorRole::OnPrimary;
/// `HoveredLabelTextColor`
pub const HOVERED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnPrimary;
/// `IconColor`
pub const ICON_COLOR: ColorRole = ColorRole::OnPrimary;
/// `IconLabelSpace`
pub const ICON_LABEL_SPACE: f32 = 8.0;
/// `IconSize`
pub const ICON_SIZE: f32 = 20.0;
/// `LabelTextColor`
pub const LABEL_TEXT_COLOR: ColorRole = ColorRole::OnPrimary;
/// `LabelTextSelectedColor`
pub const LABEL_TEXT_SELECTED_COLOR: ColorRole = ColorRole::OnPrimary;
/// `LabelTextUnselectedColor`
pub const LABEL_TEXT_UNSELECTED_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `LeadingSpace`
pub const LEADING_SPACE: f32 = 24.0;
/// `PressedContainerElevation`
pub const PRESSED_CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `PressedContainerShape`
pub const PRESSED_CONTAINER_SHAPE: Shape = Shape::Small;
/// `PressedIconColor`
pub const PRESSED_ICON_COLOR: ColorRole = ColorRole::OnPrimary;
/// `PressedLabelTextColor`
pub const PRESSED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnPrimary;
/// `SelectedContainerColor`
pub const SELECTED_CONTAINER_COLOR: ColorRole = ColorRole::Primary;
/// `SelectedContainerShapeRound`
pub const SELECTED_CONTAINER_SHAPE_ROUND: Shape = Shape::Full;
/// `SelectedContainerShapeSquare`
pub const SELECTED_CONTAINER_SHAPE_SQUARE: Shape = Shape::Medium;
/// `SelectedFocusedIconColor`
pub const SELECTED_FOCUSED_ICON_COLOR: ColorRole = ColorRole::OnPrimary;
/// `SelectedFocusedLabelTextColor`
pub const SELECTED_FOCUSED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnPrimary;
/// `SelectedHoveredIconColor`
pub const SELECTED_HOVERED_ICON_COLOR: ColorRole = ColorRole::OnPrimary;
/// `SelectedHoveredLabelTextColor`
pub const SELECTED_HOVERED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnPrimary;
/// `SelectedIconColor`
pub const SELECTED_ICON_COLOR: ColorRole = ColorRole::OnPrimary;
/// `SelectedPressedIconColor`
pub const SELECTED_PRESSED_ICON_COLOR: ColorRole = ColorRole::OnPrimary;
/// `SelectedPressedLabelTextColor`
pub const SELECTED_PRESSED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnPrimary;
/// `TrailingSpace`
pub const TRAILING_SPACE: f32 = 24.0;
/// `UnselectedContainerColor`
pub const UNSELECTED_CONTAINER_COLOR: ColorRole = ColorRole::SurfaceContainer;
/// `UnselectedFocusedIconColor`
pub const UNSELECTED_FOCUSED_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedFocusedLabelTextColor`
pub const UNSELECTED_FOCUSED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedHoveredIconColor`
pub const UNSELECTED_HOVERED_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedHoveredLabelTextColor`
pub const UNSELECTED_HOVERED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedIconColor`
pub const UNSELECTED_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedPressedIconColor`
pub const UNSELECTED_PRESSED_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedPressedLabelTextColor`
pub const UNSELECTED_PRESSED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
