// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/IconButtonTokens.kt
//         @ b8256a52 (tokens VERSION 14_1_0)
// Copyright 2025 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! `StandardIconButtonTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use md3_color::ColorRole;

/// `DisabledColor`
pub const DISABLED_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledOpacity`
pub const DISABLED_OPACITY: f32 = 0.38;
/// `FocusedColor`
pub const FOCUSED_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `HoveredColor`
pub const HOVERED_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `Color`
pub const COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `PressedColor`
pub const PRESSED_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `SelectedFocusedColor`
pub const SELECTED_FOCUSED_COLOR: ColorRole = ColorRole::Primary;
/// `SelectedHoveredColor`
pub const SELECTED_HOVERED_COLOR: ColorRole = ColorRole::Primary;
/// `SelectedColor`
pub const SELECTED_COLOR: ColorRole = ColorRole::Primary;
/// `SelectedPressedColor`
pub const SELECTED_PRESSED_COLOR: ColorRole = ColorRole::Primary;
/// `UnselectedFocusedColor`
pub const UNSELECTED_FOCUSED_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedHoveredColor`
pub const UNSELECTED_HOVERED_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedColor`
pub const UNSELECTED_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `UnselectedPressedColor`
pub const UNSELECTED_PRESSED_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
