// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/InputChipTokens.kt
//         @ b8256a52 (tokens VERSION 7_0_1)
// Copyright 2025 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! `InputChipTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::elevation;
use crate::shape::Shape;
use crate::typography::TypeScale;
use md3_color::ColorRole;

/// `ContainerElevation`
pub const CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `ContainerHeight`
pub const CONTAINER_HEIGHT: f32 = 32.0;
/// `ContainerShape`
pub const CONTAINER_SHAPE: Shape = Shape::Small;
/// `DisabledLabelTextColor`
pub const DISABLED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledLabelTextOpacity`
pub const DISABLED_LABEL_TEXT_OPACITY: f32 = 0.38;
/// `DisabledSelectedContainerColor`
pub const DISABLED_SELECTED_CONTAINER_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledSelectedContainerOpacity`
pub const DISABLED_SELECTED_CONTAINER_OPACITY: f32 = 0.12;
/// `DisabledUnselectedOutlineColor`
pub const DISABLED_UNSELECTED_OUTLINE_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledUnselectedOutlineOpacity`
pub const DISABLED_UNSELECTED_OUTLINE_OPACITY: f32 = 0.12;
/// `DraggedContainerElevation`
pub const DRAGGED_CONTAINER_ELEVATION: f32 = elevation::LEVEL4;
/// `FocusIndicatorColor`
pub const FOCUS_INDICATOR_COLOR: ColorRole = ColorRole::Secondary;
/// `LabelTextFont`
pub const LABEL_TEXT_FONT: TypeScale = TypeScale::LabelLarge;
/// `SelectedContainerColor`
pub const SELECTED_CONTAINER_COLOR: ColorRole = ColorRole::SecondaryContainer;
/// `SelectedDraggedLabelTextColor`
pub const SELECTED_DRAGGED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `SelectedFocusLabelTextColor`
pub const SELECTED_FOCUS_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `SelectedHoverLabelTextColor`
pub const SELECTED_HOVER_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `SelectedLabelTextColor`
pub const SELECTED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `SelectedOutlineWidth`
pub const SELECTED_OUTLINE_WIDTH: f32 = 0.0;
/// `SelectedPressedLabelTextColor`
pub const SELECTED_PRESSED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `UnselectedDraggedLabelTextColor`
pub const UNSELECTED_DRAGGED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedFocusLabelTextColor`
pub const UNSELECTED_FOCUS_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedFocusOutlineColor`
pub const UNSELECTED_FOCUS_OUTLINE_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedHoverLabelTextColor`
pub const UNSELECTED_HOVER_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedLabelTextColor`
pub const UNSELECTED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedOutlineColor`
pub const UNSELECTED_OUTLINE_COLOR: ColorRole = ColorRole::OutlineVariant;
/// `UnselectedOutlineWidth`
pub const UNSELECTED_OUTLINE_WIDTH: f32 = 1.0;
/// `UnselectedPressedLabelTextColor`
pub const UNSELECTED_PRESSED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `AvatarShape`
pub const AVATAR_SHAPE: Shape = Shape::Full;
/// `AvatarSize`
pub const AVATAR_SIZE: f32 = 24.0;
/// `DisabledAvatarOpacity`
pub const DISABLED_AVATAR_OPACITY: f32 = 0.38;
/// `DisabledLeadingIconColor`
pub const DISABLED_LEADING_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledLeadingIconOpacity`
pub const DISABLED_LEADING_ICON_OPACITY: f32 = 0.38;
/// `LeadingIconSize`
pub const LEADING_ICON_SIZE: f32 = 18.0;
/// `SelectedDraggedLeadingIconColor`
pub const SELECTED_DRAGGED_LEADING_ICON_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `SelectedFocusLeadingIconColor`
pub const SELECTED_FOCUS_LEADING_ICON_COLOR: ColorRole = ColorRole::Primary;
/// `SelectedHoverLeadingIconColor`
pub const SELECTED_HOVER_LEADING_ICON_COLOR: ColorRole = ColorRole::Primary;
/// `SelectedLeadingIconColor`
pub const SELECTED_LEADING_ICON_COLOR: ColorRole = ColorRole::Primary;
/// `SelectedPressedLeadingIconColor`
pub const SELECTED_PRESSED_LEADING_ICON_COLOR: ColorRole = ColorRole::Primary;
/// `UnselectedDraggedLeadingIconColor`
pub const UNSELECTED_DRAGGED_LEADING_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedFocusLeadingIconColor`
pub const UNSELECTED_FOCUS_LEADING_ICON_COLOR: ColorRole = ColorRole::Primary;
/// `UnselectedHoverLeadingIconColor`
pub const UNSELECTED_HOVER_LEADING_ICON_COLOR: ColorRole = ColorRole::Primary;
/// `UnselectedLeadingIconColor`
pub const UNSELECTED_LEADING_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedPressedLeadingIconColor`
pub const UNSELECTED_PRESSED_LEADING_ICON_COLOR: ColorRole = ColorRole::Primary;
/// `DisabledTrailingIconColor`
pub const DISABLED_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledTrailingIconOpacity`
pub const DISABLED_TRAILING_ICON_OPACITY: f32 = 0.38;
/// `SelectedDraggedTrailingIconColor`
pub const SELECTED_DRAGGED_TRAILING_ICON_COLOR: ColorRole = ColorRole::Primary;
/// `SelectedFocusTrailingIconColor`
pub const SELECTED_FOCUS_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `SelectedHoverTrailingIconColor`
pub const SELECTED_HOVER_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `SelectedPressedTrailingIconColor`
pub const SELECTED_PRESSED_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `SelectedTrailingIconColor`
pub const SELECTED_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `TrailingIconSize`
pub const TRAILING_ICON_SIZE: f32 = 18.0;
/// `UnselectedDraggedTrailingIconColor`
pub const UNSELECTED_DRAGGED_TRAILING_ICON_COLOR: ColorRole = ColorRole::Primary;
/// `UnselectedFocusTrailingIconColor`
pub const UNSELECTED_FOCUS_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedHoverTrailingIconColor`
pub const UNSELECTED_HOVER_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedPressedTrailingIconColor`
pub const UNSELECTED_PRESSED_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedTrailingIconColor`
pub const UNSELECTED_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
