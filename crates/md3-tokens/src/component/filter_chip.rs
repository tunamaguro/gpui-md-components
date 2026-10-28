// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/FilterChipTokens.kt
//         @ b8256a52 (tokens VERSION 7_0_1)
// Copyright 2025 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! `FilterChipTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::elevation;
use crate::shape::Shape;
use crate::typography::TypeScale;
use md3_color::ColorRole;

/// `ContainerHeight`
pub const CONTAINER_HEIGHT: f32 = 32.0;
/// `ContainerShape`
pub const CONTAINER_SHAPE: Shape = Shape::Small;
/// `DisabledLabelTextColor`
pub const DISABLED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledLabelTextOpacity`
pub const DISABLED_LABEL_TEXT_OPACITY: f32 = 0.38;
/// `DraggedContainerElevation`
pub const DRAGGED_CONTAINER_ELEVATION: f32 = elevation::LEVEL4;
/// `ElevatedContainerElevation`
pub const ELEVATED_CONTAINER_ELEVATION: f32 = elevation::LEVEL1;
/// `ElevatedDisabledContainerColor`
pub const ELEVATED_DISABLED_CONTAINER_COLOR: ColorRole = ColorRole::OnSurface;
/// `ElevatedDisabledContainerElevation`
pub const ELEVATED_DISABLED_CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `ElevatedDisabledContainerOpacity`
pub const ELEVATED_DISABLED_CONTAINER_OPACITY: f32 = 0.12;
/// `ElevatedFocusContainerElevation`
pub const ELEVATED_FOCUS_CONTAINER_ELEVATION: f32 = elevation::LEVEL1;
/// `ElevatedHoverContainerElevation`
pub const ELEVATED_HOVER_CONTAINER_ELEVATION: f32 = elevation::LEVEL2;
/// `ElevatedPressedContainerElevation`
pub const ELEVATED_PRESSED_CONTAINER_ELEVATION: f32 = elevation::LEVEL1;
/// `ElevatedSelectedContainerColor`
pub const ELEVATED_SELECTED_CONTAINER_COLOR: ColorRole = ColorRole::SecondaryContainer;
/// `ElevatedUnselectedContainerColor`
pub const ELEVATED_UNSELECTED_CONTAINER_COLOR: ColorRole = ColorRole::SurfaceContainerLow;
/// `FlatContainerElevation`
pub const FLAT_CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `FlatDisabledSelectedContainerColor`
pub const FLAT_DISABLED_SELECTED_CONTAINER_COLOR: ColorRole = ColorRole::OnSurface;
/// `FlatDisabledSelectedContainerOpacity`
pub const FLAT_DISABLED_SELECTED_CONTAINER_OPACITY: f32 = 0.12;
/// `FlatDisabledUnselectedOutlineColor`
pub const FLAT_DISABLED_UNSELECTED_OUTLINE_COLOR: ColorRole = ColorRole::OnSurface;
/// `FlatDisabledUnselectedOutlineOpacity`
pub const FLAT_DISABLED_UNSELECTED_OUTLINE_OPACITY: f32 = 0.12;
/// `FlatSelectedContainerColor`
pub const FLAT_SELECTED_CONTAINER_COLOR: ColorRole = ColorRole::SecondaryContainer;
/// `FlatSelectedFocusContainerElevation`
pub const FLAT_SELECTED_FOCUS_CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `FlatSelectedHoverContainerElevation`
pub const FLAT_SELECTED_HOVER_CONTAINER_ELEVATION: f32 = elevation::LEVEL1;
/// `FlatSelectedOutlineWidth`
pub const FLAT_SELECTED_OUTLINE_WIDTH: f32 = 0.0;
/// `FlatSelectedPressedContainerElevation`
pub const FLAT_SELECTED_PRESSED_CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `FlatUnselectedFocusContainerElevation`
pub const FLAT_UNSELECTED_FOCUS_CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `FlatUnselectedFocusOutlineColor`
pub const FLAT_UNSELECTED_FOCUS_OUTLINE_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `FlatUnselectedHoverContainerElevation`
pub const FLAT_UNSELECTED_HOVER_CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `FlatUnselectedOutlineColor`
pub const FLAT_UNSELECTED_OUTLINE_COLOR: ColorRole = ColorRole::OutlineVariant;
/// `FlatUnselectedOutlineWidth`
pub const FLAT_UNSELECTED_OUTLINE_WIDTH: f32 = 1.0;
/// `FlatUnselectedPressedContainerElevation`
pub const FLAT_UNSELECTED_PRESSED_CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `FocusIndicatorColor`
pub const FOCUS_INDICATOR_COLOR: ColorRole = ColorRole::Secondary;
/// `LabelTextFont`
pub const LABEL_TEXT_FONT: TypeScale = TypeScale::LabelLarge;
/// `SelectedDraggedLabelTextColor`
pub const SELECTED_DRAGGED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `SelectedFocusLabelTextColor`
pub const SELECTED_FOCUS_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `SelectedHoverLabelTextColor`
pub const SELECTED_HOVER_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `SelectedLabelTextColor`
pub const SELECTED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `SelectedPressedLabelTextColor`
pub const SELECTED_PRESSED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `UnselectedDraggedLabelTextColor`
pub const UNSELECTED_DRAGGED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedFocusLabelTextColor`
pub const UNSELECTED_FOCUS_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedHoverLabelTextColor`
pub const UNSELECTED_HOVER_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedLabelTextColor`
pub const UNSELECTED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedPressedLabelTextColor`
pub const UNSELECTED_PRESSED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `IconSize`
pub const ICON_SIZE: f32 = 18.0;
/// `DisabledLeadingIconColor`
pub const DISABLED_LEADING_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledLeadingIconOpacity`
pub const DISABLED_LEADING_ICON_OPACITY: f32 = 0.38;
/// `SelectedDraggedLeadingIconColor`
pub const SELECTED_DRAGGED_LEADING_ICON_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `SelectedFocusLeadingIconColor`
pub const SELECTED_FOCUS_LEADING_ICON_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `SelectedHoverLeadingIconColor`
pub const SELECTED_HOVER_LEADING_ICON_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `SelectedLeadingIconColor`
pub const SELECTED_LEADING_ICON_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `SelectedPressedLeadingIconColor`
pub const SELECTED_PRESSED_LEADING_ICON_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `UnselectedDraggedLeadingIconColor`
pub const UNSELECTED_DRAGGED_LEADING_ICON_COLOR: ColorRole = ColorRole::Primary;
/// `UnselectedFocusLeadingIconColor`
pub const UNSELECTED_FOCUS_LEADING_ICON_COLOR: ColorRole = ColorRole::Primary;
/// `UnselectedHoverLeadingIconColor`
pub const UNSELECTED_HOVER_LEADING_ICON_COLOR: ColorRole = ColorRole::Primary;
/// `UnselectedLeadingIconColor`
pub const UNSELECTED_LEADING_ICON_COLOR: ColorRole = ColorRole::Primary;
/// `UnselectedPressedLeadingIconColor`
pub const UNSELECTED_PRESSED_LEADING_ICON_COLOR: ColorRole = ColorRole::Primary;
/// `DisabledTrailingIconColor`
pub const DISABLED_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledTrailingIconOpacity`
pub const DISABLED_TRAILING_ICON_OPACITY: f32 = 0.38;
/// `SelectedDraggedTrailingIconColor`
pub const SELECTED_DRAGGED_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `SelectedFocusTrailingIconColor`
pub const SELECTED_FOCUS_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `SelectedHoverTrailingIconColor`
pub const SELECTED_HOVER_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `SelectedPressedTrailingIconColor`
pub const SELECTED_PRESSED_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `SelectedTrailingIconColor`
pub const SELECTED_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `UnselectedDraggedTrailingIconColor`
pub const UNSELECTED_DRAGGED_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedFocusTrailingIconColor`
pub const UNSELECTED_FOCUS_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedHoverTrailingIconColor`
pub const UNSELECTED_HOVER_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedPressedTrailingIconColor`
pub const UNSELECTED_PRESSED_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedTrailingIconColor`
pub const UNSELECTED_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
