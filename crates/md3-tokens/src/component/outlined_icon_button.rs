// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/OutlinedIconButtonTokens.kt
//         @ b8256a52 (tokens VERSION 14_1_0)
// Copyright 2022 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! `OutlinedIconButtonTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use md3_color::ColorRole;

/// `DisabledColor`
pub const DISABLED_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledOpacity`
pub const DISABLED_OPACITY: f32 = 0.38;
/// `DisabledOutlineColor`
pub const DISABLED_OUTLINE_COLOR: ColorRole = ColorRole::OutlineVariant;
/// `FocusedColor`
pub const FOCUSED_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `HoveredColor`
pub const HOVERED_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `Color`
pub const COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `OutlineColor`
pub const OUTLINE_COLOR: ColorRole = ColorRole::OutlineVariant;
/// `PressedColor`
pub const PRESSED_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `SelectedContainerColor`
pub const SELECTED_CONTAINER_COLOR: ColorRole = ColorRole::InverseSurface;
/// `SelectedDisabledContainerColor`
pub const SELECTED_DISABLED_CONTAINER_COLOR: ColorRole = ColorRole::OnSurface;
/// `SelectedDisabledContainerOpacity`
pub const SELECTED_DISABLED_CONTAINER_OPACITY: f32 = 0.1;
/// `SelectedFocusedColor`
pub const SELECTED_FOCUSED_COLOR: ColorRole = ColorRole::InverseOnSurface;
/// `SelectedHoveredColor`
pub const SELECTED_HOVERED_COLOR: ColorRole = ColorRole::InverseOnSurface;
/// `SelectedColor`
pub const SELECTED_COLOR: ColorRole = ColorRole::InverseOnSurface;
/// `SelectedPressedColor`
pub const SELECTED_PRESSED_COLOR: ColorRole = ColorRole::InverseOnSurface;
/// `UnselectedDisabledOutlineColor`
pub const UNSELECTED_DISABLED_OUTLINE_COLOR: ColorRole = ColorRole::OutlineVariant;
/// `UnselectedFocusedColor`
pub const UNSELECTED_FOCUSED_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedHoveredColor`
pub const UNSELECTED_HOVERED_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedColor`
pub const UNSELECTED_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedOutlineColor`
pub const UNSELECTED_OUTLINE_COLOR: ColorRole = ColorRole::OutlineVariant;
/// `UnselectedPressedColor`
pub const UNSELECTED_PRESSED_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
