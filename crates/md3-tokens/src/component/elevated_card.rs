// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/ElevatedCardTokens.kt
//         @ b8256a52 (tokens VERSION v0_210)
// Copyright 2022 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! `ElevatedCardTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::elevation;
use crate::shape::Shape;
use md3_color::ColorRole;

/// `ContainerColor`
pub const CONTAINER_COLOR: ColorRole = ColorRole::SurfaceContainerLow;
/// `ContainerElevation`
pub const CONTAINER_ELEVATION: f32 = elevation::LEVEL1;
/// `ContainerShape`
pub const CONTAINER_SHAPE: Shape = Shape::Medium;
/// `DisabledContainerColor`
pub const DISABLED_CONTAINER_COLOR: ColorRole = ColorRole::Surface;
/// `DisabledContainerElevation`
pub const DISABLED_CONTAINER_ELEVATION: f32 = elevation::LEVEL1;
/// `DisabledContainerOpacity`
pub const DISABLED_CONTAINER_OPACITY: f32 = 0.38;
/// `DraggedContainerElevation`
pub const DRAGGED_CONTAINER_ELEVATION: f32 = elevation::LEVEL4;
/// `FocusContainerElevation`
pub const FOCUS_CONTAINER_ELEVATION: f32 = elevation::LEVEL1;
/// `FocusIndicatorColor`
pub const FOCUS_INDICATOR_COLOR: ColorRole = ColorRole::Secondary;
/// `HoverContainerElevation`
pub const HOVER_CONTAINER_ELEVATION: f32 = elevation::LEVEL2;
/// `IconColor`
pub const ICON_COLOR: ColorRole = ColorRole::Primary;
/// `IconSize`
pub const ICON_SIZE: f32 = 24.0;
/// `PressedContainerElevation`
pub const PRESSED_CONTAINER_ELEVATION: f32 = elevation::LEVEL1;
