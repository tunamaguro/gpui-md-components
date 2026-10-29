// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/ElevatedButtonTokens.kt
//         @ b8256a52 (tokens VERSION v0_11_0)
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! `ElevatedButtonTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::elevation;
use md3_color::ColorRole;

/// `ContainerColor`
pub const CONTAINER_COLOR: ColorRole = ColorRole::SurfaceContainerLow;
/// `ContainerElevation`
pub const CONTAINER_ELEVATION: f32 = elevation::LEVEL1;
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
pub const FOCUSED_CONTAINER_ELEVATION: f32 = elevation::LEVEL1;
/// `FocusedIconColor`
pub const FOCUSED_ICON_COLOR: ColorRole = ColorRole::Primary;
/// `FocusedLabelTextColor`
pub const FOCUSED_LABEL_TEXT_COLOR: ColorRole = ColorRole::Primary;
/// `HoveredContainerElevation`
pub const HOVERED_CONTAINER_ELEVATION: f32 = elevation::LEVEL2;
/// `HoveredIconColor`
pub const HOVERED_ICON_COLOR: ColorRole = ColorRole::Primary;
/// `HoveredLabelTextColor`
pub const HOVERED_LABEL_TEXT_COLOR: ColorRole = ColorRole::Primary;
/// `IconColor`
pub const ICON_COLOR: ColorRole = ColorRole::Primary;
/// `LabelTextColor`
pub const LABEL_TEXT_COLOR: ColorRole = ColorRole::Primary;
/// `LabelTextSelectedColor`
pub const LABEL_TEXT_SELECTED_COLOR: ColorRole = ColorRole::OnPrimary;
/// `LabelTextUnselectedColor`
pub const LABEL_TEXT_UNSELECTED_COLOR: ColorRole = ColorRole::Primary;
/// `PressedContainerElevation`
pub const PRESSED_CONTAINER_ELEVATION: f32 = elevation::LEVEL1;
/// `PressedIconColor`
pub const PRESSED_ICON_COLOR: ColorRole = ColorRole::Primary;
/// `PressedLabelTextColor`
pub const PRESSED_LABEL_TEXT_COLOR: ColorRole = ColorRole::Primary;
/// `SelectedContainerColor`
pub const SELECTED_CONTAINER_COLOR: ColorRole = ColorRole::Primary;
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
/// `UnselectedContainerColor`
pub const UNSELECTED_CONTAINER_COLOR: ColorRole = ColorRole::SurfaceContainerLow;
/// `UnselectedFocusedIconColor`
pub const UNSELECTED_FOCUSED_ICON_COLOR: ColorRole = ColorRole::Primary;
/// `UnselectedFocusedLabelTextColor`
pub const UNSELECTED_FOCUSED_LABEL_TEXT_COLOR: ColorRole = ColorRole::Primary;
/// `UnselectedHoveredIconColor`
pub const UNSELECTED_HOVERED_ICON_COLOR: ColorRole = ColorRole::Primary;
/// `UnselectedHoveredLabelTextColor`
pub const UNSELECTED_HOVERED_LABEL_TEXT_COLOR: ColorRole = ColorRole::Primary;
/// `UnselectedIconColor`
pub const UNSELECTED_ICON_COLOR: ColorRole = ColorRole::OnPrimary;
/// `UnselectedPressedIconColor`
pub const UNSELECTED_PRESSED_ICON_COLOR: ColorRole = ColorRole::Primary;
/// `UnselectedPressedLabelTextColor`
pub const UNSELECTED_PRESSED_LABEL_TEXT_COLOR: ColorRole = ColorRole::Primary;
