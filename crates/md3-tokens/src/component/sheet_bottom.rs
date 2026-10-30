// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/SheetBottomTokens.kt
//         @ b8256a52 (tokens VERSION v0_210)
// Copyright 2023 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! `SheetBottomTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::elevation;
use crate::shape::Shape;
use md3_color::ColorRole;

/// `DockedContainerColor`
pub const DOCKED_CONTAINER_COLOR: ColorRole = ColorRole::SurfaceContainerLow;
/// `DockedContainerShape`（上側の 2 つの角だけに適用する）
pub const DOCKED_CONTAINER_SHAPE: Shape = Shape::ExtraLarge;
/// `DockedDragHandleColor`
pub const DOCKED_DRAG_HANDLE_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `DockedDragHandleHeight`
pub const DOCKED_DRAG_HANDLE_HEIGHT: f32 = 4.0;
/// `DockedDragHandleWidth`
pub const DOCKED_DRAG_HANDLE_WIDTH: f32 = 32.0;
/// `DockedMinimizedContainerShape`
pub const DOCKED_MINIMIZED_CONTAINER_SHAPE: Shape = Shape::None;
/// `DockedModalContainerElevation`
pub const DOCKED_MODAL_CONTAINER_ELEVATION: f32 = elevation::LEVEL1;
/// `DockedStandardContainerElevation`
pub const DOCKED_STANDARD_CONTAINER_ELEVATION: f32 = elevation::LEVEL1;
/// `FocusIndicatorColor`
pub const FOCUS_INDICATOR_COLOR: ColorRole = ColorRole::Secondary;
