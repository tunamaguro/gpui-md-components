// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/FabPrimaryContainerTokens.kt
//         @ b8256a52 (tokens VERSION v0_14_0)
// Copyright 2024 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! `FabPrimaryContainerTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::elevation;
use md3_color::ColorRole;

/// `ContainerColor`
pub const CONTAINER_COLOR: ColorRole = ColorRole::PrimaryContainer;
/// `ContainerElevation`
pub const CONTAINER_ELEVATION: f32 = elevation::LEVEL3;
/// `FocusedContainerElevation`
pub const FOCUSED_CONTAINER_ELEVATION: f32 = elevation::LEVEL3;
/// `FocusedIconColor`
pub const FOCUSED_ICON_COLOR: ColorRole = ColorRole::OnPrimaryContainer;
/// `HoveredContainerElevation`
pub const HOVERED_CONTAINER_ELEVATION: f32 = elevation::LEVEL4;
/// `HoveredIconColor`
pub const HOVERED_ICON_COLOR: ColorRole = ColorRole::OnPrimaryContainer;
/// `IconColor`
pub const ICON_COLOR: ColorRole = ColorRole::OnPrimaryContainer;
/// `PressedContainerElevation`
pub const PRESSED_CONTAINER_ELEVATION: f32 = elevation::LEVEL3;
/// `PressedIconColor`
pub const PRESSED_ICON_COLOR: ColorRole = ColorRole::OnPrimaryContainer;
