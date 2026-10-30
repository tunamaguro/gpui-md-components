// Ported from Jetpack Compose: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/SwitchTokens.kt @ b8256a52 (tokens VERSION v0_210). See NOTICE.

//! `SwitchTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::shape::Shape;
use md3_color::ColorRole;

/// `DisabledSelectedHandleColor`
pub const DISABLED_SELECTED_HANDLE_COLOR: ColorRole = ColorRole::Surface;
/// `DisabledSelectedHandleOpacity`
pub const DISABLED_SELECTED_HANDLE_OPACITY: f32 = 1.0;
/// `DisabledSelectedIconColor`
pub const DISABLED_SELECTED_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledSelectedIconOpacity`
pub const DISABLED_SELECTED_ICON_OPACITY: f32 = 0.38;
/// `DisabledSelectedTrackColor`
pub const DISABLED_SELECTED_TRACK_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledTrackOpacity`
pub const DISABLED_TRACK_OPACITY: f32 = 0.12;
/// `DisabledUnselectedHandleColor`
pub const DISABLED_UNSELECTED_HANDLE_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledUnselectedHandleOpacity`
pub const DISABLED_UNSELECTED_HANDLE_OPACITY: f32 = 0.38;
/// `DisabledUnselectedIconColor`
pub const DISABLED_UNSELECTED_ICON_COLOR: ColorRole = ColorRole::SurfaceContainerHighest;
/// `DisabledUnselectedIconOpacity`
pub const DISABLED_UNSELECTED_ICON_OPACITY: f32 = 0.38;
/// `DisabledUnselectedTrackColor`
pub const DISABLED_UNSELECTED_TRACK_COLOR: ColorRole = ColorRole::SurfaceContainerHighest;
/// `DisabledUnselectedTrackOutlineColor`
pub const DISABLED_UNSELECTED_TRACK_OUTLINE_COLOR: ColorRole = ColorRole::OnSurface;
/// `FocusIndicatorColor`
pub const FOCUS_INDICATOR_COLOR: ColorRole = ColorRole::Secondary;
/// `HandleShape`
pub const HANDLE_SHAPE: Shape = Shape::Full;
/// `PressedHandleHeight`
pub const PRESSED_HANDLE_HEIGHT: f32 = 28.0;
/// `PressedHandleWidth`
pub const PRESSED_HANDLE_WIDTH: f32 = 28.0;
/// `SelectedFocusHandleColor`
pub const SELECTED_FOCUS_HANDLE_COLOR: ColorRole = ColorRole::PrimaryContainer;
/// `SelectedFocusIconColor`
pub const SELECTED_FOCUS_ICON_COLOR: ColorRole = ColorRole::OnPrimaryContainer;
/// `SelectedFocusTrackColor`
pub const SELECTED_FOCUS_TRACK_COLOR: ColorRole = ColorRole::Primary;
/// `SelectedHandleColor`
pub const SELECTED_HANDLE_COLOR: ColorRole = ColorRole::OnPrimary;
/// `SelectedHandleHeight`
pub const SELECTED_HANDLE_HEIGHT: f32 = 24.0;
/// `SelectedHandleWidth`
pub const SELECTED_HANDLE_WIDTH: f32 = 24.0;
/// `SelectedHoverHandleColor`
pub const SELECTED_HOVER_HANDLE_COLOR: ColorRole = ColorRole::PrimaryContainer;
/// `SelectedHoverIconColor`
pub const SELECTED_HOVER_ICON_COLOR: ColorRole = ColorRole::OnPrimaryContainer;
/// `SelectedHoverTrackColor`
pub const SELECTED_HOVER_TRACK_COLOR: ColorRole = ColorRole::Primary;
/// `SelectedIconColor`
pub const SELECTED_ICON_COLOR: ColorRole = ColorRole::OnPrimaryContainer;
/// `SelectedIconSize`
pub const SELECTED_ICON_SIZE: f32 = 16.0;
/// `SelectedPressedHandleColor`
pub const SELECTED_PRESSED_HANDLE_COLOR: ColorRole = ColorRole::PrimaryContainer;
/// `SelectedPressedIconColor`
pub const SELECTED_PRESSED_ICON_COLOR: ColorRole = ColorRole::OnPrimaryContainer;
/// `SelectedPressedTrackColor`
pub const SELECTED_PRESSED_TRACK_COLOR: ColorRole = ColorRole::Primary;
/// `SelectedTrackColor`
pub const SELECTED_TRACK_COLOR: ColorRole = ColorRole::Primary;
/// `StateLayerShape`
pub const STATE_LAYER_SHAPE: Shape = Shape::Full;
/// `StateLayerSize`
pub const STATE_LAYER_SIZE: f32 = 40.0;
/// `TrackHeight`
pub const TRACK_HEIGHT: f32 = 32.0;
/// `TrackOutlineWidth`
pub const TRACK_OUTLINE_WIDTH: f32 = 2.0;
/// `TrackShape`
pub const TRACK_SHAPE: Shape = Shape::Full;
/// `TrackWidth`
pub const TRACK_WIDTH: f32 = 52.0;
/// `UnselectedFocusHandleColor`
pub const UNSELECTED_FOCUS_HANDLE_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedFocusIconColor`
pub const UNSELECTED_FOCUS_ICON_COLOR: ColorRole = ColorRole::SurfaceContainerHighest;
/// `UnselectedFocusTrackColor`
pub const UNSELECTED_FOCUS_TRACK_COLOR: ColorRole = ColorRole::SurfaceContainerHighest;
/// `UnselectedFocusTrackOutlineColor`
pub const UNSELECTED_FOCUS_TRACK_OUTLINE_COLOR: ColorRole = ColorRole::Outline;
/// `UnselectedHandleColor`
pub const UNSELECTED_HANDLE_COLOR: ColorRole = ColorRole::Outline;
/// `UnselectedHandleHeight`
pub const UNSELECTED_HANDLE_HEIGHT: f32 = 16.0;
/// `UnselectedHandleWidth`
pub const UNSELECTED_HANDLE_WIDTH: f32 = 16.0;
/// `UnselectedHoverHandleColor`
pub const UNSELECTED_HOVER_HANDLE_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedHoverIconColor`
pub const UNSELECTED_HOVER_ICON_COLOR: ColorRole = ColorRole::SurfaceContainerHighest;
/// `UnselectedHoverTrackColor`
pub const UNSELECTED_HOVER_TRACK_COLOR: ColorRole = ColorRole::SurfaceContainerHighest;
/// `UnselectedHoverTrackOutlineColor`
pub const UNSELECTED_HOVER_TRACK_OUTLINE_COLOR: ColorRole = ColorRole::Outline;
/// `UnselectedIconColor`
pub const UNSELECTED_ICON_COLOR: ColorRole = ColorRole::SurfaceContainerHighest;
/// `UnselectedIconSize`
pub const UNSELECTED_ICON_SIZE: f32 = 16.0;
/// `UnselectedPressedHandleColor`
pub const UNSELECTED_PRESSED_HANDLE_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedPressedIconColor`
pub const UNSELECTED_PRESSED_ICON_COLOR: ColorRole = ColorRole::SurfaceContainerHighest;
/// `UnselectedPressedTrackColor`
pub const UNSELECTED_PRESSED_TRACK_COLOR: ColorRole = ColorRole::SurfaceContainerHighest;
/// `UnselectedPressedTrackOutlineColor`
pub const UNSELECTED_PRESSED_TRACK_OUTLINE_COLOR: ColorRole = ColorRole::Outline;
/// `UnselectedTrackColor`
pub const UNSELECTED_TRACK_COLOR: ColorRole = ColorRole::SurfaceContainerHighest;
/// `UnselectedTrackOutlineColor`
pub const UNSELECTED_TRACK_OUTLINE_COLOR: ColorRole = ColorRole::Outline;
/// `IconHandleHeight`
pub const ICON_HANDLE_HEIGHT: f32 = 24.0;
/// `IconHandleWidth`
pub const ICON_HANDLE_WIDTH: f32 = 24.0;
