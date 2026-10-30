// Ported from Jetpack Compose: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/CheckboxTokens.kt @ b8256a52 (tokens VERSION 14_1_0). See NOTICE.

//! `CheckboxTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::shape::{CornerSize, Shape};
use md3_color::ColorRole;

/// `ContainerShape`
pub const CONTAINER_SHAPE: CornerSize = CornerSize::Fixed(2.0);
/// `ContainerSize`
pub const CONTAINER_SIZE: f32 = 18.0;
/// `FocusIndicatorColor`
pub const FOCUS_INDICATOR_COLOR: ColorRole = ColorRole::Secondary;
/// `IconSize`
pub const ICON_SIZE: f32 = 18.0;
/// `SelectedContainerColor`
pub const SELECTED_CONTAINER_COLOR: ColorRole = ColorRole::Primary;
/// `SelectedDisabledContainerColor`
pub const SELECTED_DISABLED_CONTAINER_COLOR: ColorRole = ColorRole::OnSurface;
/// `SelectedDisabledContainerOpacity`
pub const SELECTED_DISABLED_CONTAINER_OPACITY: f32 = 0.38;
/// `SelectedDisabledContainerOutlineWidth`
pub const SELECTED_DISABLED_CONTAINER_OUTLINE_WIDTH: f32 = 0.0;
/// `SelectedDisabledIconColor`
pub const SELECTED_DISABLED_ICON_COLOR: ColorRole = ColorRole::Surface;
/// `SelectedErrorContainerColor`
pub const SELECTED_ERROR_CONTAINER_COLOR: ColorRole = ColorRole::Error;
/// `SelectedErrorFocusContainerColor`
pub const SELECTED_ERROR_FOCUS_CONTAINER_COLOR: ColorRole = ColorRole::Error;
/// `SelectedErrorFocusIconColor`
pub const SELECTED_ERROR_FOCUS_ICON_COLOR: ColorRole = ColorRole::OnError;
/// `SelectedErrorHoverContainerColor`
pub const SELECTED_ERROR_HOVER_CONTAINER_COLOR: ColorRole = ColorRole::Error;
/// `SelectedErrorHoverIconColor`
pub const SELECTED_ERROR_HOVER_ICON_COLOR: ColorRole = ColorRole::OnError;
/// `SelectedErrorIconColor`
pub const SELECTED_ERROR_ICON_COLOR: ColorRole = ColorRole::OnError;
/// `SelectedErrorPressedContainerColor`
pub const SELECTED_ERROR_PRESSED_CONTAINER_COLOR: ColorRole = ColorRole::Error;
/// `SelectedErrorPressedIconColor`
pub const SELECTED_ERROR_PRESSED_ICON_COLOR: ColorRole = ColorRole::OnError;
/// `SelectedFocusContainerColor`
pub const SELECTED_FOCUS_CONTAINER_COLOR: ColorRole = ColorRole::Primary;
/// `SelectedFocusIconColor`
pub const SELECTED_FOCUS_ICON_COLOR: ColorRole = ColorRole::OnPrimary;
/// `SelectedFocusOutlineWidth`
pub const SELECTED_FOCUS_OUTLINE_WIDTH: f32 = 0.0;
/// `SelectedHoverContainerColor`
pub const SELECTED_HOVER_CONTAINER_COLOR: ColorRole = ColorRole::Primary;
/// `SelectedHoverIconColor`
pub const SELECTED_HOVER_ICON_COLOR: ColorRole = ColorRole::OnPrimary;
/// `SelectedHoverOutlineWidth`
pub const SELECTED_HOVER_OUTLINE_WIDTH: f32 = 0.0;
/// `SelectedIconColor`
pub const SELECTED_ICON_COLOR: ColorRole = ColorRole::OnPrimary;
/// `SelectedOutlineWidth`
pub const SELECTED_OUTLINE_WIDTH: f32 = 0.0;
/// `SelectedPressedContainerColor`
pub const SELECTED_PRESSED_CONTAINER_COLOR: ColorRole = ColorRole::Primary;
/// `SelectedPressedIconColor`
pub const SELECTED_PRESSED_ICON_COLOR: ColorRole = ColorRole::OnPrimary;
/// `SelectedPressedOutlineWidth`
pub const SELECTED_PRESSED_OUTLINE_WIDTH: f32 = 0.0;
/// `StateLayerShape`
pub const STATE_LAYER_SHAPE: Shape = Shape::Full;
/// `StateLayerSize`
pub const STATE_LAYER_SIZE: f32 = 40.0;
/// `UnselectedDisabledContainerOpacity`
pub const UNSELECTED_DISABLED_CONTAINER_OPACITY: f32 = 0.38;
/// `UnselectedDisabledOutlineColor`
pub const UNSELECTED_DISABLED_OUTLINE_COLOR: ColorRole = ColorRole::OnSurface;
/// `UnselectedDisabledOutlineWidth`
pub const UNSELECTED_DISABLED_OUTLINE_WIDTH: f32 = 2.0;
/// `UnselectedErrorFocusOutlineColor`
pub const UNSELECTED_ERROR_FOCUS_OUTLINE_COLOR: ColorRole = ColorRole::Error;
/// `UnselectedErrorHoverOutlineColor`
pub const UNSELECTED_ERROR_HOVER_OUTLINE_COLOR: ColorRole = ColorRole::Error;
/// `UnselectedErrorOutlineColor`
pub const UNSELECTED_ERROR_OUTLINE_COLOR: ColorRole = ColorRole::Error;
/// `UnselectedErrorPressedOutlineColor`
pub const UNSELECTED_ERROR_PRESSED_OUTLINE_COLOR: ColorRole = ColorRole::Error;
/// `UnselectedFocusOutlineColor`
pub const UNSELECTED_FOCUS_OUTLINE_COLOR: ColorRole = ColorRole::OnSurface;
/// `UnselectedFocusOutlineWidth`
pub const UNSELECTED_FOCUS_OUTLINE_WIDTH: f32 = 2.0;
/// `UnselectedHoverOutlineColor`
pub const UNSELECTED_HOVER_OUTLINE_COLOR: ColorRole = ColorRole::OnSurface;
/// `UnselectedHoverOutlineWidth`
pub const UNSELECTED_HOVER_OUTLINE_WIDTH: f32 = 2.0;
/// `UnselectedOutlineColor`
pub const UNSELECTED_OUTLINE_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedOutlineWidth`
pub const UNSELECTED_OUTLINE_WIDTH: f32 = 2.0;
/// `UnselectedPressedOutlineColor`
pub const UNSELECTED_PRESSED_OUTLINE_COLOR: ColorRole = ColorRole::OnSurface;
/// `UnselectedPressedOutlineWidth`
pub const UNSELECTED_PRESSED_OUTLINE_WIDTH: f32 = 2.0;
