// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/SearchBarTokens.kt
//         @ b8256a52 (tokens VERSION v0_210)
// Copyright 2025 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! `SearchBarTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::elevation;
use crate::shape::Shape;
use crate::typography::TypeScale;
use md3_color::ColorRole;

/// `AvatarShape`
pub const AVATAR_SHAPE: Shape = Shape::Full;
/// `AvatarSize`
pub const AVATAR_SIZE: f32 = 30.0;
/// `ContainerColor`
pub const CONTAINER_COLOR: ColorRole = ColorRole::SurfaceContainerHigh;
/// `ContainerElevation`
pub const CONTAINER_ELEVATION: f32 = elevation::LEVEL3;
/// `ContainerHeight`
pub const CONTAINER_HEIGHT: f32 = 56.0;
/// `ContainerShape`
pub const CONTAINER_SHAPE: Shape = Shape::Full;
/// `FocusIndicatorColor`
pub const FOCUS_INDICATOR_COLOR: ColorRole = ColorRole::Secondary;
/// `HoverSupportingTextColor`
pub const HOVER_SUPPORTING_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `InputTextColor`
pub const INPUT_TEXT_COLOR: ColorRole = ColorRole::OnSurface;
/// `InputTextFont`
pub const INPUT_TEXT_FONT: TypeScale = TypeScale::BodyLarge;
/// `LeadingIconColor`
pub const LEADING_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `PressedSupportingTextColor`
pub const PRESSED_SUPPORTING_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `SupportingTextColor`
pub const SUPPORTING_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `SupportingTextFont`
pub const SUPPORTING_TEXT_FONT: TypeScale = TypeScale::BodyLarge;
/// `TrailingIconColor`
pub const TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
