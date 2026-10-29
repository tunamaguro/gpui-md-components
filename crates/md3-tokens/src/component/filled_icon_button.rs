// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/FilledIconButtonTokens.kt
//         @ b8256a52 (tokens VERSION 14_1_0)
// Copyright 2022 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! `FilledIconButtonTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use md3_color::ColorRole;

/// `ContainerColor`
pub const CONTAINER_COLOR: ColorRole = ColorRole::Primary;
/// `DisabledContainerColor`
pub const DISABLED_CONTAINER_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledContainerOpacity`
pub const DISABLED_CONTAINER_OPACITY: f32 = 0.1;
/// `DisabledColor`
pub const DISABLED_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledOpacity`
pub const DISABLED_OPACITY: f32 = 0.38;
/// `FocusedColor`
pub const FOCUSED_COLOR: ColorRole = ColorRole::OnPrimary;
/// `HoveredColor`
pub const HOVERED_COLOR: ColorRole = ColorRole::OnPrimary;
/// `Color`
pub const COLOR: ColorRole = ColorRole::OnPrimary;
/// `PressedColor`
pub const PRESSED_COLOR: ColorRole = ColorRole::OnPrimary;
/// `SelectedContainerColor`
pub const SELECTED_CONTAINER_COLOR: ColorRole = ColorRole::Primary;
/// `SelectedFocusedColor`
pub const SELECTED_FOCUSED_COLOR: ColorRole = ColorRole::OnPrimary;
/// `SelectedHoveredColor`
pub const SELECTED_HOVERED_COLOR: ColorRole = ColorRole::OnPrimary;
/// `SelectedColor`
pub const SELECTED_COLOR: ColorRole = ColorRole::OnPrimary;
/// `SelectedPressedColor`
pub const SELECTED_PRESSED_COLOR: ColorRole = ColorRole::OnPrimary;
/// `UnselectedContainerColor`
pub const UNSELECTED_CONTAINER_COLOR: ColorRole = ColorRole::SurfaceContainer;
/// `UnselectedFocusedColor`
pub const UNSELECTED_FOCUSED_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedHoveredColor`
pub const UNSELECTED_HOVERED_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedColor`
pub const UNSELECTED_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedPressedColor`
pub const UNSELECTED_PRESSED_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
