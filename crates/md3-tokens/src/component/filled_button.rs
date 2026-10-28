// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/FilledButtonTokens.kt
//         @ b8256a52 (tokens VERSION v0_11_0)
// Copyright 2025 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! `FilledButtonTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::elevation;
use md3_color::ColorRole;

/// `ContainerColor`
pub const CONTAINER_COLOR: ColorRole = ColorRole::Primary;
/// `ContainerElevation`
pub const CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
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
/// `LabelTextColor`
pub const LABEL_TEXT_COLOR: ColorRole = ColorRole::OnPrimary;
/// `LabelTextSelectedColor`
pub const LABEL_TEXT_SELECTED_COLOR: ColorRole = ColorRole::OnPrimary;
/// `LabelTextUnselectedColor`
pub const LABEL_TEXT_UNSELECTED_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `PressedContainerElevation`
pub const PRESSED_CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `PressedIconColor`
pub const PRESSED_ICON_COLOR: ColorRole = ColorRole::OnPrimary;
/// `PressedLabelTextColor`
pub const PRESSED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnPrimary;
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
