// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/OutlinedCardTokens.kt
//         @ b8256a52 (tokens VERSION v0_192)
// Copyright 2022 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! `OutlinedCardTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::elevation;
use crate::shape::Shape;
use md3_color::ColorRole;

/// `ContainerColor`
pub const CONTAINER_COLOR: ColorRole = ColorRole::Surface;
/// `ContainerElevation`
pub const CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `ContainerShape`
pub const CONTAINER_SHAPE: Shape = Shape::Medium;
/// `DisabledContainerElevation`
pub const DISABLED_CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `DisabledOutlineColor`
pub const DISABLED_OUTLINE_COLOR: ColorRole = ColorRole::Outline;
/// `DisabledOutlineOpacity`
pub const DISABLED_OUTLINE_OPACITY: f32 = 0.12;
/// `DraggedContainerElevation`
pub const DRAGGED_CONTAINER_ELEVATION: f32 = elevation::LEVEL3;
/// `DraggedOutlineColor`
pub const DRAGGED_OUTLINE_COLOR: ColorRole = ColorRole::OutlineVariant;
/// `FocusContainerElevation`
pub const FOCUS_CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `FocusOutlineColor`
pub const FOCUS_OUTLINE_COLOR: ColorRole = ColorRole::OnSurface;
/// `HoverContainerElevation`
pub const HOVER_CONTAINER_ELEVATION: f32 = elevation::LEVEL1;
/// `HoverOutlineColor`
pub const HOVER_OUTLINE_COLOR: ColorRole = ColorRole::OutlineVariant;
/// `IconColor`
pub const ICON_COLOR: ColorRole = ColorRole::Primary;
/// `IconSize`
pub const ICON_SIZE: f32 = 24.0;
/// `OutlineColor`
pub const OUTLINE_COLOR: ColorRole = ColorRole::OutlineVariant;
/// `OutlineWidth`
pub const OUTLINE_WIDTH: f32 = 1.0;
/// `PressedContainerElevation`
pub const PRESSED_CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `PressedOutlineColor`
pub const PRESSED_OUTLINE_COLOR: ColorRole = ColorRole::OutlineVariant;
