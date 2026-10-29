// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/NavigationRailCollapsedTokens.kt
//         @ b8256a52 (tokens VERSION v0_11_0)
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! `NavigationRailCollapsedTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::elevation;
use crate::shape::Shape;
use md3_color::ColorRole;

/// `ContainerElevation`
pub const CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `ContainerShape`
pub const CONTAINER_SHAPE: Shape = Shape::None;
/// `ContainerWidth`
pub const CONTAINER_WIDTH: f32 = 96.0;
/// `ItemVerticalSpace`
pub const ITEM_VERTICAL_SPACE: f32 = 4.0;
/// `TopSpace`
pub const TOP_SPACE: f32 = 44.0;
/// `ContainerColor`
pub const CONTAINER_COLOR: ColorRole = ColorRole::Surface;
/// `NarrowContainerWidth`
pub const NARROW_CONTAINER_WIDTH: f32 = 80.0;
